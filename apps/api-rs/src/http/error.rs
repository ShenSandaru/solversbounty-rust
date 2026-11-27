use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    ValidationError,
    Unauthenticated,
    AuthRejected,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    NotReady,
    Internal,
}

impl ApiErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::AuthRejected => "AUTH_REJECTED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::RateLimited => "RATE_LIMITED",
            Self::NotReady => "NOT_READY",
            Self::Internal => "INTERNAL",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub status_code: u16,
    pub code: &'static str,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<ApiIssue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ApiErrorCode,
    pub message: String,
    pub issues: Option<Vec<ApiIssue>>,
    pub checks: Option<std::collections::HashMap<String, String>>,
}

impl ApiError {
    pub fn not_ready(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: ApiErrorCode::NotReady,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: ApiErrorCode::Unauthenticated,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: ApiErrorCode::Forbidden,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn validation(message: impl Into<String>, issues: Vec<ApiIssue>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ApiErrorCode::ValidationError,
            message: message.into(),
            issues: Some(issues),
            checks: None,
        }
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: ApiErrorCode::RateLimited,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn auth_rejected(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: ApiErrorCode::AuthRejected,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: ApiErrorCode::NotFound,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: ApiErrorCode::Conflict,
            message: message.into(),
            issues: None,
            checks: None,
        }
    }

    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ApiErrorCode::Internal,
            message: "Internal server error".into(),
            issues: None,
            checks: None,
        }
    }

    pub fn body(&self, request_id: &str) -> ApiErrorBody {
        ApiErrorBody {
            status_code: self.status.as_u16(),
            code: self.code.as_str(),
            message: self.message.clone(),
            request_id: request_id.to_string(),
            issues: self.issues.clone(),
            checks: self.checks.clone(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = crate::http::middleware::current_request_id();
        let status = self.status;
        let body = self.body(&request_id);
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
