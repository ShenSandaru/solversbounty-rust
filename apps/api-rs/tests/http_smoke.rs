use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use solversbounty_api::config::{AppConfig, NodeEnv};
use solversbounty_api::db::DatabaseStatus;
use solversbounty_api::http::routes;
use solversbounty_api::modules::anti_abuse::InMemoryRateLimit;
use solversbounty_api::modules::rewards::payout_worker::PayoutWorker;
use solversbounty_api::AppState;
use std::sync::Arc;
use tower::ServiceExt;

fn test_state() -> AppState {
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
        db_status: DatabaseStatus::NotConfigured,
        rate_limit: Arc::new(tokio::sync::Mutex::new(InMemoryRateLimit::new())),
        payout_worker: Arc::new(PayoutWorker::disabled()),
    }
}

#[tokio::test]
async fn health_ok() {
    let app = routes::router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_nonce_not_ready_without_db() {
    let app = routes::router(test_state());
    let body = r#"{"walletAddress":"11111111111111111111111111111111"}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/nonce")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], "NOT_READY");
    assert!(json.get("requestId").is_some());
}

#[tokio::test]
async fn cors_allows_configured_origin() {
    let app = routes::router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "http://localhost:3000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-credentials")
            .unwrap(),
        "true"
    );
}