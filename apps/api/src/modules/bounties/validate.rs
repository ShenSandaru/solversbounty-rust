use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::http::error::{ApiError, ApiIssue, ApiResult};
use crate::modules::bounties::lamports::{
    parse_sol_to_lamports, MAX_PRINCIPAL_LAMPORTS, MAX_REWARDED_SOLVERS,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionInput {
    pub id: Option<String>,
    pub order_index: Option<i32>,
    pub question_type: Option<String>,
    pub prompt: String,
    pub options: Option<Vec<String>>,
    pub expected_answer: String,
    pub verification_method: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBountyInput {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub instructions: String,
    pub prompt: Option<String>,
    pub category: String,
    pub expected_answer: Option<String>,
    pub verification_method: Option<String>,
    pub questions: Option<Vec<QuestionInput>>,
    pub reward_sol: String,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: String,
}

#[derive(Debug, Clone)]
pub struct QuestionWrite {
    pub id: Option<uuid::Uuid>,
    pub order_index: i32,
    pub question_type: String,
    pub prompt: String,
    pub options: Option<serde_json::Value>,
    pub expected_answer_ciphertext: String,
    pub encryption_key_id: String,
    pub verification_method: String,
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
    pub questions: Vec<QuestionWrite>,
    pub reward_lamports: i64,
    pub max_rewarded_solvers: i32,
    pub attempt_limit: i32,
    pub deadline: DateTime<Utc>,
}

pub fn parse_create_input(
    input: CreateBountyInput,
    now: DateTime<Utc>,
    mut encrypt: impl FnMut(
        &str,
    ) -> Result<String, crate::modules::bounties::answer_cipher::CipherError>,
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
    let raw_questions: Vec<QuestionInput> =
        if let Some(qs) = input.questions.filter(|q| !q.is_empty()) {
            qs
        } else if let (Some(p), Some(e), Some(v)) = (
            input.prompt.as_deref(),
            input.expected_answer.as_deref(),
            input.verification_method.as_deref(),
        ) {
            vec![QuestionInput {
                id: None,
                order_index: Some(0),
                question_type: Some("short_answer".into()),
                prompt: p.to_string(),
                options: None,
                expected_answer: e.to_string(),
                verification_method: v.to_string(),
            }]
        } else {
            issues.push(ApiIssue {
                path: "questions".into(),
                message: "Provide at least one question.".into(),
            });
            Vec::new()
        };

    if !valid_category(&input.category) {
        issues.push(ApiIssue {
            path: "category".into(),
            message: "Choose a category.".into(),
        });
    }

    let mut seen_order = std::collections::HashSet::new();
    for (idx, q) in raw_questions.iter().enumerate() {
        if let Some(raw_id) = q.id.as_deref() {
            if uuid::Uuid::parse_str(raw_id).is_err() {
                issues.push(ApiIssue {
                    path: format!("questions.{idx}.id"),
                    message: "Invalid question id.".into(),
                });
            }
        }
        let order_idx = q.order_index.unwrap_or(idx as i32);
        if order_idx < 0 || !seen_order.insert(order_idx) {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.orderIndex"),
                message: "Question order indexes must be unique non-negative integers.".into(),
            });
        }
        let q_type = q.question_type.as_deref().unwrap_or("short_answer");
        if !valid_question_type(q_type) {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.questionType"),
                message: "Choose a valid question type.".into(),
            });
        }
        let q_prompt = q.prompt.trim();
        if q_prompt.is_empty() || q_prompt.len() > 5000 {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.prompt"),
                message: "Enter the challenge question.".into(),
            });
        }
        let q_expected = q.expected_answer.trim();
        if q_expected.is_empty() || q_expected.len() > 500 {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.expectedAnswer"),
                message: "Enter the expected answer.".into(),
            });
        }
        if !valid_verification(&q.verification_method) {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.verificationMethod"),
                message: "Choose a verification method.".into(),
            });
        }
        if q_type == "multiple_choice" {
            let opts: Vec<String> = q
                .options
                .as_ref()
                .map(|list| {
                    list.iter()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            if opts.len() < 2 {
                issues.push(ApiIssue {
                    path: format!("questions.{idx}.options"),
                    message: "Multiple-choice questions require at least 2 options.".into(),
                });
            } else {
                let unique: std::collections::HashSet<&String> = opts.iter().collect();
                if unique.len() != opts.len() {
                    issues.push(ApiIssue {
                        path: format!("questions.{idx}.options"),
                        message: "Multiple-choice options must be distinct.".into(),
                    });
                }
                if !opts.iter().any(|o| o == q_expected) {
                    issues.push(ApiIssue {
                        path: format!("questions.{idx}.expectedAnswer"),
                        message: "Expected answer must match one of the multiple-choice options."
                            .into(),
                    });
                }
            }
        } else if q.options.as_ref().is_some_and(|opts| !opts.is_empty()) {
            issues.push(ApiIssue {
                path: format!("questions.{idx}.options"),
                message: "Options are only allowed for multiple-choice questions.".into(),
            });
        }
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
            message: "Choose a deadline in the future. The server clock is authoritative later."
                .into(),
        });
    }
    if !issues.is_empty() {
        return Err(ApiError::validation("Validation failed.", issues));
    }
    let reward_lamports = reward_lamports.unwrap();
    let principal = crate::modules::bounties::lamports::principal_lamports(
        reward_lamports,
        input.max_rewarded_solvers,
    );
    if principal <= 0 || principal > MAX_PRINCIPAL_LAMPORTS {
        issues.push(ApiIssue {
            path: "rewardSol".into(),
            message: "The funding total exceeds the allowed cap.".into(),
        });
        return Err(ApiError::validation("Validation failed.", issues));
    }
    let mut questions = Vec::with_capacity(raw_questions.len());
    for (idx, q) in raw_questions.into_iter().enumerate() {
        let q_expected = q.expected_answer.trim();
        let ciphertext = encrypt(q_expected).map_err(|e| ApiError {
            status: axum::http::StatusCode::from_u16(e.status)
                .unwrap_or(axum::http::StatusCode::SERVICE_UNAVAILABLE),
            code: match e.code {
                "NOT_READY" => crate::http::error::ApiErrorCode::NotReady,
                _ => crate::http::error::ApiErrorCode::Internal,
            },
            message: e.message.clone(),
            issues: None,
            checks: None,
        })?;
        let q_type = q.question_type.unwrap_or_else(|| "short_answer".into());
        let options = if q_type == "multiple_choice" {
            q.options.map(|list| {
                serde_json::Value::Array(
                    list.into_iter()
                        .map(|s| serde_json::Value::String(s.trim().to_string()))
                        .collect(),
                )
            })
        } else {
            None
        };
        let parsed_id =
            q.id.as_deref()
                .and_then(|raw| uuid::Uuid::parse_str(raw).ok());
        questions.push(QuestionWrite {
            id: parsed_id,
            order_index: q.order_index.unwrap_or(idx as i32),
            question_type: q_type,
            prompt: q.prompt.trim().to_string(),
            options,
            expected_answer_ciphertext: ciphertext,
            encryption_key_id: crate::modules::bounties::answer_cipher::ANSWER_KEY_ID.into(),
            verification_method: q.verification_method,
        });
    }
    let first = questions.first().cloned().ok_or_else(ApiError::internal)?;
    Ok(BountyWrite {
        title: title.to_string(),
        description: description.to_string(),
        instructions: instructions.to_string(),
        prompt: first.prompt,
        category: input.category,
        expected_answer_ciphertext: first.expected_answer_ciphertext,
        encryption_key_id: first.encryption_key_id,
        verification_method: first.verification_method,
        questions,
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

fn valid_question_type(value: &str) -> bool {
    matches!(
        value,
        "multiple_choice"
            | "short_answer"
            | "logic_puzzle"
            | "mathematics"
            | "numerical_pattern"
            | "cipher"
    )
}

fn valid_verification(value: &str) -> bool {
    matches!(
        value,
        "exact" | "case_insensitive" | "normalized_text" | "numeric_exact"
    )
}

pub const RESEARCH_CONSENT_VERSION: &str = "research-consent-v1";

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn base_input() -> CreateBountyInput {
        CreateBountyInput {
            title: "Test Bounty".into(),
            description: "Description".into(),
            instructions: "".into(),
            prompt: None,
            category: "logic".into(),
            expected_answer: None,
            verification_method: None,
            questions: None,
            reward_sol: "0.1".into(),
            max_rewarded_solvers: 5,
            attempt_limit: 3,
            deadline: (Utc::now() + Duration::days(7)).to_rfc3339(),
        }
    }

    #[test]
    fn accepts_legacy_single_question() {
        let mut input = base_input();
        input.prompt = Some("What is 2+2?".into());
        input.expected_answer = Some("4".into());
        input.verification_method = Some("numeric_exact".into());
        let parsed = parse_create_input(input, Utc::now(), |s| Ok(format!("enc:{s}"))).unwrap();
        assert_eq!(parsed.questions.len(), 1);
        assert_eq!(parsed.questions[0].order_index, 0);
        assert_eq!(parsed.questions[0].expected_answer_ciphertext, "enc:4");
    }

    #[test]
    fn accepts_multi_question_with_mcq_and_preserves_order() {
        let mut input = base_input();
        input.questions = Some(vec![
            QuestionInput {
                id: None,
                order_index: Some(0),
                question_type: Some("multiple_choice".into()),
                prompt: "Pick 4".into(),
                options: Some(vec!["3".into(), "4".into()]),
                expected_answer: "4".into(),
                verification_method: "exact".into(),
            },
            QuestionInput {
                id: None,
                order_index: Some(1),
                question_type: Some("short_answer".into()),
                prompt: "Capital?".into(),
                options: None,
                expected_answer: "Paris".into(),
                verification_method: "case_insensitive".into(),
            },
        ]);
        let parsed = parse_create_input(input, Utc::now(), |s| Ok(format!("enc:{s}"))).unwrap();
        assert_eq!(parsed.questions.len(), 2);
        assert_eq!(parsed.questions[0].order_index, 0);
        assert_eq!(parsed.questions[1].order_index, 1);
    }

    #[test]
    fn rejects_invalid_mcq_configs() {
        let mut one_opt = base_input();
        one_opt.questions = Some(vec![QuestionInput {
            id: None,
            order_index: None,
            question_type: Some("multiple_choice".into()),
            prompt: "Q".into(),
            options: Some(vec!["A".into()]),
            expected_answer: "A".into(),
            verification_method: "exact".into(),
        }]);
        assert!(parse_create_input(one_opt, Utc::now(), |s| Ok(s.into())).is_err());

        let mut bad_ans = base_input();
        bad_ans.questions = Some(vec![QuestionInput {
            id: None,
            order_index: None,
            question_type: Some("multiple_choice".into()),
            prompt: "Q".into(),
            options: Some(vec!["A".into(), "B".into()]),
            expected_answer: "C".into(),
            verification_method: "exact".into(),
        }]);
        assert!(parse_create_input(bad_ans, Utc::now(), |s| Ok(s.into())).is_err());
    }

    #[test]
    fn rejects_invalid_question_id_and_duplicate_order_index() {
        let mut bad_id = base_input();
        bad_id.questions = Some(vec![QuestionInput {
            id: Some("not-a-uuid".into()),
            order_index: Some(0),
            question_type: Some("short_answer".into()),
            prompt: "Q1".into(),
            options: None,
            expected_answer: "A1".into(),
            verification_method: "exact".into(),
        }]);
        assert!(parse_create_input(bad_id, Utc::now(), |s| Ok(s.into())).is_err());

        let mut dup_order = base_input();
        dup_order.questions = Some(vec![
            QuestionInput {
                id: None,
                order_index: Some(0),
                question_type: Some("short_answer".into()),
                prompt: "Q1".into(),
                options: None,
                expected_answer: "A1".into(),
                verification_method: "exact".into(),
            },
            QuestionInput {
                id: None,
                order_index: Some(0),
                question_type: Some("short_answer".into()),
                prompt: "Q2".into(),
                options: None,
                expected_answer: "A2".into(),
                verification_method: "exact".into(),
            },
        ]);
        assert!(parse_create_input(dup_order, Utc::now(), |s| Ok(s.into())).is_err());
    }
}
