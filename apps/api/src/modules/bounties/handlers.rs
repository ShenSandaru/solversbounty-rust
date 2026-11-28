use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::http::error::{ApiError, ApiResult};
use crate::modules::auth::constants::CSRF_HEADER;
use crate::modules::auth::context::resolve_auth_user;
use crate::modules::auth::cookies::csrf_tokens_match;
use crate::modules::bounties::answer_cipher::{decrypt_answer, encrypt_answer, parse_answer_key};
use crate::modules::bounties::dto::{funding_view as funding_view_json, private_json, public_json};
use crate::modules::bounties::funding::normalize_funding_purpose;
use crate::modules::bounties::funding_match::match_obligation;
use crate::modules::bounties::repo::{self, ApplyFundingInput, ApplyFundingResult};
use crate::modules::bounties::validate::{parse_create_input, CreateBountyInput};
use crate::modules::solana::gateway::TransferOutcome;
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

pub async fn create_bounty(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let key = answer_key(&state)?;
    let input: CreateBountyInput = serde_json::from_value(body)
        .map_err(|_| ApiError::validation("Validation failed.", vec![]))?;
    let write = parse_create_input(input, Utc::now(), |plain| encrypt_answer(plain, &key))?;
    let row = repo::create_bounty(pool, user.id, &write)
        .await
        .map_err(|_| ApiError::internal())?;
    let expected =
        decrypt_answer(&row.expected_answer_ciphertext, &key).map_err(|_| ApiError::internal())?;
    Ok(Json(private_json(&row, &expected)))
}

