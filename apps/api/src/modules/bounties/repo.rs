use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::db::tables;
use crate::modules::bounties::capacity::{capacity_decision, CapacityDecision, CapacityReason};
use crate::modules::bounties::dto::{
    OwnerBounty, PublicBountyRecord, StoredAttempt, StoredObligation, StoredSession,
};
use crate::modules::bounties::funding_ledger::{buckets_from_entries, deposit_reference, LedgerEntry};
use crate::modules::bounties::lamports::{commission_lamports, principal_lamports};
use crate::modules::bounties::validate::BountyWrite;

pub async fn create_bounty(
    pool: &PgPool,
    creator_id: Uuid,
    input: &BountyWrite,
) -> Result<OwnerBounty, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let now = Utc::now();
    let bounty_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, creator_id, title, description, instructions, category, reward_lamports, max_rewarded_solvers, attempt_limit, deadline, status, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6::bounty_category, $7, $8, $9, $10, 'draft'::bounty_status, $11)",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .bind(creator_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.instructions)
    .bind(&input.category)
    .bind(input.reward_lamports)
    .bind(input.max_rewarded_solvers)
    .bind(input.attempt_limit)
    .bind(input.deadline)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    let question_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, bounty_id, prompt, expected_answer_ciphertext, encryption_key_id, verification_method)
         VALUES ($1, $2, $3, $4, $5, $6::verification_method)",
        table = tables::BOUNTY_QUESTIONS
    ))
    .bind(question_id)
    .bind(bounty_id)
    .bind(&input.prompt)
    .bind(&input.expected_answer_ciphertext)
    .bind(&input.encryption_key_id)
    .bind(&input.verification_method)
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        Some(creator_id),
        "bounty.draft_saved",
        "bounty",
        bounty_id,
    )
    .await?;
    tx.commit().await?;
    load_owner(pool, bounty_id, Some(creator_id)).await
}

pub async fn update_draft(
    pool: &PgPool,
    id: Uuid,
    creator_id: Uuid,
    input: &BountyWrite,
) -> Result<Result<OwnerBounty, &'static str>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row: Option<(String,)> = sqlx::query_as(&format!(
        "SELECT status::text FROM {table} WHERE id = $1 AND creator_id = $2",
        table = tables::BOUNTIES
    ))
    .bind(id)
    .bind(creator_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((status,)) = row else {
        return Ok(Err("missing"));
    };
    if status != "draft" {
        return Ok(Err("not_draft"));
    }
    sqlx::query(&format!(
        "UPDATE {table} SET title = $1, description = $2, instructions = $3, category = $4::bounty_category,
         reward_lamports = $5, max_rewarded_solvers = $6, attempt_limit = $7, deadline = $8 WHERE id = $9",
        table = tables::BOUNTIES
    ))
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.instructions)
    .bind(&input.category)
    .bind(input.reward_lamports)
    .bind(input.max_rewarded_solvers)
    .bind(input.attempt_limit)
    .bind(input.deadline)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET prompt = $1, expected_answer_ciphertext = $2, encryption_key_id = $3, verification_method = $4::verification_method WHERE bounty_id = $5",
        table = tables::BOUNTY_QUESTIONS
    ))
    .bind(&input.prompt)
    .bind(&input.expected_answer_ciphertext)
    .bind(&input.encryption_key_id)
    .bind(&input.verification_method)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        Some(creator_id),
        "bounty.draft_updated",
        "bounty",
        id,
    )
    .await?;
    tx.commit().await?;
    let owner = load_owner(pool, id, Some(creator_id)).await?;
    Ok(Ok(owner))
}

pub async fn find_owned(
    pool: &PgPool,
    id: Uuid,
    creator_id: Uuid,
) -> Result<Option<OwnerBounty>, sqlx::Error> {
    match load_owner(pool, id, Some(creator_id)).await {
        Ok(v) => Ok(Some(v)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn list_owned(pool: &PgPool, creator_id: Uuid) -> Result<Vec<OwnerBounty>, sqlx::Error> {
    let ids: Vec<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE creator_id = $1 ORDER BY created_at DESC",
        table = tables::BOUNTIES
    ))
    .bind(creator_id)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Ok(row) = load_owner(pool, id, Some(creator_id)).await {
            out.push(row);
        }
    }
    Ok(out)
}

pub async fn close_past_deadline(pool: &PgPool, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'deadline'::bounty_close_reason, closed_at = $1
         WHERE status = 'active'::bounty_status AND deadline <= $1",
        table = tables::BOUNTIES
    ))
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_active(pool: &PgPool, now: DateTime<Utc>) -> Result<Vec<PublicBountyRecord>, sqlx::Error> {
    close_past_deadline(pool, now).await?;
    let rows = sqlx::query(&format!(
        "SELECT b.id, b.title, b.description, b.instructions, b.category::text, b.status::text,
         b.reward_lamports, b.max_rewarded_solvers, b.attempt_limit, b.deadline, b.close_reason::text,
         q.prompt, q.verification_method::text,
         (SELECT COUNT(*)::int FROM reward_allocations a WHERE a.bounty_id = b.id) AS rewarded_count
         FROM {bounties} b
         JOIN {questions} q ON q.bounty_id = b.id
         WHERE b.status = 'active'::bounty_status AND b.deadline > $1
         ORDER BY b.deadline ASC",
        bounties = tables::BOUNTIES,
        questions = tables::BOUNTY_QUESTIONS
    ))
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(read_public_row).collect())
}

