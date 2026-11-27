use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::task::JoinHandle;
use tracing::warn;

use crate::config::AppConfig;
use crate::modules::solana::gateway::{MockSolanaGateway, SolanaGateway, TransferLeg};

pub const TICK_MS: u64 = 15_000;
pub const BATCH: usize = 10;

pub fn payout_memo(payout_id: &str) -> String {
    format!("sb:payout:{payout_id}")
}

pub struct PayoutWorker {
    pool: Option<PgPool>,
    config: Arc<AppConfig>,
    gateway: Arc<dyn SolanaGateway>,
    running: AtomicBool,
    tick_started: AtomicBool,
    handle: std::sync::Mutex<Option<JoinHandle<()>>>,
}

impl PayoutWorker {
    pub fn new(pool: Option<PgPool>, config: Arc<AppConfig>) -> Self {
        Self {
            pool,
            config,
            gateway: Arc::new(MockSolanaGateway::default()),
            running: AtomicBool::new(false),
            tick_started: AtomicBool::new(false),
            handle: std::sync::Mutex::new(None),
        }
    }

    pub fn disabled() -> Self {
        Self::new(
            None,
            Arc::new(crate::config::AppConfig {
                node_env: crate::config::NodeEnv::Test,
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
        )
    }

    pub fn with_gateway(mut self, gateway: Arc<dyn SolanaGateway>) -> Self {
        self.gateway = gateway;
        self
    }

    pub fn start(self: Arc<Self>) {
        if self.tick_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let worker = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(TICK_MS));
            loop {
                interval.tick().await;
                worker.nudge().await;
            }
        });
        *self.handle.lock().unwrap() = Some(handle);
    }

    pub async fn nudge(&self) {
        if self.pool.is_none() {
            return;
        }
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let result = self.process_batch().await;
        if result.is_err() {
            warn!("Payout processing skipped.");
        }
        self.running.store(false, Ordering::SeqCst);
    }

    async fn process_batch(&self) -> Result<(), ()> {
        use chrono::Utc;

        use crate::modules::rewards::repo;
        use crate::modules::solana::gateway::TransferOutcome;

        let pool = self.pool.as_ref().ok_or(())?;
        let ids = repo::list_actionable(pool, BATCH as i64)
            .await
            .map_err(|_| ())?;
        for id in ids {
            let Some(payout) = repo::read_payout(pool, id).await.map_err(|_| ())? else {
                continue;
            };
            if payout.status == "confirmed" || payout.status == "failed" {
                continue;
            }
            if let Some(open) = payout
                .signatures
                .iter()
                .find(|s| s.commitment == "submitted")
            {
                let lookup = self.gateway.lookup(&open.signature).await;
                if lookup.outcome == TransferOutcome::Unavailable
                    || lookup.outcome == TransferOutcome::NotFound
                {
                    continue;
                }
                let Some(transfer) = lookup.transfer else {
                    continue;
                };
                if transfer.failed {
                    let _ = repo::note_retryable(pool, id, &open.signature).await;
                    continue;
                }
                if transfer.cluster != "devnet" {
                    continue;
                }
                let treasury = self.config.treasury_public_key.clone().unwrap_or_default();
                if matches_payout(&transfer, &payout, &treasury) {
                    let commitment = if transfer.commitment == "finalized" {
                        "finalized"
                    } else {
                        "confirmed"
                    };
                    let _ = repo::settle(pool, id, &open.signature, commitment, Utc::now()).await;
                }
                continue;
            }
            if payout.status != "pending" && payout.status != "retryable" {
                continue;
            }
            if !self.broadcast_enabled() {
                continue;
            }
            if payout.reserved_lamports != payout.amount_lamports || payout.settled {
                let _ = repo::note_last_error(
                    pool,
                    id,
                    "The reserved balance does not match the allocation.",
                )
                .await;
                continue;
            }
            let treasury = self.config.treasury_public_key.clone().unwrap_or_default();
            if treasury.is_empty() {
                let _ =
                    repo::note_last_error(pool, id, "Treasury identity is not configured.").await;
                continue;
            }
            let Some(secret) = self.config.treasury_secret_key.clone() else {
                let _ = repo::note_last_error(pool, id, "Treasury signing is not ready.").await;
                continue;
            };
            let Some(blockhash) = self.gateway.recent_blockhash().await else {
                let _ = repo::note_last_error(pool, id, "Devnet RPC is not ready.").await;
                continue;
            };
            let memo = crate::modules::solana::payout_memo(&payout.id.to_string());
            let signed = crate::modules::solana::sign_payout_transfer(
                crate::modules::solana::SignPayoutInput {
                    secret: &secret,
                    destination: &payout.solver_wallet,
                    lamports: payout.amount_lamports,
                    memo: &memo,
                    recent_blockhash: &blockhash,
                    expected_treasury: &treasury,
                },
            );
            let Some(signed) = signed else {
                let _ = repo::note_last_error(pool, id, "The payout could not be signed.").await;
                continue;
            };
            let recorded = repo::record_send(
                pool,
                id,
                &signed.signature,
                &signed.treasury,
                &payout.solver_wallet,
                payout.amount_lamports,
            )
            .await
            .unwrap_or(repo::RecordSendResult::Blocked);
            if !matches!(recorded, repo::RecordSendResult::Recorded) {
                continue;
            }
            let _ = self.gateway.submit(&signed.serialized).await;
        }
        Ok(())
    }

    pub fn broadcast_enabled(&self) -> bool {
        self.config.payout_broadcast_enabled
    }

    pub fn gateway(&self) -> Arc<dyn SolanaGateway> {
        Arc::clone(&self.gateway)
    }
}

