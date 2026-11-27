use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, Transaction};
use uuid::Uuid;

use crate::db::tables;
use crate::modules::bounties::funding_ledger::{buckets_from_entries, LedgerEntry};

pub async fn list_actionable(pool: &PgPool, limit: i64) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE status IN ('pending'::payout_status, 'submitted'::payout_status, 'retryable'::payout_status)
         ORDER BY created_at ASC LIMIT $1",
        table = tables::PAYOUTS
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct PayoutSnapshot {
    pub id: Uuid,
    pub status: String,
    pub amount_lamports: i64,
    pub bounty_id: Uuid,
    pub solver_wallet: String,
    pub reserved_lamports: i64,
    pub settled: bool,
    pub signatures: Vec<ChainSig>,
}

#[derive(Debug, Clone)]
pub struct ChainSig {
    pub signature: String,
    pub commitment: String,
}

pub async fn read_payout(pool: &PgPool, id: Uuid) -> Result<Option<PayoutSnapshot>, sqlx::Error> {
    let row = sqlx::query(&format!(
        "SELECT p.id, p.status::text, a.amount_lamports, a.bounty_id, u.wallet_address
         FROM {payouts} p
         JOIN {allocations} a ON a.id = p.allocation_id
         JOIN {users} u ON u.id = a.solver_id
         WHERE p.id = $1",
        payouts = tables::PAYOUTS,
        allocations = tables::REWARD_ALLOCATIONS,
        users = tables::USERS
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let bounty_id: Uuid = row.get("bounty_id");
    let signatures = load_chain_sigs(pool, id).await?;
    let ledger = load_ledger_for_payout(pool, id, bounty_id).await?;
    let buckets = buckets_from_entries(&ledger);
    let settled = ledger.iter().any(|e| e.entry_type == "settle");
    Ok(Some(PayoutSnapshot {
        id: row.get("id"),
        status: row.get("status"),
        amount_lamports: row.get("amount_lamports"),
        bounty_id,
        solver_wallet: row.get("wallet_address"),
        reserved_lamports: buckets.reserved,
        settled,
        signatures,
    }))
}

pub enum RecordSendResult {
    Recorded,
    Blocked,
}

pub async fn record_send(
    pool: &PgPool,
    id: Uuid,
    signature: &str,
    source: &str,
    destination: &str,
    amount_lamports: i64,
) -> Result<RecordSendResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_payout(&mut tx, id).await?;
    let snapshot = read_payout_tx(&mut tx, id).await?;
    let Some(current) = snapshot else {
        return Ok(RecordSendResult::Blocked);
    };
    let terminal = !current.signatures.is_empty()
        && current.signatures.iter().all(|s| s.commitment == "failed");
    let open = (current.status == "pending" && current.signatures.is_empty())
        || (current.status == "retryable" && terminal);
    if !open || current.amount_lamports != amount_lamports {
        return Ok(RecordSendResult::Blocked);
    }
    let chain_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, signature, kind, amount_lamports, source_address, destination_address, commitment, payout_id)
         VALUES ($1, $2, 'payout'::chain_transaction_kind, $3, $4, $5, 'submitted', $6)",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(chain_id)
    .bind(signature)
    .bind(amount_lamports)
    .bind(source)
    .bind(destination)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'submitted'::payout_status, send_count = send_count + 1, last_error = NULL WHERE id = $1",
        table = tables::PAYOUTS
    ))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(RecordSendResult::Recorded)
}

pub async fn note_retryable(
    pool: &PgPool,
    id: Uuid,
    signature: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_payout(&mut tx, id).await?;
    sqlx::query(&format!(
        "UPDATE {table} SET commitment = 'failed' WHERE signature = $1",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(signature)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'retryable'::payout_status, last_error = $1 WHERE id = $2",
        table = tables::PAYOUTS
    ))
    .bind("The transfer failed on Devnet.")
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub enum SettleResult {
    Settled,
    Blocked,
}

pub async fn settle(
    pool: &PgPool,
    id: Uuid,
    signature: &str,
    commitment: &str,
    now: DateTime<Utc>,
) -> Result<SettleResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_payout(&mut tx, id).await?;
    let snapshot = read_payout_tx(&mut tx, id).await?;
    let Some(current) = snapshot else {
        return Ok(SettleResult::Blocked);
    };
    if current.status == "confirmed" {
        return Ok(SettleResult::Settled);
    }
    let chain_id: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE signature = $1",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(signature)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(chain_id) = chain_id else {
        return Ok(SettleResult::Blocked);
    };
    let settled_exists: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE chain_transaction_id = $1 AND entry_type = 'settle'::ledger_entry_type LIMIT 1",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(chain_id)
    .fetch_optional(&mut *tx)
    .await?;
    if settled_exists.is_none() {
        let ledger_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, bounty_id, entry_type, amount_lamports, payout_id, chain_transaction_id)
             VALUES ($1, $2, 'settle'::ledger_entry_type, $3, $4, $5)",
            table = tables::BOUNTY_LEDGER_ENTRIES
        ))
        .bind(ledger_id)
        .bind(current.bounty_id)
        .bind(current.amount_lamports)
        .bind(id)
        .bind(chain_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(&format!(
        "UPDATE {table} SET commitment = $1, confirmed_at = $2 WHERE signature = $3",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(commitment)
    .bind(now)
    .bind(signature)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE {allocations} SET status = 'paid'::allocation_status
         WHERE id = (SELECT allocation_id FROM {payouts} WHERE id = $1)",
        allocations = tables::REWARD_ALLOCATIONS,
        payouts = tables::PAYOUTS
    ))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'confirmed'::payout_status, last_error = NULL WHERE id = $1",
        table = tables::PAYOUTS
    ))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(SettleResult::Settled)
}