pub async fn find_public(
    pool: &PgPool,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<Option<PublicBountyRecord>, sqlx::Error> {
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'deadline'::bounty_close_reason, closed_at = $1
         WHERE id = $2 AND status = 'active'::bounty_status AND deadline <= $1",
        table = tables::BOUNTIES
    ))
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    let row = sqlx::query(&format!(
        "SELECT b.id, b.title, b.description, b.instructions, b.category::text, b.status::text,
         b.reward_lamports, b.max_rewarded_solvers, b.attempt_limit, b.deadline, b.close_reason::text,
         q.prompt, q.verification_method::text,
         (SELECT COUNT(*)::int FROM reward_allocations a WHERE a.bounty_id = b.id) AS rewarded_count
         FROM {bounties} b
         JOIN {questions} q ON q.bounty_id = b.id
         WHERE b.id = $1 AND b.status NOT IN ('draft'::bounty_status, 'funding_pending'::bounty_status)",
        bounties = tables::BOUNTIES,
        questions = tables::BOUNTY_QUESTIONS
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| read_public_row(&r)))
}

pub struct FundingSnapshot {
    pub bounty: OwnerBounty,
    pub obligations: Vec<StoredObligation>,
    pub buckets: crate::modules::bounties::funding_ledger::LedgerBuckets,
}

pub async fn ensure_funding(
    pool: &PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
    expected_reward_pool: i64,
    expected_commission: i64,
) -> Result<Result<FundingSnapshot, &'static str>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_bounty(&mut tx, bounty_id).await?;
    let status = bounty_status(&mut tx, bounty_id, creator_id).await?;
    let Some(status) = status else {
        return Ok(Err("missing"));
    };
    if status == "closed" {
        return Ok(Err("closed"));
    }
    for (purpose, amount) in [("reward_pool", expected_reward_pool), ("commission", expected_commission)] {
        let existing: Option<(Uuid, Option<Uuid>, String, i64)> = sqlx::query_as(&format!(
            "SELECT id, chain_transaction_id, status::text, expected_lamports FROM {table}
             WHERE bounty_id = $1 AND purpose = $2::funding_purpose",
            table = tables::BOUNTY_FUNDINGS
        ))
        .bind(bounty_id)
        .bind(purpose)
        .fetch_optional(&mut *tx)
        .await?;
        if existing.is_none() {
            let funding_id = Uuid::new_v4();
            sqlx::query(&format!(
                "INSERT INTO {table} (id, bounty_id, purpose, deposit_reference, expected_lamports, status)
                 VALUES ($1, $2, $3::funding_purpose, $4, $5, 'pending'::funding_status)",
                table = tables::BOUNTY_FUNDINGS
            ))
            .bind(funding_id)
            .bind(bounty_id)
            .bind(purpose)
            .bind(deposit_reference(&bounty_id.to_string(), purpose))
            .bind(amount)
            .execute(&mut *tx)
            .await?;
        } else if let Some((fid, chain_id, st, old)) = existing {
            if chain_id.is_none() && st == "pending" && old != amount {
                sqlx::query(&format!(
                    "UPDATE {table} SET expected_lamports = $1 WHERE id = $2",
                    table = tables::BOUNTY_FUNDINGS
                ))
                .bind(amount)
                .bind(fid)
                .execute(&mut *tx)
                .await?;
            }
        }
    }
    tx.commit().await?;
    load_funding_snapshot(pool, bounty_id, creator_id).await.map(Ok)
}

pub async fn load_funding_snapshot(
    pool: &PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
) -> Result<FundingSnapshot, sqlx::Error> {
    let bounty = load_owner(pool, bounty_id, Some(creator_id)).await?;
    let obligations = load_obligations(pool, bounty_id).await?;
    let buckets = load_buckets(pool, bounty_id).await?;
    Ok(FundingSnapshot {
        bounty,
        obligations,
        buckets,
    })
}

