#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LedgerBuckets {
    pub funded: i64,
    pub reserved: i64,
    pub paid: i64,
    pub refundable: i64,
    pub refunded: i64,
}

#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub entry_type: String,
    pub amount_lamports: i64,
}

pub fn deposit_reference(bounty_id: &str, purpose: &str) -> String {
    format!("sb:deposit:{bounty_id}:{purpose}")
}

pub fn empty_buckets() -> LedgerBuckets {
    LedgerBuckets::default()
}

pub fn buckets_from_entries(entries: &[LedgerEntry]) -> LedgerBuckets {
    let mut buckets = empty_buckets();
    for entry in entries {
        match entry.entry_type.as_str() {
            "deposit" => {
                buckets.funded += entry.amount_lamports;
                buckets.refundable += entry.amount_lamports;
            }
            "reserve" => {
                buckets.reserved += entry.amount_lamports;
                buckets.refundable -= entry.amount_lamports;
            }
            "settle" => {
                buckets.reserved -= entry.amount_lamports;
                buckets.paid += entry.amount_lamports;
            }
            "release" => {
                buckets.reserved -= entry.amount_lamports;
                buckets.refundable += entry.amount_lamports;
            }
            "refund" => {
                buckets.refundable -= entry.amount_lamports;
                buckets.refunded += entry.amount_lamports;
            }
            _ => {}
        }
    }
    buckets
}

pub fn buckets_balance(buckets: &LedgerBuckets) -> bool {
    buckets.funded == buckets.reserved + buckets.paid + buckets.refundable + buckets.refunded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reward_pool_deposit_identity() {
        let buckets = buckets_from_entries(&[LedgerEntry {
            entry_type: "deposit".into(),
            amount_lamports: 1_000_000_000,
        }]);
        assert_eq!(buckets.funded, 1_000_000_000);
        assert_eq!(buckets.refundable, 1_000_000_000);
        assert_eq!(buckets.paid, 0);
        assert!(buckets_balance(&buckets));
    }
}