pub async fn list_bounties(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let rows = repo::list_active(pool, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?;
    let q = query.q.as_deref().unwrap_or("").trim().to_lowercase();
    let out: Vec<Value> = rows
        .into_iter()
        .filter(|r| query.category.as_deref().is_none_or(|c| r.category == c))
        .filter(|r| {
            q.is_empty()
                || r.title.to_lowercase().contains(&q)
                || r.description.to_lowercase().contains(&q)
        })
        .map(|r| public_json(&r))
        .collect();
    Ok(Json(Value::Array(out)))
}

pub async fn public_bounty(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let id = parse_uuid(&id)?;
    let row = repo::find_public(pool, id, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Bounty not found."))?;
    Ok(Json(public_json(&row)))
}

pub async fn mine_bounties(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let key = answer_key(&state)?;
    let rows = repo::list_owned(pool, user.id)
        .await
        .map_err(|_| ApiError::internal())?;
    let out: Result<Vec<Value>, ApiError> = rows
        .iter()
        .map(|row| {
            let expected = decrypt_answer(&row.expected_answer_ciphertext, &key)
                .map_err(|_| ApiError::internal())?;
            Ok(private_json(row, &expected))
        })
        .collect();
    Ok(Json(Value::Array(out?)))
}

pub async fn private_bounty(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let id = parse_uuid(&id)?;
    let row = repo::find_owned(pool, id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Bounty not found."))?;
    let key = answer_key(&state)?;
    let expected =
        decrypt_answer(&row.expected_answer_ciphertext, &key).map_err(|_| ApiError::internal())?;
    Ok(Json(private_json(&row, &expected)))
}

pub async fn patch_bounty(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let id = parse_uuid(&id)?;
    let key = answer_key(&state)?;
    let input: CreateBountyInput = serde_json::from_value(body)
        .map_err(|_| ApiError::validation("Validation failed.", vec![]))?;
    let write = parse_create_input(input, Utc::now(), |plain| encrypt_answer(plain, &key))?;
    let updated = repo::update_draft(pool, id, user.id, &write)
        .await
        .map_err(|_| ApiError::internal())?;
    match updated {
        Ok(row) => {
            let mut decrypted = Vec::with_capacity(row.questions.len());
            for q in &row.questions {
                let ans = decrypt_answer(&q.expected_answer_ciphertext, &key)
                    .map_err(|_| ApiError::internal())?;
                decrypted.push(ans);
            }
            let expected = decrypted.first().cloned().unwrap_or_default();
            Ok(Json(crate::modules::bounties::dto::private_json_with_questions(
                &row,
                &expected,
                &decrypted,
            )))
        }
        Err("missing") => Err(ApiError::not_found("Bounty not found.")),
        Err("not_draft") => Err(ApiError::conflict("Only draft bounties can be edited.")),
        Err("invalid_question") => Err(ApiError::validation("Question does not belong to this bounty.", vec![])),
        Err(_) => Err(ApiError::internal()),
    }
}

pub async fn funding_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let id = parse_uuid(&id)?;
    let snapshot = prepare_funding(&state, pool, id, user.id).await?;
    let treasury = state.config.treasury_public_key.clone().unwrap_or_default();
    Ok(Json(funding_view_json(
        &snapshot.bounty,
        &snapshot.obligations,
        &snapshot.buckets,
        &treasury,
    )))
}

pub async fn submit_funding(
    State(state): State<AppState>,
    Path((id, purpose)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let bounty_id = parse_uuid(&id)?;
    let purpose = normalize_funding_purpose(&purpose)
        .ok_or_else(|| ApiError::validation("Validation failed.", vec![]))?;
    let signature = body
        .get("signature")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !is_base58_signature(&signature) {
        return Err(ApiError::validation(
            "Enter the Solana transaction signature.",
            vec![],
        ));
    }
    let treasury = state
        .config
        .treasury_public_key
        .as_deref()
        .ok_or_else(|| ApiError::not_ready("Treasury identity is not configured."))?;
    let wallet = user.wallet_address.clone();
    let snapshot = prepare_funding(&state, pool, bounty_id, user.id).await?;
    let obligation = snapshot
        .obligations
        .iter()
        .find(|o| o.purpose == purpose)
        .ok_or_else(|| ApiError::validation("Validation failed.", vec![]))?;
    if obligation.expected_lamports <= 0 {
        return Err(ApiError::validation(
            "The funding amount must be a positive number of lamports.",
            vec![],
        ));
    }
    if obligation.status == "confirmed" && obligation.signature.as_deref() == Some(&signature) {
        return Ok(Json(funding_view_json(
            &snapshot.bounty,
            &snapshot.obligations,
            &snapshot.buckets,
            treasury,
        )));
    }
    if obligation.status == "confirmed" {
        return Err(ApiError::conflict("That obligation is already confirmed."));
    }
    if let Some((bid, p)) = repo::find_signature_use(pool, &signature)
        .await
        .map_err(|_| ApiError::internal())?
    {
        if bid != bounty_id || p != purpose {
            return Err(ApiError::conflict(
                "That transaction already funds another obligation.",
            ));
        }
    }
    let lookup = state.payout_worker.gateway().lookup(&signature).await;
    if lookup.outcome == TransferOutcome::Unavailable {
        return Err(ApiError::not_ready(
            "The Solana RPC did not answer. Submit this same signature again. It was not marked failed.",
        ));
    }
    let transfer = lookup.transfer.ok_or_else(|| {
        ApiError::conflict(
            "The transaction is not confirmed yet. Submit this same signature again.",
        )
    })?;
    if transfer.commitment == "processed" {
        return Err(ApiError::conflict(
            "The transaction is not confirmed yet. Submit this same signature again.",
        ));
    }
    if transfer.cluster != "devnet" {
        return Err(ApiError::conflict(
            "The transaction is not on Solana Devnet.",
        ));
    }
    if transfer.failed {
        let _ = repo::apply_funding(
            pool,
            ApplyFundingInput {
                bounty_id,
                creator_id: user.id,
                purpose: purpose.to_string(),
                signature: signature.clone(),
                source_address: transfer.fee_payer.clone(),
                destination_address: treasury.to_string(),
                amount_lamports: obligation.expected_lamports,
                commitment: if transfer.commitment == "finalized" {
                    "finalized".into()
                } else {
                    "confirmed".into()
                },
                rejected: true,
                now: Utc::now(),
            },
        )
        .await;
        return Err(ApiError::conflict(
            "The transaction failed on Solana Devnet.",
        ));
    }
    let matched = match_obligation(
        &transfer,
        &wallet,
        treasury,
        obligation.expected_lamports,
        &obligation.deposit_reference,
    )
    .ok_or_else(|| {
        ApiError::conflict(
            "The transaction sender, treasury destination, amount, or deposit reference does not match this obligation.",
        )
    })?;
    let applied = repo::apply_funding(
        pool,
        ApplyFundingInput {
            bounty_id,
            creator_id: user.id,
            purpose: purpose.to_string(),
            signature,
            source_address: matched.0,
            destination_address: matched.1,
            amount_lamports: matched.2,
            commitment: if transfer.commitment == "finalized" {
                "finalized".into()
            } else {
                "confirmed".into()
            },
            rejected: false,
            now: Utc::now(),
        },
    )
    .await
    .map_err(|_| ApiError::internal())?;
    let snapshot = match applied {
        ApplyFundingResult::Applied(s) => s,
        ApplyFundingResult::Missing => return Err(ApiError::not_found("Bounty not found.")),
        ApplyFundingResult::SignatureInUse => {
            return Err(ApiError::conflict(
                "That transaction already funds another obligation.",
            ))
        }
        ApplyFundingResult::AlreadyConfirmed => {
            return Err(ApiError::conflict("That obligation is already confirmed."))
        }
    };
    Ok(Json(funding_view_json(
        &snapshot.bounty,
        &snapshot.obligations,
        &snapshot.buckets,
        treasury,
    )))
}

pub async fn activate_bounty(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    require_csrf(&headers)?;
    let user = resolve_auth_user(pool, &headers).await?;
    let id = parse_uuid(&id)?;
    let key = answer_key(&state)?;
    let result = repo::activate_if_funded(pool, id, user.id, Utc::now())
        .await
        .map_err(|_| ApiError::internal())?;
    match result {
        Ok(row) => {
            let expected = decrypt_answer(&row.expected_answer_ciphertext, &key)
                .map_err(|_| ApiError::internal())?;
            Ok(Json(private_json(&row, &expected)))
        }
        Err("missing") => Err(ApiError::not_found("Bounty not found.")),
        Err("closed" | "incomplete") => Err(ApiError::conflict(
            "Reward-pool funding and the separate 5% commission must both be verified before activation.",
        )),
        Err(_) => Err(ApiError::internal()),
    }
}

pub async fn bounty_analytics(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let pool = state.require_pool()?;
    let user = resolve_auth_user(pool, &headers).await?;
    let id = parse_uuid(&id)?;
    let aggregates = repo::creator_analytics(pool, id, user.id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Bounty not found."))?;
    let accuracy = crate::modules::bounties::dto::accuracy_text(
        aggregates.correct_count,
        aggregates.attempt_count,
    );
    Ok(Json(serde_json::json!({
        "participantCount": aggregates.participant_count,
        "correctCount": aggregates.correct_count,
        "incorrectCount": aggregates.incorrect_count,
        "attemptCount": aggregates.attempt_count,
        "averageCompletionTimeMs": aggregates.average_completion_time_ms,
        "accuracy": accuracy,
    })))
}

async fn prepare_funding(
    _state: &AppState,
    pool: &sqlx::PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
) -> ApiResult<repo::FundingSnapshot> {
    let row = repo::find_owned(pool, bounty_id, creator_id)
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or_else(|| ApiError::not_found("Bounty not found."))?;
    let (pool_amount, commission) =
        repo::expected_funding_amounts(row.reward_lamports, row.max_rewarded_solvers);
    let ensured = repo::ensure_funding(pool, bounty_id, creator_id, pool_amount, commission)
        .await
        .map_err(|_| ApiError::internal())?;
    match ensured {
        Ok(snapshot) => Ok(snapshot),
        Err("missing") => Err(ApiError::not_found("Bounty not found.")),
        Err("closed") => Err(ApiError::conflict("This bounty is closed.")),
        Err(_) => Err(ApiError::internal()),
    }
}

fn answer_key(state: &AppState) -> ApiResult<Vec<u8>> {
    parse_answer_key(state.config.answer_encryption_key.as_deref()).map_err(|e| ApiError {
        status: axum::http::StatusCode::from_u16(e.status)
            .unwrap_or(axum::http::StatusCode::SERVICE_UNAVAILABLE),
        code: crate::http::error::ApiErrorCode::NotReady,
        message: e.message,
        issues: None,
        checks: None,
    })
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

fn is_base58_signature(value: &str) -> bool {
    let len = value.len();
    (64..=88).contains(&len)
        && value.chars().all(
            |c| matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'),
        )
}