pub async fn find_signature_use(
    pool: &PgPool,
    signature: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let row: Option<(Uuid, String)> = sqlx::query_as(&format!(
        "SELECT f.bounty_id, f.purpose::text FROM {chain} c
         JOIN {fundings} f ON f.chain_transaction_id = c.id
         WHERE c.signature = $1 LIMIT 1",
        chain = tables::CHAIN_TRANSACTIONS,
        fundings = tables::BOUNTY_FUNDINGS
    ))
    .bind(signature)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub struct ApplyFundingInput {
    pub bounty_id: Uuid,
    pub creator_id: Uuid,
    pub purpose: String,
    pub signature: String,
    pub source_address: String,
    pub destination_address: String,
    pub amount_lamports: i64,
    pub commitment: String,
    pub rejected: bool,
    pub now: DateTime<Utc>,
}

pub enum ApplyFundingResult {
    Missing,
    SignatureInUse,
    AlreadyConfirmed,
    Applied(FundingSnapshot),
}

pub async fn apply_funding(
    pool: &PgPool,
    input: ApplyFundingInput,
) -> Result<ApplyFundingResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_bounty(&mut tx, input.bounty_id).await?;
    let status = bounty_status(&mut tx, input.bounty_id, input.creator_id).await?;
    if !matches!(status.as_deref(), Some(s) if s != "closed" && s != "active") {
        return Ok(ApplyFundingResult::Missing);
    }
    let obligation: Option<(Uuid, String, Option<String>)> = sqlx::query_as(&format!(
        "SELECT f.id, f.status::text, c.signature FROM {fundings} f
         LEFT JOIN {chain} c ON c.id = f.chain_transaction_id
         WHERE f.bounty_id = $1 AND f.purpose = $2::funding_purpose",
        fundings = tables::BOUNTY_FUNDINGS,
        chain = tables::CHAIN_TRANSACTIONS
    ))
    .bind(input.bounty_id)
    .bind(&input.purpose)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((funding_id, ob_status, existing_sig)) = obligation else {
        return Ok(ApplyFundingResult::Missing);
    };
    if ob_status == "confirmed" {
        if existing_sig.as_deref() == Some(&input.signature) {
            tx.commit().await?;
            let snap = load_funding_snapshot(pool, input.bounty_id, input.creator_id).await?;
            return Ok(ApplyFundingResult::Applied(snap));
        }
        return Ok(ApplyFundingResult::AlreadyConfirmed);
    }
    let other_use: Option<(Uuid, String)> = sqlx::query_as(&format!(
        "SELECT f.bounty_id, f.purpose::text FROM {chain} c
         JOIN {fundings} f ON f.chain_transaction_id = c.id
         WHERE c.signature = $1 AND (f.bounty_id <> $2 OR f.purpose::text <> $3) LIMIT 1",
        chain = tables::CHAIN_TRANSACTIONS,
        fundings = tables::BOUNTY_FUNDINGS
    ))
    .bind(&input.signature)
    .bind(input.bounty_id)
    .bind(&input.purpose)
    .fetch_optional(&mut *tx)
    .await?;
    if other_use.is_some() {
        return Ok(ApplyFundingResult::SignatureInUse);
    }
    let chain_id: Uuid = if let Some((id,)) = sqlx::query_as::<_, (Uuid,)>(
        &format!(
            "SELECT id FROM {table} WHERE signature = $1",
            table = tables::CHAIN_TRANSACTIONS
        ),
    )
    .bind(&input.signature)
    .fetch_optional(&mut *tx)
    .await?
    {
        id
    } else {
        let new_chain_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, signature, kind, amount_lamports, source_address, destination_address, commitment, confirmed_at)
             VALUES ($1, $2, 'deposit'::chain_transaction_kind, $3, $4, $5, $6, $7)",
            table = tables::CHAIN_TRANSACTIONS
        ))
        .bind(new_chain_id)
        .bind(&input.signature)
        .bind(input.amount_lamports)
        .bind(&input.source_address)
        .bind(&input.destination_address)
        .bind(&input.commitment)
        .bind(if input.rejected { None::<DateTime<Utc>> } else { Some(input.now) })
        .execute(&mut *tx)
        .await?;
        new_chain_id
    };
    sqlx::query(&format!(
        "UPDATE {table} SET chain_transaction_id = $1, observed_lamports = $2, status = $3::funding_status, confirmed_at = $4 WHERE id = $5",
        table = tables::BOUNTY_FUNDINGS
    ))
    .bind(chain_id)
    .bind(input.amount_lamports)
    .bind(if input.rejected { "rejected" } else { "confirmed" })
    .bind(if input.rejected { None::<DateTime<Utc>> } else { Some(input.now) })
    .bind(funding_id)
    .execute(&mut *tx)
    .await?;
    if !input.rejected && input.purpose == "reward_pool" {
        let posted: Option<Uuid> = sqlx::query_scalar(&format!(
            "SELECT id FROM {table} WHERE chain_transaction_id = $1 AND entry_type = 'deposit'::ledger_entry_type LIMIT 1",
            table = tables::BOUNTY_LEDGER_ENTRIES
        ))
        .bind(chain_id)
        .fetch_optional(&mut *tx)
        .await?;
        if posted.is_none() {
            let ledger_id = Uuid::new_v4();
            sqlx::query(&format!(
                "INSERT INTO {table} (id, bounty_id, entry_type, amount_lamports, funding_id, chain_transaction_id)
                 VALUES ($1, $2, 'deposit'::ledger_entry_type, $3, $4, $5)",
                table = tables::BOUNTY_LEDGER_ENTRIES
            ))
            .bind(ledger_id)
            .bind(input.bounty_id)
            .bind(input.amount_lamports)
            .bind(funding_id)
            .bind(chain_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    if status == Some("draft".into()) {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'funding_pending'::bounty_status WHERE id = $1",
            table = tables::BOUNTIES
        ))
        .bind(input.bounty_id)
        .execute(&mut *tx)
        .await?;
    }
    let action = if input.rejected {
        "funding.rejected"
    } else {
        match input.purpose.as_str() {
            "reward_pool" => "funding.reward_pool_confirmed",
            _ => "funding.commission_confirmed",
        }
    };
    audit(
        &mut tx,
        Some(input.creator_id),
        action,
        "bounty",
        input.bounty_id,
    )
    .await?;
    tx.commit().await?;
    let snap = load_funding_snapshot(pool, input.bounty_id, input.creator_id).await?;
    Ok(ApplyFundingResult::Applied(snap))
}

