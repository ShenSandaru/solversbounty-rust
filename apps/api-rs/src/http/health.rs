use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::config::APP_NAME;
use crate::db::DatabaseStatus;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

pub async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: APP_NAME,
    })
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    #[serde(rename = "checks")]
    pub checks: ReadyChecks,
}

#[derive(Serialize)]
pub struct ReadyChecks {
    pub database: DatabaseStatus,
}

pub async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    let (status_code, status) = match state.db_status {
        DatabaseStatus::Down => (StatusCode::SERVICE_UNAVAILABLE, "not_ready"),
        _ => (StatusCode::OK, "ready"),
    };
    (
        status_code,
        Json(ReadyResponse {
            status,
            checks: ReadyChecks {
                database: state.db_status,
            },
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::readiness;
    use crate::config::{AppConfig, NodeEnv};
    use crate::db::DatabaseStatus;
    use crate::AppState;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use std::sync::Arc;

    fn test_state(db: DatabaseStatus) -> AppState {
        AppState {
            config: Arc::new(AppConfig {
                node_env: NodeEnv::Test,
                port: 3001,
                database_url: None,
                cors_origins: vec!["http://localhost:3000".into()],
                json_body_limit_bytes: 65536,
                auth_token_secret: None,
                answer_encryption_key: None,
                treasury_secret_key: None,
                treasury_public_key: None,
                payout_broadcast_enabled: false,
                admin_wallet_allowlist: vec![],
                solana_rpc_url: None,
                login_domain: "localhost:3000".into(),
                login_uri: "http://localhost:3000".into(),
            }),
            pool: None,
            db_status: db,
            rate_limit: Arc::new(tokio::sync::Mutex::new(
                crate::modules::anti_abuse::InMemoryRateLimit::new(),
            )),
            payout_worker: Arc::new(crate::modules::rewards::payout_worker::PayoutWorker::disabled()),
        }
    }

    #[tokio::test]
    async fn ready_when_not_configured() {
        let response = readiness(State(test_state(DatabaseStatus::NotConfigured))).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn not_ready_when_down() {
        let response = readiness(State(test_state(DatabaseStatus::Down))).await.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}