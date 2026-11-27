use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::db::tables;

pub struct StoredNonce {
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthUser {
    pub id: Uuid,
    pub wallet_address: String,
    pub role: String,
}

pub async fn issue_nonce(
    pool: &PgPool,
    user_wallet: &str,
    nonce: &str,
    expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(&format!(
        "UPDATE {table} SET retired_at = $1 WHERE user_wallet = $2 AND used_at IS NULL AND retired_at IS NULL",
        table = tables::AUTH_NONCES
    ))
    .bind(now)
    .bind(user_wallet)
    .execute(&mut *tx)
    .await?;
    let nonce_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, user_wallet, nonce, expires_at) VALUES ($1, $2, $3, $4)",
        table = tables::AUTH_NONCES
    ))
    .bind(nonce_id)
    .bind(user_wallet)
    .bind(nonce)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub async fn find_active_nonce(
    pool: &PgPool,
    user_wallet: &str,
    now: DateTime<Utc>,
) -> Result<Option<StoredNonce>, sqlx::Error> {
    let row: Option<(String, DateTime<Utc>)> = sqlx::query_as(&format!(
        "SELECT nonce, expires_at FROM {table}
         WHERE user_wallet = $1 AND used_at IS NULL AND retired_at IS NULL AND expires_at > $2
         ORDER BY expires_at DESC LIMIT 1",
        table = tables::AUTH_NONCES
    ))
    .bind(user_wallet)
    .bind(now)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(nonce, expires_at)| StoredNonce { nonce, expires_at }))
}

pub async fn consume_login(
    pool: &PgPool,
    user_wallet: &str,
    nonce: &str,
    role: &str,
    refresh_token_hash: &str,
    session_expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Option<AuthUser>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let saved = consume_login_tx(
        &mut tx,
        user_wallet,
        nonce,
        role,
        refresh_token_hash,
        session_expires_at,
        now,
    )
    .await?;
    if saved.is_some() {
        tx.commit().await?;
    }
    Ok(saved)
}

async fn consume_login_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_wallet: &str,
    nonce: &str,
    role: &str,
    refresh_token_hash: &str,
    session_expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Option<AuthUser>, sqlx::Error> {
    let row: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(&format!(
        "SELECT id, expires_at FROM {table}
         WHERE user_wallet = $1 AND nonce = $2 AND used_at IS NULL AND retired_at IS NULL",
        table = tables::AUTH_NONCES
    ))
    .bind(user_wallet)
    .bind(nonce)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((nonce_id, expires_at)) = row else {
        return Ok(None);
    };
    if expires_at <= now {
        return Ok(None);
    }
    let updated = sqlx::query(&format!(
        "UPDATE {table} SET used_at = $1 WHERE id = $2 AND used_at IS NULL AND retired_at IS NULL",
        table = tables::AUTH_NONCES
    ))
    .bind(now)
    .bind(nonce_id)
    .execute(&mut **tx)
    .await?;
    if updated.rows_affected() != 1 {
        return Ok(None);
    }
    let user = upsert_user(tx, user_wallet, role, now).await?;
    sqlx::query(&format!(
        "UPDATE {table} SET revoked_at = $1 WHERE user_id = $2 AND revoked_at IS NULL",
        table = tables::AUTH_SESSIONS
    ))
    .bind(now)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;
    let session_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3, $4)",
        table = tables::AUTH_SESSIONS
    ))
    .bind(session_id)
    .bind(user.id)
    .bind(refresh_token_hash)
    .bind(session_expires_at)
    .execute(&mut **tx)
    .await?;
    Ok(Some(user))
}

async fn upsert_user(
    tx: &mut Transaction<'_, Postgres>,
    wallet_address: &str,
    role: &str,
    now: DateTime<Utc>,
) -> Result<AuthUser, sqlx::Error> {
    let existing: Option<(Uuid, String)> = sqlx::query_as(&format!(
        "SELECT id, role::text FROM {table} WHERE wallet_address = $1",
        table = tables::USERS
    ))
    .bind(wallet_address)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some((id, _)) = existing {
        sqlx::query(&format!(
            "UPDATE {table} SET role = $1::user_role WHERE id = $2",
            table = tables::USERS
        ))
        .bind(role)
        .bind(id)
        .execute(&mut **tx)
        .await?;
        return Ok(AuthUser {
            id,
            wallet_address: wallet_address.to_string(),
            role: role.to_string(),
        });
    }
    let id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, wallet_address, role, updated_at) VALUES ($1, $2, $3::user_role, $4)",
        table = tables::USERS
    ))
    .bind(id)
    .bind(wallet_address)
    .bind(role)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(AuthUser {
        id,
        wallet_address: wallet_address.to_string(),
        role: role.to_string(),
    })
}

pub async fn find_active_session(
    pool: &PgPool,
    refresh_token_hash: &str,
    now: DateTime<Utc>,
) -> Result<Option<AuthUser>, sqlx::Error> {
    let row: Option<(Uuid, String, String)> = sqlx::query_as(&format!(
        "SELECT u.id, u.wallet_address, u.role::text FROM {sessions} s
         JOIN {users} u ON u.id = s.user_id
         WHERE s.refresh_token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > $2",
        sessions = tables::AUTH_SESSIONS,
        users = tables::USERS
    ))
    .bind(refresh_token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id, wallet_address, role)| AuthUser {
        id,
        wallet_address,
        role,
    }))
}

pub async fn revoke_session(
    pool: &PgPool,
    refresh_token_hash: &str,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(&format!(
        "UPDATE {table} SET revoked_at = $1 WHERE refresh_token_hash = $2 AND revoked_at IS NULL",
        table = tables::AUTH_SESSIONS
    ))
    .bind(now)
    .bind(refresh_token_hash)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}
