use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::http::error::{ApiError, ApiIssue, ApiResult};
use crate::modules::bounties::lamports::{
    parse_sol_to_lamports, MAX_PRINCIPAL_LAMPORTS, MAX_REWARDED_SOLVERS,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBountyInput {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub instructions: String,
    pub prompt: String,
    pub category: String,
    pub expected_answer: String,
    pub verification_method: String,
    pub reward_sol: String,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: String,
}

pub struct BountyWrite {
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub prompt: String,
    pub category: String,
    pub expected_answer_ciphertext: String,
    pub encryption_key_id: String,
    pub verification_method: String,
    pub reward_lamports: i64,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: DateTime<Utc>,
}

pub fn parse_create_input(
    input: CreateBountyInput,
    now: DateTime<Utc>,
    encrypt: impl FnOnce(&str) -> Result<String, crate::modules::bounties::answer_cipher::CipherError>,
) -> ApiResult<BountyWrite> {
    let mut issues = Vec::new();
    let title = input.title.trim();
    if title.is_empty() || title.len() > 140 {
        issues.push(ApiIssue {
            path: "title".into(),
            message: "Enter a title.".into(),
        });
    }
    let description = input.description.trim();
    if description.is_empty() || description.len() > 5000 {
        issues.push(ApiIssue {
            path: "description".into(),
            message: "Enter a description.".into(),
        });
    }
    let instructions = input.instructions.trim();
    if instructions.len() > 5000 {
        issues.push(ApiIssue {
            path: "instructions".into(),
            message: "Instructions are too long.".into(),
        });
    }
    let prompt = input.prompt.trim();
    if prompt.is_empty() || prompt.len() > 5000 {
        issues.push(ApiIssue {
            path: "prompt".into(),
            message: "Enter the challenge question.".into(),
        });
    }
    if !valid_category(&input.category) {
        issues.push(ApiIssue {
            path: "category".into(),
            message: "Choose a category.".into(),
        });
    }
    let expected = input.expected_answer.trim();
    if expected.is_empty() || expected.len() > 500 {
        issues.push(ApiIssue {
            path: "expectedAnswer".into(),
            message: "Enter the expected answer.".into(),
        });
    }
    if !valid_verification(&input.verification_method) {
        issues.push(ApiIssue {
            path: "verificationMethod".into(),
            message: "Choose a verification method.".into(),
        });
    }
    let reward_lamports = parse_sol_to_lamports(&input.reward_sol);
    if reward_lamports.unwrap_or(0) <= 0 {
        issues.push(ApiIssue {
            path: "rewardSol".into(),
            message: "Enter a positive SOL amount with at most 9 decimal places.".into(),
        });
    }
    if input.max_rewarded_solvers <= 0 {
        issues.push(ApiIssue {
            path: "maxRewardedSolvers".into(),
            message: "Enter at least one rewarded solver.".into(),
        });
    }
    if input.max_rewarded_solvers > MAX_REWARDED_SOLVERS {
        issues.push(ApiIssue {
            path: "maxRewardedSolvers".into(),
            message: "Enter a smaller solver cap.".into(),
        });
    }
    if input.attempt_limit <= 0 || input.attempt_limit > 10 {
        issues.push(ApiIssue {
            path: "attemptLimit".into(),
            message: "Enter at least one attempt.".into(),
        });
    }
    let deadline = DateTime::parse_from_rfc3339(&input.deadline)
        .map(|d| d.with_timezone(&Utc))
        .ok();
    if deadline.is_none() || deadline.unwrap() <= now {
        issues.push(ApiIssue {
            path: "deadline".into(),
            message: "Choose a deadline in the future. The server clock is authoritative later.".into(),
        });
    }
    if !issues.is_empty() {
        return Err(ApiError::validation("Validation failed.", issues));
    }
    let reward_lamports = reward_lamports.unwrap();
    let principal =
        crate::modules::bounties::lamports::principal_lamports(reward_lamports, input.max_rewarded_solvers);
    if principal <= 0 || principal > MAX_PRINCIPAL_LAMPORTS {
        issues.push(ApiIssue {
            path: "rewardSol".into(),
            message: "The funding total exceeds the allowed cap.".into(),
        });
        return Err(ApiError::validation("Validation failed.", issues));
    }
    let ciphertext = encrypt(expected).map_err(|e| ApiError {
        status: axum::http::StatusCode::from_u16(e.status).unwrap_or(axum::http::StatusCode::SERVICE_UNAVAILABLE),
        code: match e.code {
            "NOT_READY" => crate::http::error::ApiErrorCode::NotReady,
            _ => crate::http::error::ApiErrorCode::Internal,
        },
        message: e.message,
        issues: None,
        checks: None,
    })?;
    Ok(BountyWrite {
        title: title.to_string(),
        description: description.to_string(),
        instructions: instructions.to_string(),
        prompt: prompt.to_string(),
        category: input.category,
        expected_answer_ciphertext: ciphertext,
        encryption_key_id: crate::modules::bounties::answer_cipher::ANSWER_KEY_ID.into(),
        verification_method: input.verification_method,
        reward_lamports,
        max_rewarded_solvers: input.max_rewarded_solvers,
        attempt_limit: input.attempt_limit,
        deadline: deadline.unwrap(),
    })
}

fn valid_category(value: &str) -> bool {
    matches!(
        value,
        "logic" | "mathematics" | "numerical_patterns" | "ciphers"
    )
}

fn valid_verification(value: &str) -> bool {
    matches!(
        value,
        "exact" | "case_insensitive" | "normalized_text" | "numeric_exact"
    )
}

pub const RESEARCH_CONSENT_VERSION: &str = "research-consent-v1";