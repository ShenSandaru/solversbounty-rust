use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::http::error::{ApiError, ApiResult};
use crate::modules::auth::constants::CSRF_HEADER;
use crate::modules::auth::context::resolve_auth_user;
use crate::modules::auth::cookies::csrf_tokens_match;
use crate::modules::bounties::answer_cipher::{decrypt_answer, parse_answer_key};
use crate::modules::bounties::dto::{session_result as session_result_json, session_view};
use crate::modules::bounties::repo::{self, SubmitAnswerResult};
use crate::modules::bounties::validate::RESEARCH_CONSENT_VERSION;
use crate::modules::verification::{answers_match, VerificationMethod};
use crate::AppState;

pub async fn consent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    if body.get("accepted").and_then(|v| v.as_bool()) != Some(true) {
        return Err(ApiError::validation("Validation failed.", vec![]));
    }
    let bounty_id = parse_uuid(&id)?;
    let saved = repo::record_consent(pool, bounty_id, user.id, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?;
    match saved {
        Ok((policy_version, accepted_at)) => Ok(Json(serde_json::json!({
            "policyVersion": policy_version,
            "acceptedAt": accepted_at.to_rfc3339(),
        }))),
        Err("missing") => Err(ApiError::not_found("Bounty not found.")),
        Err("inactive") => Err(ApiError::conflict(
            "This bounty is not open for new solvers.",
        )),
        Err(_) => Err(ApiError::internal()),
    }
}

pub async fn start_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let bounty_id = parse_uuid(&id)?;
    let started = repo::start_session(pool, bounty_id, user.id, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?;
    match started {
        Ok(session) => Ok(Json(session_view(&session))),
        Err("missing") => Err(ApiError::not_found("Bounty not found.")),
        Err("no_consent") => Err(ApiError::conflict(
            "Research consent is required before a solving session.",
        )),
        Err("full") => Err(ApiError::conflict(
            "This bounty has reached its rewarded solver cap.",
        )),
        Err("inactive") => Err(ApiError::conflict(
            "This bounty is not open for new solvers.",
        )),
        Err(_) => Err(ApiError::internal()),
    }
}

pub async fn read_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let session_id = parse_uuid(&id)?;
    let session = repo::read_session(pool, session_id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Solving session not found."))?;
    Ok(Json(session_view(&session)))
}

pub async fn session_result(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let session_id = parse_uuid(&id)?;
    let session = repo::read_session(pool, session_id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Solving session not found."))?;
    Ok(Json(session_result_json(&session)))
}

pub async fn submit_answer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let session_id = parse_uuid(&id)?;
    let answer = body
        .get("answer")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if answer.is_empty() {
        return Err(ApiError::validation("Validation failed.", vec![]));
    }
    {
        let mut limiter = state.rate_limit.lock().await;
        if !limiter.allow(&format!("submit:{}", user.id), 30, 60_000) {
            return Err(ApiError::rate_limited(
                "Too many submissions. Try again in a minute.",
            ));
        }
    }
    let key = parse_answer_key(state.config.answer_encryption_key.as_deref())
        .map_err(|e| ApiError::not_ready(e.message))?;
    let material = repo::verification_material(pool, session_id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Solving session not found."))?;
    let method = VerificationMethod::from_db(&material.1).ok_or_else(|| ApiError::internal())?;
    let expected = decrypt_answer(&material.0, &key).map_err(|_| ApiError::internal())?;
    let correct = answers_match(method, &expected, &answer);
    let saved = repo::submit_answer(pool, session_id, user.id, &answer, Utc::now(), correct)
        .await
        .map_err(|_| ApiError::internal())?;
    match saved {
        SubmitAnswerResult::Missing => Err(ApiError::not_found("Solving session not found.")),
        SubmitAnswerResult::Inactive => Err(ApiError::conflict(
            "This bounty is not open for another attempt.",
        )),
        SubmitAnswerResult::Limit | SubmitAnswerResult::Completed => {
            Err(ApiError::conflict("This solving session is complete."))
        }
        SubmitAnswerResult::Stored(session) => {
            if session.attempts.last().is_some_and(|a| a.reward_reserved) {
                state.payout_worker.nudge().await;
            }
            Ok(Json(session_view(&session)))
        }
    }
}

fn require_csrf(headers: &HeaderMap) -> ApiResult<()> {
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    let csrf = headers.get(CSRF_HEADER).and_then(|v| v.to_str().ok());
    if !csrf_tokens_match(cookie, csrf) {
        return Err(ApiError::forbidden("CSRF token is invalid."));
    }
    Ok(())
}

fn parse_uuid(value: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| ApiError::validation("Validation failed.", vec![]))
}

#[allow(dead_code)]
pub fn consent_version() -> &'static str {
    RESEARCH_CONSENT_VERSION
}