pub async fn activate_if_funded(
    pool: &PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Result<OwnerBounty, &'static str>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_bounty(&mut tx, bounty_id).await?;
    let status = bounty_status(&mut tx, bounty_id, creator_id).await?;
    let Some(status) = status else {
        return Ok(Err("missing"));
    };
    if status == "closed" {
        return Ok(Err("closed"));
    }
    if status == "active" {
        tx.commit().await?;
        let owner = load_owner(pool, bounty_id, Some(creator_id)).await?;
        return Ok(Ok(owner));
    }
    let counts: (i64, i64) = sqlx::query_as(&format!(
        "SELECT
           COUNT(*) FILTER (WHERE purpose = 'reward_pool'::funding_purpose AND status = 'confirmed'::funding_status),
           COUNT(*) FILTER (WHERE purpose = 'commission'::funding_purpose AND status = 'confirmed'::funding_status)
         FROM {table} WHERE bounty_id = $1",
        table = tables::BOUNTY_FUNDINGS
    ))
    .bind(bounty_id)
    .fetch_one(&mut *tx)
    .await?;
    if counts.0 < 1 || counts.1 < 1 {
        return Ok(Err("incomplete"));
    }
    sqlx::query(&format!(
        "UPDATE {table} SET status = 'active'::bounty_status, published_at = $1 WHERE id = $2",
        table = tables::BOUNTIES
    ))
    .bind(now)
    .bind(bounty_id)
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        Some(creator_id),
        "bounty.activated",
        "bounty",
        bounty_id,
    )
    .await?;
    tx.commit().await?;
    load_owner(pool, bounty_id, Some(creator_id)).await.map(Ok)
}

pub async fn record_consent(
    pool: &PgPool,
    bounty_id: Uuid,
    user_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Result<(String, DateTime<Utc>), &'static str>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_bounty(&mut tx, bounty_id).await?;
    let row: Option<(String, DateTime<Utc>)> = sqlx::query_as(&format!(
        "SELECT status::text, deadline FROM {table} WHERE id = $1",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((status, deadline)) = row else {
        return Ok(Err("missing"));
    };
    if status == "draft" || status == "funding_pending" {
        return Ok(Err("missing"));
    }
    if status != "active" || deadline <= now {
        if status == "active" {
            sqlx::query(&format!(
                "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'deadline'::bounty_close_reason, closed_at = $1 WHERE id = $2",
                table = tables::BOUNTIES
            ))
            .bind(now)
            .bind(bounty_id)
            .execute(&mut *tx)
            .await?;
        }
        return Ok(Err("inactive"));
    }
    let existing: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(&format!(
        "SELECT id, accepted_at FROM {table} WHERE bounty_id = $1 AND user_id = $2",
        table = tables::CONSENT_RECORDS
    ))
    .bind(bounty_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((_, accepted_at)) = existing {
        tx.commit().await?;
        return Ok(Ok((
            crate::modules::bounties::validate::RESEARCH_CONSENT_VERSION.into(),
            accepted_at,
        )));
    }
    let consent_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, user_id, bounty_id, policy_version, accepted_at) VALUES ($1, $2, $3, $4, $5)",
        table = tables::CONSENT_RECORDS
    ))
    .bind(consent_id)
    .bind(user_id)
    .bind(bounty_id)
    .bind(crate::modules::bounties::validate::RESEARCH_CONSENT_VERSION)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Ok((
        crate::modules::bounties::validate::RESEARCH_CONSENT_VERSION.into(),
        now,
    )))
}