fn matches_payout(
    transfer: &crate::modules::solana::gateway::ObservedTransfer,
    payout: &crate::modules::rewards::repo::PayoutSnapshot,
    treasury: &str,
) -> bool {
    if transfer.fee_payer != treasury {
        return false;
    }
    let memo = payout_memo(&payout.id.to_string());
    if transfer.memo.as_deref() != Some(memo.as_str()) {
        return false;
    }
    let matches: Vec<&TransferLeg> = transfer
        .transfers
        .iter()
        .filter(|t| {
            t.source == treasury
                && t.destination == payout.solver_wallet
                && t.lamports == payout.amount_lamports
        })
        .collect();
    matches.len() == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::solana::gateway::TransferOutcome;

    #[tokio::test]
    async fn skips_broadcast_when_disabled() {
        let worker = PayoutWorker::disabled();
        assert!(!worker.broadcast_enabled());
        worker.nudge().await;
    }

    #[tokio::test]
    async fn tick_interval_constant() {
        assert_eq!(TICK_MS, 15_000);
    }

    #[test]
    fn payout_memo_format() {
        assert_eq!(
            payout_memo("11111111-1111-4111-8111-111111111111"),
            "sb:payout:11111111-1111-4111-8111-111111111111"
        );
    }

    #[tokio::test]
    async fn gateway_mock_blockhash() {
        let gw = MockSolanaGateway::default();
        let hash = gw.recent_blockhash().await;
        assert!(hash.is_some());
        assert_eq!(hash.unwrap().len(), 32);
    }

    #[tokio::test]
    async fn lookup_default_not_found() {
        let gw = MockSolanaGateway::default();
        let lookup = gw.lookup("sig").await;
        assert_eq!(lookup.outcome, TransferOutcome::NotFound);
    }

    #[tokio::test]
    async fn broadcast_gate_blocks_rpc_when_off() {
        let config = Arc::new(AppConfig {
            node_env: crate::config::NodeEnv::Test,
            port: 3001,
            database_url: Some("postgresql://example".into()),
            cors_origins: vec![],
            json_body_limit_bytes: 65536,
            auth_token_secret: None,
            answer_encryption_key: None,
            treasury_secret_key: None,
            treasury_public_key: None,
            payout_broadcast_enabled: false,
            admin_wallet_allowlist: vec![],
            solana_rpc_url: None,
            login_domain: "localhost".into(),
            login_uri: "http://localhost".into(),
        });
        let gateway = Arc::new(MockSolanaGateway::default());
        let worker = PayoutWorker::new(
            Some(PgPool::connect_lazy("postgresql://invalid").unwrap()),
            config,
        )
        .with_gateway(gateway.clone());
        worker.nudge().await;
        assert_eq!(
            gateway
                .blockhash_reads
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }
}
