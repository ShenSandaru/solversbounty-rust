use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct TransferLeg {
    pub source: String,
    pub destination: String,
    pub lamports: i64,
}

#[derive(Debug, Clone)]
pub struct ObservedTransfer {
    pub cluster: String,
    pub commitment: String,
    pub failed: bool,
    pub fee_payer: String,
    pub memo: Option<String>,
    pub transfers: Vec<TransferLeg>,
}

#[derive(Debug, Clone)]
pub struct TransferLookup {
    pub outcome: TransferOutcome,
    pub transfer: Option<ObservedTransfer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOutcome {
    Unavailable,
    NotFound,
    Observed,
}

#[async_trait]
pub trait SolanaGateway: Send + Sync {
    async fn recent_blockhash(&self) -> Option<String>;
    async fn submit(&self, signed_tx: &[u8]) -> bool;
    async fn lookup(&self, signature: &str) -> TransferLookup;
}

pub struct MockSolanaGateway {
    pub blockhash_reads: std::sync::atomic::AtomicUsize,
    pub submits: std::sync::atomic::AtomicUsize,
    pub lookup_result: std::sync::Mutex<TransferLookup>,
}

impl Default for MockSolanaGateway {
    fn default() -> Self {
        Self {
            blockhash_reads: std::sync::atomic::AtomicUsize::new(0),
            submits: std::sync::atomic::AtomicUsize::new(0),
            lookup_result: std::sync::Mutex::new(TransferLookup {
                outcome: TransferOutcome::NotFound,
                transfer: None,
            }),
        }
    }
}

#[async_trait]
impl SolanaGateway for MockSolanaGateway {
    async fn recent_blockhash(&self) -> Option<String> {
        let n = self
            .blockhash_reads
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Some(format!("{n}1111111111111111111111111111111")[..32].to_string())
    }

    async fn submit(&self, _signed_tx: &[u8]) -> bool {
        self.submits
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        true
    }

    async fn lookup(&self, _signature: &str) -> TransferLookup {
        self.lookup_result.lock().unwrap().clone()
    }
}
