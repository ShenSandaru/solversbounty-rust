use axum::http::HeaderMap;
use chrono::Utc;
use sqlx::PgPool;

use crate::http::error::{ApiError, ApiResult};
use crate::modules::auth::cookies::read_session_cookie;
use crate::modules::auth::repo::{self, AuthUser};
use crate::modules::auth::session::hash_session_token;

pub async fn resolve_auth_user(pool: &PgPool, headers: &HeaderMap) -> ApiResult<AuthUser> {
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    let token = read_session_cookie(cookie)
        .ok_or_else(|| ApiError::unauthenticated("Session cookie is missing."))?;
    let hash = hash_session_token(&token);
    let user = repo::find_active_session(pool, &hash, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::unauthenticated("Authentication required."))?;
    Ok(user)
}
