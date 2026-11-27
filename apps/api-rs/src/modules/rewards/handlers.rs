use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use serde_json::Value;
use uuid::Uuid;

use crate::http::error::{ApiError, ApiResult};
use crate::modules::auth::constants::CSRF_HEADER;
use crate::modules::auth::context::resolve_auth_user;
use crate::modules::auth::cookies::csrf_tokens_match;
use crate::modules::rewards::repo;
use crate::AppState;

pub async fn payouts_mine(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let rows = repo::solver_payouts(pool, user.id)
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(Value::Array(rows)))
}

pub async fn bounty_payouts(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let bounty_id = parse_uuid(&id)?;
    let summary = repo::creator_summary(pool, bounty_id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Bounty not found."))?;
    Ok(Json(summary))
}

pub async fn admin_payouts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    if user.role != "admin" {
        return Err(ApiError::forbidden("Forbidden."));
    }
    let rows = repo::admin_payouts(pool)
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(Value::Array(rows)))
}

pub async fn admin_run_payouts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    if user.role != "admin" {
        return Err(ApiError::forbidden("Forbidden."));
    }
    require_csrf(&headers)?;
    state.payout_worker.nudge().await;
    Ok(Json(serde_json::json!({ "processed": true })))
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