pub async fn start_session(
    pool: &PgPool,
    bounty_id: Uuid,
    solver_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Result<StoredSession, &'static str>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    lock_bounty(&mut tx, bounty_id).await?;
    let bounty: Option<(String, DateTime<Utc>, i32, i32)> = sqlx::query_as(&format!(
        "SELECT status::text, deadline, max_rewarded_solvers, attempt_limit FROM {table} WHERE id = $1",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((status, deadline, max_solvers, _attempt_limit)) = bounty else {
        return Ok(Err("missing"));
    };
    if status == "draft" || status == "funding_pending" {
        return Ok(Err("missing"));
    }
    if status == "active" && deadline <= now {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'deadline'::bounty_close_reason, closed_at = $1 WHERE id = $2",
            table = tables::BOUNTIES
        ))
        .bind(now)
        .bind(bounty_id)
        .execute(&mut *tx)
        .await?;
        return Ok(Err("inactive"));
    }
    if status != "active" {
        let close: Option<String> = sqlx::query_scalar(&format!(
            "SELECT close_reason::text FROM {table} WHERE id = $1",
            table = tables::BOUNTIES
        ))
        .bind(bounty_id)
        .fetch_optional(&mut *tx)
        .await?;
        if close.as_deref() == Some("fully_allocated") {
            return Ok(Err("full"));
        }
        return Ok(Err("inactive"));
    }
    let allocation_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {table} WHERE bounty_id = $1",
        table = tables::REWARD_ALLOCATIONS
    ))
    .bind(bounty_id)
    .fetch_one(&mut *tx)
    .await?;
    if allocation_count >= max_solvers as i64 {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'fully_allocated'::bounty_close_reason, closed_at = $1 WHERE id = $2",
            table = tables::BOUNTIES
        ))
        .bind(now)
        .bind(bounty_id)
        .execute(&mut *tx)
        .await?;
        return Ok(Err("full"));
    }
    let consent_id: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE bounty_id = $1 AND user_id = $2",
        table = tables::CONSENT_RECORDS
    ))
    .bind(bounty_id)
    .bind(solver_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(consent_id) = consent_id else {
        return Ok(Err("no_consent"));
    };
    let existing: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT id FROM {table} WHERE bounty_id = $1 AND solver_id = $2",
        table = tables::SOLVING_SESSIONS
    ))
    .bind(bounty_id)
    .bind(solver_id)
    .fetch_optional(&mut *tx)
    .await?;
    let session_id = if let Some(id) = existing {
        id
    } else {
        let new_session_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, bounty_id, solver_id, consent_id, started_at, status)
             VALUES ($1, $2, $3, $4, $5, 'active'::solving_session_status)",
            table = tables::SOLVING_SESSIONS
        ))
        .bind(new_session_id)
        .bind(bounty_id)
        .bind(solver_id)
        .bind(consent_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        new_session_id
    };
    tx.commit().await?;
    let session = read_session(pool, session_id, solver_id).await?;
    Ok(session.ok_or("missing"))
}

pub async fn read_session(
    pool: &PgPool,
    session_id: Uuid,
    solver_id: Uuid,
) -> Result<Option<StoredSession>, sqlx::Error> {
    load_session(pool, session_id, solver_id).await
}

pub async fn verification_material(
    pool: &PgPool,
    session_id: Uuid,
    solver_id: Uuid,
) -> Result<Option<(String, String)>, sqlx::Error> {
    let row = sqlx::query(&format!(
        "SELECT q.expected_answer_ciphertext, q.verification_method::text
         FROM {sessions} s
         JOIN {questions} q ON q.bounty_id = s.bounty_id
         WHERE s.id = $1 AND s.solver_id = $2",
        sessions = tables::SOLVING_SESSIONS,
        questions = tables::BOUNTY_QUESTIONS
    ))
    .bind(session_id)
    .bind(solver_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.get("expected_answer_ciphertext"), r.get("verification_method"))))
}

pub enum SubmitAnswerResult {
    Missing,
    Completed,
    Inactive,
    Limit,
    Stored(StoredSession),
}

