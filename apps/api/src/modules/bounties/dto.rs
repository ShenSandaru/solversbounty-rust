use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::modules::bounties::funding_ledger::LedgerBuckets;
use crate::modules::bounties::lamports::{commission_lamports, principal_lamports};

#[derive(Debug, Clone)]
pub struct OwnerBounty {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub category: String,
    pub reward_lamports: i64,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: DateTime<Utc>,
    pub status: String,
    pub close_reason: Option<String>,
    pub prompt: String,
    pub expected_answer_ciphertext: String,
    pub encryption_key_id: String,
    pub verification_method: String,
}

#[derive(Debug, Clone)]
pub struct PublicBountyRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub prompt: String,
    pub category: String,
    pub verification_method: String,
    pub status: String,
    pub reward_lamports: i64,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: DateTime<Utc>,
    pub close_reason: Option<String>,
    pub rewarded_count: i32,
}

#[derive(Debug, Clone)]
pub struct StoredObligation {
    pub purpose: String,
    pub deposit_reference: String,
    pub expected_lamports: i64,
    pub observed_lamports: Option<i64>,
    pub status: String,
    pub signature: Option<String>,
    pub commitment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredSession {
    pub id: String,
    pub bounty_id: String,
    pub solver_id: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub attempt_limit: i32,
    pub title: String,
    pub prompt: String,
    pub instructions: String,
    pub verification_method: String,
    pub reward_lamports: i64,
    pub deadline: DateTime<Utc>,
    pub attempts: Vec<StoredAttempt>,
}

#[derive(Debug, Clone)]
pub struct StoredAttempt {
    pub attempt_number: i32,
    pub is_correct: bool,
    pub ineligibility_reason: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub reward_reserved: bool,
}

pub fn private_json(row: &OwnerBounty, expected_answer: &str) -> Value {
    let principal = principal_lamports(row.reward_lamports, row.max_rewarded_solvers);
    let commission = commission_lamports(principal);
    json!({
        "id": row.id,
        "title": row.title,
        "description": row.description,
        "instructions": row.instructions,
        "prompt": row.prompt,
        "category": row.category,
        "verificationMethod": row.verification_method,
        "expectedAnswer": expected_answer,
        "rewardLamports": row.reward_lamports.to_string(),
        "principalLamports": principal.to_string(),
        "commissionLamports": commission.to_string(),
        "maxRewardedSolvers": row.max_rewarded_solvers,
        "attemptLimit": row.attempt_limit,
        "deadline": row.deadline.to_rfc3339(),
        "status": row.status,
        "closeReason": row.close_reason,
    })
}

pub fn public_json(row: &PublicBountyRecord) -> Value {
    json!({
        "id": row.id,
        "title": row.title,
        "description": row.description,
        "instructions": row.instructions,
        "prompt": row.prompt,
        "category": row.category,
        "status": row.status,
        "rewardLamports": row.reward_lamports.to_string(),
        "maxRewardedSolvers": row.max_rewarded_solvers,
        "rewardedCount": row.rewarded_count,
        "attemptLimit": row.attempt_limit,
        "deadline": row.deadline.to_rfc3339(),
    })
}

pub fn session_view(session: &StoredSession) -> Value {
    let latest = session.attempts.last();
    json!({
        "id": session.id,
        "bountyId": session.bounty_id,
        "status": session.status,
        "startedAt": session.started_at.to_rfc3339(),
        "attemptLimit": session.attempt_limit,
        "attemptsUsed": session.attempts.len(),
        "remainingAttempts": (session.attempt_limit - session.attempts.len() as i32).max(0),
        "title": session.title,
        "prompt": session.prompt,
        "instructions": session.instructions,
        "verificationMethod": session.verification_method,
        "rewardLamports": session.reward_lamports.to_string(),
        "deadline": session.deadline.to_rfc3339(),
        "latest": latest.map(|item| json!({
            "attemptNumber": item.attempt_number,
            "isCorrect": item.is_correct,
            "ineligibilityReason": item.ineligibility_reason,
            "rewardStatus": if item.reward_reserved {
                Value::String("pending".into())
            } else {
                Value::Null
            },
        })),
    })
}

pub fn session_result(session: &StoredSession) -> Value {
    let latest = session.attempts.last();
    json!({
        "status": session.status,
        "attemptsUsed": session.attempts.len(),
        "remainingAttempts": (session.attempt_limit - session.attempts.len() as i32).max(0),
        "latest": latest.map(|item| json!({
            "attemptNumber": item.attempt_number,
            "isCorrect": item.is_correct,
            "ineligibilityReason": item.ineligibility_reason,
        })),
    })
}

pub fn funding_view(
    bounty: &OwnerBounty,
    obligations: &[StoredObligation],
    buckets: &LedgerBuckets,
    treasury: &str,
) -> Value {
    let pool = principal_lamports(bounty.reward_lamports, bounty.max_rewarded_solvers);
    let commission = commission_lamports(pool);
    let can_activate = obligations.iter().all(|o| o.status == "confirmed");
    json!({
        "bountyId": bounty.id,
        "status": bounty.status,
        "treasuryAddress": treasury,
        "network": "devnet",
        "custody": "custodial",
        "rewardLamports": bounty.reward_lamports.to_string(),
        "maxRewardedSolvers": bounty.max_rewarded_solvers,
        "rewardPoolLamports": pool.to_string(),
        "commissionLamports": commission.to_string(),
        "totalFundingLamports": (pool + commission).to_string(),
        "obligations": obligations.iter().map(|o| json!({
            "purpose": o.purpose,
            "expectedLamports": o.expected_lamports.to_string(),
            "observedLamports": o.observed_lamports.map(|v| v.to_string()),
            "depositReference": o.deposit_reference,
            "status": o.status,
            "signature": o.signature,
            "commitment": o.commitment,
        })).collect::<Vec<_>>(),
        "buckets": json!({
            "funded": buckets.funded.to_string(),
            "reserved": buckets.reserved.to_string(),
            "paid": buckets.paid.to_string(),
            "refundable": buckets.refundable.to_string(),
            "refunded": buckets.refunded.to_string(),
        }),
        "canActivate": can_activate,
    })
}

pub fn accuracy_text(correct: i32, total: i32) -> Option<String> {
    if total == 0 {
        return None;
    }
    let scaled = (correct as i64 * 10_000) / total as i64;
    Some(format!(
        "{}.{:04}",
        scaled / 10_000,
        (scaled % 10_000).abs()
    ))
}