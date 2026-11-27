use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::error;

use crate::config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatus {
    NotConfigured,
    Up,
    Down,
}

pub async fn connect_optional(config: &AppConfig) -> (Option<PgPool>, DatabaseStatus) {
    let Some(url) = config.database_url.as_ref() else {
        return (None, DatabaseStatus::NotConfigured);
    };
    match PgPoolOptions::new().max_connections(5).connect(url).await {
        Ok(pool) => {
            if sqlx::query("SELECT 1").execute(&pool).await.is_err() {
                error!("Database ping failed");
                return (None, DatabaseStatus::Down);
            }
            (Some(pool), DatabaseStatus::Up)
        }
        Err(err) => {
            error!("Database connection failed: {}", err);
            (None, DatabaseStatus::Down)
        }
    }
}

pub mod helpers;
pub mod tables {
    //! SQL table names (snake_case), aligned with Prisma @@map names.

    pub const USERS: &str = "users";
    pub const AUTH_NONCES: &str = "auth_nonces";
    pub const AUTH_SESSIONS: &str = "auth_sessions";
    pub const BOUNTIES: &str = "bounties";
    pub const BOUNTY_QUESTIONS: &str = "bounty_questions";
    pub const BOUNTY_FUNDINGS: &str = "bounty_fundings";
    pub const BOUNTY_LEDGER_ENTRIES: &str = "bounty_ledger_entries";
    pub const CONSENT_RECORDS: &str = "consent_records";
    pub const SOLVING_SESSIONS: &str = "solving_sessions";
    pub const SUBMISSION_ATTEMPTS: &str = "submission_attempts";
    pub const REWARD_ALLOCATIONS: &str = "reward_allocations";
    pub const PAYOUTS: &str = "payouts";
    pub const CHAIN_TRANSACTIONS: &str = "chain_transactions";
    pub const AUDIT_LOGS: &str = "audit_logs";
}