pub async fn submit_answer(
    pool: &PgPool,
    session_id: Uuid,
    solver_id: Uuid,
    answer: &str,
    now: DateTime<Utc>,
    correct: bool,
) -> Result<SubmitAnswerResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let session_row: Option<(Uuid, String)> = sqlx::query_as(&format!(
        "SELECT bounty_id, status::text FROM {table} WHERE id = $1 AND solver_id = $2",
        table = tables::SOLVING_SESSIONS
    ))
    .bind(session_id)
    .bind(solver_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((bounty_id, session_status)) = session_row else {
        return Ok(SubmitAnswerResult::Missing);
    };
    lock_bounty(&mut tx, bounty_id).await?;
    if session_status == "completed" {
        return Ok(SubmitAnswerResult::Completed);
    }
    let bounty: (String, DateTime<Utc>, i32, i32, i64) = sqlx::query_as(&format!(
        "SELECT status::text, deadline, attempt_limit, max_rewarded_solvers, reward_lamports FROM {table} WHERE id = $1",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .fetch_one(&mut *tx)
    .await?;
    let (b_status, deadline, attempt_limit, max_solvers, reward_lamports) = bounty;
    if b_status == "active" && deadline <= now {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'deadline'::bounty_close_reason, closed_at = $1 WHERE id = $2",
            table = tables::BOUNTIES
        ))
        .bind(now)
        .bind(bounty_id)
        .execute(&mut *tx)
        .await?;
        return Ok(SubmitAnswerResult::Inactive);
    }
    if b_status != "active" {
        return Ok(SubmitAnswerResult::Inactive);
    }
    let prior: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {table} WHERE session_id = $1",
        table = tables::SUBMISSION_ATTEMPTS
    ))
    .bind(session_id)
    .fetch_one(&mut *tx)
    .await?;
    if prior >= attempt_limit as i64 {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'completed'::solving_session_status WHERE id = $1",
            table = tables::SOLVING_SESSIONS
        ))
        .bind(session_id)
        .execute(&mut *tx)
        .await?;
        return Ok(SubmitAnswerResult::Limit);
    }
    let buckets = load_buckets_tx(&mut tx, bounty_id).await?;
    let allocation_count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {table} WHERE bounty_id = $1",
        table = tables::REWARD_ALLOCATIONS
    ))
    .bind(bounty_id)
    .fetch_one(&mut *tx)
    .await?;
    let capacity = if correct {
        Some(capacity_decision(
            allocation_count as i32,
            max_solvers,
            buckets.refundable,
            reward_lamports,
        ))
    } else {
        None
    };
    let ineligibility = match capacity {
        Some(CapacityDecision::Reject(CapacityReason::PoolExhausted)) => Some("pool_exhausted"),
        Some(CapacityDecision::Reject(CapacityReason::InsufficientRefundable)) => {
            Some("insufficient_refundable")
        }
        _ => None,
    };
    let attempt_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, session_id, attempt_number, submitted_answer, is_correct, ineligibility_reason, submitted_at)
         VALUES ($1, $2, $3, $4, $5, $6::ineligibility_reason, $7)",
        table = tables::SUBMISSION_ATTEMPTS
    ))
    .bind(attempt_id)
    .bind(session_id)
    .bind(prior as i32 + 1)
    .bind(answer)
    .bind(correct)
    .bind(ineligibility)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    let mut reserved = false;
    if matches!(capacity, Some(CapacityDecision::Reserve)) {
        let allocation_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, bounty_id, solver_id, attempt_id, amount_lamports, status)
             VALUES ($1, $2, $3, $4, $5, 'reserved'::allocation_status)",
            table = tables::REWARD_ALLOCATIONS
        ))
        .bind(allocation_id)
        .bind(bounty_id)
        .bind(solver_id)
        .bind(attempt_id)
        .bind(reward_lamports)
        .execute(&mut *tx)
        .await?;
        let payout_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, allocation_id, idempotency_key, status, updated_at)
             VALUES ($1, $2, $3, 'pending'::payout_status, $4)",
            table = tables::PAYOUTS
        ))
        .bind(payout_id)
        .bind(allocation_id)
        .bind(format!("payout:{bounty_id}:{solver_id}"))
        .bind(now)
        .execute(&mut *tx)
        .await?;
        let reserve_ledger_id = Uuid::new_v4();
        sqlx::query(&format!(
            "INSERT INTO {table} (id, bounty_id, entry_type, amount_lamports, payout_id)
             VALUES ($1, $2, 'reserve'::ledger_entry_type, $3, $4)",
            table = tables::BOUNTY_LEDGER_ENTRIES
        ))
        .bind(reserve_ledger_id)
        .bind(bounty_id)
        .bind(reward_lamports)
        .bind(payout_id)
        .execute(&mut *tx)
        .await?;
        if allocation_count + 1 >= max_solvers as i64 {
            sqlx::query(&format!(
                "UPDATE {table} SET status = 'closed'::bounty_status, close_reason = 'fully_allocated'::bounty_close_reason, closed_at = $1 WHERE id = $2",
                table = tables::BOUNTIES
            ))
            .bind(now)
            .bind(bounty_id)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'completed'::solving_session_status WHERE id = $1",
            table = tables::SOLVING_SESSIONS
        ))
        .bind(session_id)
        .execute(&mut *tx)
        .await?;
        reserved = true;
    } else if correct || prior + 1 >= attempt_limit as i64 {
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'completed'::solving_session_status WHERE id = $1",
            table = tables::SOLVING_SESSIONS
        ))
        .bind(session_id)
        .execute(&mut *tx)
        .await?;
    }
    audit(
        &mut tx,
        Some(solver_id),
        "submission.recorded",
        "solving_session",
        session_id,
    )
    .await?;
    tx.commit().await?;
    let session = load_session(pool, session_id, solver_id).await?;
    if let Some(mut s) = session {
        if let Some(last) = s.attempts.last_mut() {
            last.reward_reserved = reserved;
        }
        Ok(SubmitAnswerResult::Stored(s))
    } else {
        Ok(SubmitAnswerResult::Missing)
    }
}

pub struct ResearchAggregates {
    pub participant_count: i32,
    pub correct_count: i32,
    pub incorrect_count: i32,
    pub attempt_count: i32,
    pub average_completion_time_ms: Option<i32>,
}