pub async fn note_last_error(pool: &PgPool, id: Uuid, message: &str) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "UPDATE {table} SET last_error = $1 WHERE id = $2",
        table = tables::PAYOUTS
    ))
    .bind(message)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn solver_payouts(pool: &PgPool, solver_id: Uuid) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT p.id, a.bounty_id, a.amount_lamports, p.status::text, c.signature
         FROM {payouts} p
         JOIN {allocations} a ON a.id = p.allocation_id
         LEFT JOIN {chain} c ON c.payout_id = p.id AND p.status = 'confirmed'::payout_status
         WHERE a.solver_id = $1
         ORDER BY p.created_at DESC",
        payouts = tables::PAYOUTS,
        allocations = tables::REWARD_ALLOCATIONS,
        chain = tables::CHAIN_TRANSACTIONS
    ))
    .bind(solver_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "bountyId": r.get::<Uuid, _>("bounty_id").to_string(),
                "amountLamports": r.get::<i64, _>("amount_lamports").to_string(),
                "status": r.get::<String, _>("status"),
                "signature": r.get::<Option<String>, _>("signature"),
            })
        })
        .collect())
}

pub async fn creator_summary(
    pool: &PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let owned: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE id = $1 AND creator_id = $2",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .bind(creator_id)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Ok(None);
    }
    let rows = sqlx::query(&format!(
        "SELECT a.amount_lamports, p.status::text FROM {allocations} a
         LEFT JOIN {payouts} p ON p.allocation_id = a.id WHERE a.bounty_id = $1",
        allocations = tables::REWARD_ALLOCATIONS,
        payouts = tables::PAYOUTS
    ))
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;
    let mut pending_c = 0i32;
    let mut submitted_c = 0i32;
    let mut confirmed_c = 0i32;
    let mut failed_c = 0i32;
    let mut retryable_c = 0i32;
    let mut pending_l = 0i64;
    let mut submitted_l = 0i64;
    let mut confirmed_l = 0i64;
    let mut failed_l = 0i64;
    let mut retryable_l = 0i64;
    for row in rows {
        let status: Option<String> = row.get("status");
        let amount: i64 = row.get("amount_lamports");
        match status.as_deref() {
            Some("pending") => {
                pending_c += 1;
                pending_l += amount;
            }
            Some("submitted") => {
                submitted_c += 1;
                submitted_l += amount;
            }
            Some("confirmed") => {
                confirmed_c += 1;
                confirmed_l += amount;
            }
            Some("failed") => {
                failed_c += 1;
                failed_l += amount;
            }
            Some("retryable") => {
                retryable_c += 1;
                retryable_l += amount;
            }
            _ => {}
        }
    }
    let ledger_rows = sqlx::query(&format!(
        "SELECT entry_type::text, amount_lamports FROM {table} WHERE bounty_id = $1",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;
    let entries: Vec<LedgerEntry> = ledger_rows
        .iter()
        .map(|r| LedgerEntry {
            entry_type: r.get("entry_type"),
            amount_lamports: r.get("amount_lamports"),
        })
        .collect();
    let buckets = buckets_from_entries(&entries);
    Ok(Some(serde_json::json!({
        "bountyId": bounty_id.to_string(),
        "counts": {
            "pending": pending_c,
            "submitted": submitted_c,
            "confirmed": confirmed_c,
            "failed": failed_c,
            "retryable": retryable_c,
        },
        "lamportsByStatus": {
            "pending": pending_l.to_string(),
            "submitted": submitted_l.to_string(),
            "confirmed": confirmed_l.to_string(),
            "failed": failed_l.to_string(),
            "retryable": retryable_l.to_string(),
        },
        "buckets": {
            "funded": buckets.funded.to_string(),
            "reserved": buckets.reserved.to_string(),
            "paid": buckets.paid.to_string(),
            "refundable": buckets.refundable.to_string(),
            "refunded": buckets.refunded.to_string(),
        }
    })))
}

pub async fn admin_payouts(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT p.id, a.bounty_id, a.amount_lamports, p.status::text, p.send_count, p.last_error, u.wallet_address,
         (SELECT signature FROM {chain} c WHERE c.payout_id = p.id ORDER BY c.created_at DESC LIMIT 1) AS signature
         FROM {payouts} p
         JOIN {allocations} a ON a.id = p.allocation_id
         JOIN {users} u ON u.id = a.solver_id
         ORDER BY p.created_at DESC LIMIT 200",
        payouts = tables::PAYOUTS,
        allocations = tables::REWARD_ALLOCATIONS,
        users = tables::USERS,
        chain = tables::CHAIN_TRANSACTIONS
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "bountyId": r.get::<Uuid, _>("bounty_id").to_string(),
                "amountLamports": r.get::<i64, _>("amount_lamports").to_string(),
                "status": r.get::<String, _>("status"),
                "destination": r.get::<String, _>("wallet_address"),
                "sendCount": r.get::<i32, _>("send_count"),
                "lastError": r.get::<Option<String>, _>("last_error"),
                "signature": r.get::<Option<String>, _>("signature"),
            })
        })
        .collect())
}

