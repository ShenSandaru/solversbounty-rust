use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::http::error::{ApiError, ApiIssue, ApiResult};
use crate::modules::auth::constants::{
    CSRF_COOKIE, CSRF_HEADER, NONCE_RATE_LIMIT, NONCE_RATE_WINDOW_MS, NONCE_TTL_MS, SESSION_TTL_MS,
};
use crate::modules::auth::cookies::{
    clear_csrf_cookie, clear_session_cookie, csrf_cookie, csrf_tokens_match, read_named_cookie,
    read_session_cookie,
};
use crate::modules::auth::login_message::{build_login_message, LoginMessageInput};
use crate::modules::auth::repo;
use crate::modules::auth::session::{hash_session_token, random_token};
use crate::modules::solana::{canonical_wallet_address, decode_signature, verify_wallet_signature};
use crate::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBody {
    pub wallet_address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginBody {
    pub wallet_address: String,
    pub signature: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NonceResponse {
    pub nonce: String,
    pub message: String,
    pub expires_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    pub id: String,
    pub wallet_address: String,
    pub role: String,
    pub csrf_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub wallet_address: String,
    pub role: String,
    pub csrf_token: String,
}

pub async fn issue_nonce(
    State(state): State<AppState>,
    Json(body): Json<WalletBody>,
) -> ApiResult<Json<NonceResponse>> {
    validate_wallet(&body.wallet_address)?;
    let pool = state.require_pool()?;
    let wallet = canonical_wallet_address(&body.wallet_address).map_err(|_| validation_wallet())?;
    let key = format!("auth-nonce:{}", wallet);
    {
        let mut limiter = state.rate_limit.lock().await;
        if !limiter.allow(&key, NONCE_RATE_LIMIT, NONCE_RATE_WINDOW_MS) {
            return Err(ApiError::rate_limited("Too many nonce requests."));
        }
    }
    let now = Utc::now();
    let expires_at = now + Duration::milliseconds(NONCE_TTL_MS);
    let mut nonce_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = URL_SAFE_NO_PAD.encode(nonce_bytes);
    repo::issue_nonce(pool, &wallet, &nonce, expires_at, now)
        .await
        .map_err(|_| ApiError::internal())?;
    let message = build_login_message(&LoginMessageInput {
        domain: state.config.login_domain.clone(),
        uri: state.config.login_uri.clone(),
        wallet_address: wallet.clone(),
        nonce: nonce.clone(),
        issued_at: now,
        expires_at,
    });
    Ok(Json(NonceResponse {
        nonce,
        message,
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> ApiResult<Response> {
    validate_wallet(&body.wallet_address)?;
    let pool = state.require_pool()?;
    let wallet = canonical_wallet_address(&body.wallet_address).map_err(|_| validation_wallet())?;
    let sig_bytes = decode_signature(&body.signature)
        .map_err(|_| ApiError::auth_rejected("Authentication failed."))?;
    let now = Utc::now();
    let active = repo::find_active_nonce(pool, &wallet, now)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::auth_rejected("Authentication failed."))?;
    let issued_at = active.expires_at - Duration::milliseconds(NONCE_TTL_MS);
    let message = build_login_message(&LoginMessageInput {
        domain: state.config.login_domain.clone(),
        uri: state.config.login_uri.clone(),
        wallet_address: wallet.clone(),
        nonce: active.nonce.clone(),
        issued_at,
        expires_at: active.expires_at,
    });
    if !verify_wallet_signature(&wallet, &message, &sig_bytes) {
        return Err(ApiError::auth_rejected("Authentication failed."));
    }
    let session_token = random_token();
    let csrf_token = random_token();
    let hash = hash_session_token(&session_token);
    let role = role_for(&state, &wallet);
    let session_expires = now + Duration::milliseconds(SESSION_TTL_MS);
    let saved = repo::consume_login(
        pool,
        &wallet,
        &active.nonce,
        &role,
        &hash,
        session_expires,
        now,
    )
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(|| ApiError::auth_rejected("Authentication failed."))?;
    let secure = state.config.node_env.secure_cookies();
    let cookies = login_cookies(&session_token, &csrf_token, secure);
    let body = LoginResponse {
        wallet_address: saved.wallet_address,
        role: saved.role,
        csrf_token: csrf_token.clone(),
    };
    Ok((StatusCode::OK, set_cookies(cookies), Json(body)).into_response())
}

pub async fn session(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    let pool = state.require_pool()?;
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    let token = read_session_cookie(cookie)
        .ok_or_else(|| ApiError::unauthenticated("Session cookie is missing."))?;
    let user = repo::find_active_session(pool, &hash_session_token(&token), Utc::now())
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::unauthenticated("Authentication required."))?;
    let secure = state.config.node_env.secure_cookies();
    let existing_csrf = read_named_cookie(cookie, CSRF_COOKIE);
    let (csrf_token, extra_cookie) = if let Some(csrf) = existing_csrf {
        (csrf, None)
    } else {
        let csrf = random_token();
        let line = csrf_cookie(&csrf, SESSION_TTL_MS / 1000, secure);
        (csrf, Some(line))
    };
    let body = SessionUser {
        id: user.id.to_string(),
        wallet_address: user.wallet_address,
        role: user.role,
        csrf_token,
    };
    let mut cookies = Vec::new();
    if let Some(line) = extra_cookie {
        cookies.push(line);
    }
    if cookies.is_empty() {
        Ok((StatusCode::OK, Json(body)).into_response())
    } else {
        Ok((StatusCode::OK, set_cookies(cookies), Json(body)).into_response())
    }
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    let pool = state.require_pool()?;
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    let csrf = headers.get(CSRF_HEADER).and_then(|v| v.to_str().ok());
    if !csrf_tokens_match(cookie, csrf) {
        return Err(ApiError::forbidden("CSRF token is invalid."));
    }
    if let Some(token) = read_session_cookie(cookie) {
        let _ = repo::revoke_session(pool, &hash_session_token(&token), Utc::now())
            .await
            .map_err(|_| ApiError::internal())?;
    }
    let secure = state.config.node_env.secure_cookies();
    let cookies = vec![clear_session_cookie(secure), clear_csrf_cookie(secure)];
    Ok((
        StatusCode::OK,
        set_cookies(cookies),
        Json(serde_json::json!({ "revoked": true })),
    )
        .into_response())
}

fn role_for(state: &AppState, wallet: &str) -> String {
    let admins: Vec<String> = state
        .config
        .admin_wallet_allowlist
        .iter()
        .filter_map(|w| canonical_wallet_address(w).ok())
        .collect();
    if admins.iter().any(|w| w == wallet) {
        "admin".into()
    } else {
        "user".into()
    }
}

fn validate_wallet(wallet: &str) -> ApiResult<()> {
    if wallet.len() < 32 || wallet.len() > 44 {
        return Err(validation_wallet());
    }
    Ok(())
}

fn validation_wallet() -> ApiError {
    ApiError::validation(
        "Validation failed.",
        vec![ApiIssue {
            path: "walletAddress".into(),
            message: "Wallet address must be 32 to 44 characters.".into(),
        }],
    )
}

pub fn set_cookies(lines: Vec<String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for line in lines {
        if let Ok(value) = axum::http::HeaderValue::from_str(&line) {
            headers.append(axum::http::header::SET_COOKIE, value);
        }
    }
    headers
}

pub fn login_cookies(session_token: &str, csrf_token: &str, secure: bool) -> Vec<String> {
    let max_age = SESSION_TTL_MS / 1000;
    vec![
        super::cookies::session_cookie(session_token, max_age, secure),
        super::cookies::csrf_cookie(csrf_token, max_age, secure),
    ]
}