pub async fn creator_analytics(
    pool: &PgPool,
    bounty_id: Uuid,
    creator_id: Uuid,
) -> Result<Option<ResearchAggregates>, sqlx::Error> {
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
    let sessions = sqlx::query(&format!(
        "SELECT s.id, s.started_at FROM {sessions} s WHERE s.bounty_id = $1",
        sessions = tables::SOLVING_SESSIONS
    ))
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;
    let participant_count = sessions.len() as i32;
    let mut correct_count = 0i32;
    let mut incorrect_count = 0i32;
    let mut durations = Vec::new();
    for session in sessions {
        let sid: Uuid = session.get("id");
        let started: DateTime<Utc> = session.get("started_at");
        let attempts = sqlx::query(&format!(
            "SELECT is_correct, submitted_at, attempt_number FROM {table} WHERE session_id = $1 ORDER BY attempt_number ASC",
            table = tables::SUBMISSION_ATTEMPTS
        ))
        .bind(sid)
        .fetch_all(pool)
        .await?;
        for row in &attempts {
            let is_correct: bool = row.get("is_correct");
            if is_correct {
                correct_count += 1;
            } else {
                incorrect_count += 1;
            }
        }
        let first_correct = attempts.iter().find(|r| r.get::<bool, _>("is_correct"));
        if let Some(row) = first_correct {
            let submitted: DateTime<Utc> = row.get("submitted_at");
            durations.push((submitted - started).num_milliseconds() as i32);
        }
    }
    let attempt_count = correct_count + incorrect_count;
    let average_completion_time_ms = if durations.is_empty() {
        None
    } else {
        Some(durations.iter().sum::<i32>() / durations.len() as i32)
    };
    Ok(Some(ResearchAggregates {
        participant_count,
        correct_count,
        incorrect_count,
        attempt_count,
        average_completion_time_ms,
    }))
}

async fn load_owner(
    pool: &PgPool,
    id: Uuid,
    creator_id: Option<Uuid>,
) -> Result<OwnerBounty, sqlx::Error> {
    let mut query = format!(
        "SELECT b.id, b.creator_id, b.title, b.description, b.instructions, b.category::text, b.reward_lamports,
         b.max_rewarded_solvers, b.attempt_limit, b.deadline, b.status::text, b.close_reason::text,
         q.prompt, q.expected_answer_ciphertext, q.encryption_key_id, q.verification_method::text
         FROM {bounties} b JOIN {questions} q ON q.bounty_id = b.id WHERE b.id = $1",
        bounties = tables::BOUNTIES,
        questions = tables::BOUNTY_QUESTIONS
    );
    if creator_id.is_some() {
        query.push_str(" AND b.creator_id = $2");
    }
    let row = if let Some(cid) = creator_id {
        sqlx::query(&query).bind(id).bind(cid).fetch_one(pool).await?
    } else {
        sqlx::query(&query).bind(id).fetch_one(pool).await?
    };
    Ok(read_owner_row(&row))
}

fn read_owner_row(row: &sqlx::postgres::PgRow) -> OwnerBounty {
    OwnerBounty {
        id: row.get::<Uuid, _>("id").to_string(),
        title: row.get("title"),
        description: row.get("description"),
        instructions: row.get("instructions"),
        category: row.get("category"),
        reward_lamports: row.get("reward_lamports"),
        max_rewarded_solvers: row.get("max_rewarded_solvers"),
        attempt_limit: row.get("attempt_limit"),
        deadline: row.get("deadline"),
        status: row.get("status"),
        close_reason: row.get("close_reason"),
        prompt: row.get("prompt"),
        expected_answer_ciphertext: row.get("expected_answer_ciphertext"),
        encryption_key_id: row.get("encryption_key_id"),
        verification_method: row.get("verification_method"),
    }
}

fn read_public_row(row: &sqlx::postgres::PgRow) -> PublicBountyRecord {
    PublicBountyRecord {
        id: row.get::<Uuid, _>("id").to_string(),
        title: row.get("title"),
        description: row.get("description"),
        instructions: row.get("instructions"),
        prompt: row.get("prompt"),
        category: row.get("category"),
        verification_method: row.get("verification_method"),
        status: row.get("status"),
        reward_lamports: row.get("reward_lamports"),
        max_rewarded_solvers: row.get("max_rewarded_solvers"),
        attempt_limit: row.get("attempt_limit"),
        deadline: row.get("deadline"),
        close_reason: row.get("close_reason"),
        rewarded_count: row.get("rewarded_count"),
    }
}

async fn load_obligations(pool: &PgPool, bounty_id: Uuid) -> Result<Vec<StoredObligation>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT f.purpose::text, f.deposit_reference, f.expected_lamports, f.observed_lamports, f.status::text,
         c.signature, c.commitment FROM {fundings} f
         LEFT JOIN {chain} c ON c.id = f.chain_transaction_id WHERE f.bounty_id = $1 ORDER BY f.purpose::text",
        fundings = tables::BOUNTY_FUNDINGS,
        chain = tables::CHAIN_TRANSACTIONS
    ))
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| StoredObligation {
            purpose: r.get("purpose"),
            deposit_reference: r.get("deposit_reference"),
            expected_lamports: r.get("expected_lamports"),
            observed_lamports: r.get("observed_lamports"),
            status: r.get("status"),
            signature: r.get("signature"),
            commitment: r.get("commitment"),
        })
        .collect())
}

