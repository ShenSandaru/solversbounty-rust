use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;

use crate::http::{health, middleware as mw};
use crate::modules::auth::handlers as auth;
use crate::modules::bounties::handlers as bounties;
use crate::modules::rewards::handlers as rewards;
use crate::modules::submissions::handlers as submissions;
use crate::AppState;

pub fn router(state: AppState) -> Router {
    let limit = state.config.json_body_limit_bytes;
    let cors = mw::cors_layer(&state.config.cors_origins);

    Router::new()
        .route("/health", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/auth/nonce", post(auth::issue_nonce))
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::session))
        .route("/auth/logout", post(auth::logout))
        .route("/bounties", post(bounties::create_bounty).get(bounties::list_bounties))
        .route("/bounties/mine", get(bounties::mine_bounties))
        .route("/bounties/{id}/public", get(bounties::public_bounty))
        .route("/bounties/{id}", get(bounties::private_bounty).patch(bounties::patch_bounty))
        .route("/bounties/{id}/funding", get(bounties::funding_view))
        .route(
            "/bounties/{id}/funding/{purpose}",
            post(bounties::submit_funding),
        )
        .route("/bounties/{id}/activate", post(bounties::activate_bounty))
        .route("/bounties/{id}/consent", post(submissions::consent))
        .route("/bounties/{id}/sessions", post(submissions::start_session))
        .route("/bounties/{id}/analytics", get(bounties::bounty_analytics))
        .route("/sessions/{id}", get(submissions::read_session))
        .route("/sessions/{id}/result", get(submissions::session_result))
        .route("/sessions/{id}/submissions", post(submissions::submit_answer))
        .route("/payouts/mine", get(rewards::payouts_mine))
        .route("/bounties/{id}/payouts", get(rewards::bounty_payouts))
        .route("/admin/payouts", get(rewards::admin_payouts))
        .route("/admin/payouts/run", post(rewards::admin_run_payouts))
        .layer(RequestBodyLimitLayer::new(limit))
        .layer(cors)
        .layer(middleware::from_fn(mw::request_id))
        .with_state(state)
}