async fn lock_payout(tx: &mut Transaction<'_, sqlx::Postgres>, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "SELECT id FROM {table} WHERE id = $1 FOR UPDATE",
        table = tables::PAYOUTS
    ))
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn read_payout_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    id: Uuid,
) -> Result<Option<PayoutSnapshot>, sqlx::Error> {
    let row = sqlx::query(&format!(
        "SELECT p.id, p.status::text, a.amount_lamports, a.bounty_id, u.wallet_address
         FROM {payouts} p
         JOIN {allocations} a ON a.id = p.allocation_id
         JOIN {users} u ON u.id = a.solver_id
         WHERE p.id = $1",
        payouts = tables::PAYOUTS,
        allocations = tables::REWARD_ALLOCATIONS,
        users = tables::USERS
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let bounty_id: Uuid = row.get("bounty_id");
    let signatures = load_chain_sigs_tx(tx, id).await?;
    let ledger = load_ledger_for_payout_tx(tx, id, bounty_id).await?;
    let buckets = buckets_from_entries(&ledger);
    let settled = ledger.iter().any(|e| e.entry_type == "settle");
    Ok(Some(PayoutSnapshot {
        id: row.get("id"),
        status: row.get("status"),
        amount_lamports: row.get("amount_lamports"),
        bounty_id,
        solver_wallet: row.get("wallet_address"),
        reserved_lamports: buckets.reserved,
        settled,
        signatures,
    }))
}

async fn load_chain_sigs(pool: &PgPool, payout_id: Uuid) -> Result<Vec<ChainSig>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT signature, commitment FROM {table} WHERE payout_id = $1 ORDER BY created_at ASC",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(payout_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| ChainSig {
            signature: r.get("signature"),
            commitment: r.get("commitment"),
        })
        .collect())
}

async fn load_chain_sigs_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    payout_id: Uuid,
) -> Result<Vec<ChainSig>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT signature, commitment FROM {table} WHERE payout_id = $1 ORDER BY created_at ASC",
        table = tables::CHAIN_TRANSACTIONS
    ))
    .bind(payout_id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(rows
        .iter()
        .map(|r| ChainSig {
            signature: r.get("signature"),
            commitment: r.get("commitment"),
        })
        .collect())
}

async fn load_ledger_for_payout(
    pool: &PgPool,
    payout_id: Uuid,
    bounty_id: Uuid,
) -> Result<Vec<LedgerEntry>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT entry_type::text, amount_lamports FROM {table} WHERE bounty_id = $1 OR payout_id = $2",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(bounty_id)
    .bind(payout_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| LedgerEntry {
            entry_type: r.get("entry_type"),
            amount_lamports: r.get("amount_lamports"),
        })
        .collect())
}

async fn load_ledger_for_payout_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    payout_id: Uuid,
    bounty_id: Uuid,
) -> Result<Vec<LedgerEntry>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT entry_type::text, amount_lamports FROM {table} WHERE bounty_id = $1 OR payout_id = $2",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(bounty_id)
    .bind(payout_id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(rows
        .iter()
        .map(|r| LedgerEntry {
            entry_type: r.get("entry_type"),
            amount_lamports: r.get("amount_lamports"),
        })
        .collect())
}