async fn load_buckets(
    pool: &PgPool,
    bounty_id: Uuid,
) -> Result<crate::modules::bounties::funding_ledger::LedgerBuckets, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT entry_type::text, amount_lamports FROM {table} WHERE bounty_id = $1",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(bounty_id)
    .fetch_all(pool)
    .await?;
    let entries: Vec<LedgerEntry> = rows
        .iter()
        .map(|r| LedgerEntry {
            entry_type: r.get("entry_type"),
            amount_lamports: r.get("amount_lamports"),
        })
        .collect();
    Ok(buckets_from_entries(&entries))
}

async fn load_buckets_tx(
    tx: &mut Transaction<'_, Postgres>,
    bounty_id: Uuid,
) -> Result<crate::modules::bounties::funding_ledger::LedgerBuckets, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT entry_type::text, amount_lamports FROM {table} WHERE bounty_id = $1",
        table = tables::BOUNTY_LEDGER_ENTRIES
    ))
    .bind(bounty_id)
    .fetch_all(&mut **tx)
    .await?;
    let entries: Vec<LedgerEntry> = rows
        .iter()
        .map(|r| LedgerEntry {
            entry_type: r.get("entry_type"),
            amount_lamports: r.get("amount_lamports"),
        })
        .collect();
    Ok(buckets_from_entries(&entries))
}

async fn load_session(pool: &PgPool, session_id: Uuid, solver_id: Uuid) -> Result<Option<StoredSession>, sqlx::Error> {
    let row = sqlx::query(&format!(
        "SELECT s.id, s.bounty_id, s.solver_id, s.status::text, s.started_at,
         b.title, b.instructions, b.attempt_limit, b.reward_lamports, b.deadline,
         q.prompt, q.verification_method::text
         FROM {sessions} s
         JOIN {bounties} b ON b.id = s.bounty_id
         JOIN {questions} q ON q.bounty_id = b.id
         WHERE s.id = $1 AND s.solver_id = $2",
        sessions = tables::SOLVING_SESSIONS,
        bounties = tables::BOUNTIES,
        questions = tables::BOUNTY_QUESTIONS
    ))
    .bind(session_id)
    .bind(solver_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let attempts = sqlx::query(&format!(
        "SELECT a.attempt_number, a.is_correct, a.ineligibility_reason::text, a.submitted_at,
         (SELECT COUNT(*) > 0 FROM {allocations} ra WHERE ra.attempt_id = a.id) AS reward_reserved
         FROM {attempts} a WHERE a.session_id = $1 ORDER BY a.attempt_number ASC",
        allocations = tables::REWARD_ALLOCATIONS,
        attempts = tables::SUBMISSION_ATTEMPTS
    ))
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(Some(StoredSession {
        id: row.get::<Uuid, _>("id").to_string(),
        bounty_id: row.get::<Uuid, _>("bounty_id").to_string(),
        solver_id: row.get::<Uuid, _>("solver_id").to_string(),
        status: row.get("status"),
        started_at: row.get("started_at"),
        attempt_limit: row.get("attempt_limit"),
        title: row.get("title"),
        prompt: row.get("prompt"),
        instructions: row.get("instructions"),
        verification_method: row.get("verification_method"),
        reward_lamports: row.get("reward_lamports"),
        deadline: row.get("deadline"),
        attempts: attempts
            .iter()
            .map(|a| StoredAttempt {
                attempt_number: a.get("attempt_number"),
                is_correct: a.get("is_correct"),
                ineligibility_reason: a.get("ineligibility_reason"),
                submitted_at: a.get("submitted_at"),
                reward_reserved: a.get("reward_reserved"),
            })
            .collect(),
    }))
}

async fn lock_bounty(tx: &mut Transaction<'_, Postgres>, bounty_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "SELECT id FROM {table} WHERE id = $1 FOR UPDATE",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn bounty_status(
    tx: &mut Transaction<'_, Postgres>,
    bounty_id: Uuid,
    creator_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(&format!(
        "SELECT status::text FROM {table} WHERE id = $1 AND creator_id = $2",
        table = tables::BOUNTIES
    ))
    .bind(bounty_id)
    .bind(creator_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|(s,)| s))
}

async fn audit(
    tx: &mut Transaction<'_, Postgres>,
    actor: Option<Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
) -> Result<(), sqlx::Error> {
    let audit_id = Uuid::new_v4();
    sqlx::query(&format!(
        "INSERT INTO {table} (id, actor_user_id, action, entity_type, entity_id) VALUES ($1, $2, $3, $4, $5)",
        table = tables::AUDIT_LOGS
    ))
    .bind(audit_id)
    .bind(actor)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub fn expected_funding_amounts(reward_lamports: i64, max_solvers: i32) -> (i64, i64) {
    let pool = principal_lamports(reward_lamports, max_solvers);
    (pool, commission_lamports(pool))
}