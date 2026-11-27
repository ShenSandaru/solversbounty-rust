pub mod config;
pub mod db;
pub mod http;
pub mod modules;

use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::db::DatabaseStatus;
use crate::modules::anti_abuse::InMemoryRateLimit;
use crate::modules::rewards::payout_worker::PayoutWorker;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: Option<PgPool>,
    pub db_status: DatabaseStatus,
    pub rate_limit: Arc<Mutex<InMemoryRateLimit>>,
    pub payout_worker: Arc<PayoutWorker>,
}

impl AppState {
    pub fn require_pool(&self) -> Result<&PgPool, http::error::ApiError> {
        self.pool
            .as_ref()
            .ok_or_else(|| http::error::ApiError::not_ready("Database is not configured."))
    }
}

pub async fn build_state(config: AppConfig) -> AppState {
    let config = Arc::new(config);
    let (pool, db_status) = db::connect_optional(&config).await;
    let mut worker = PayoutWorker::new(pool.clone(), Arc::clone(&config));
    if let Some(url) = config.solana_rpc_url.clone() {
        if let Some(gateway) = modules::solana::RpcSolanaGateway::new(url) {
            worker = worker.with_gateway(Arc::new(gateway));
        }
    }
    let payout_worker = Arc::new(worker);
    if pool.is_some() {
        payout_worker.clone().start();
    }
    AppState {
        config,
        pool,
        db_status,
        rate_limit: Arc::new(Mutex::new(InMemoryRateLimit::new())),
        payout_worker,
    }
}
