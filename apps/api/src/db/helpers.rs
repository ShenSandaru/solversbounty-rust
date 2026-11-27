//! sqlx-oriented helpers aligned with Prisma `@@map` snake_case table names.
//! Handlers call these once `AppState::require_pool()` succeeds.

use sqlx::PgPool;
use uuid::Uuid;

use super::tables;

pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}

pub async fn find_user_by_wallet(
    pool: &PgPool,
    wallet_address: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE wallet_address = $1",
        table = tables::USERS
    ))
    .bind(wallet_address)
    .fetch_optional(pool)
    .await
}

pub async fn find_session_by_token_hash(
    pool: &PgPool,
    refresh_token_hash: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(&format!(
        "SELECT user_id FROM {table} WHERE refresh_token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()",
        table = tables::AUTH_SESSIONS
    ))
    .bind(refresh_token_hash)
    .fetch_optional(pool)
    .await
}