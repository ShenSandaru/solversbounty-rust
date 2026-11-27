#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityReason {
    PoolExhausted,
    InsufficientRefundable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityDecision {
    Reserve,
    Reject(CapacityReason),
}

pub fn capacity_decision(
    allocation_count: i32,
    max_rewarded_solvers: i32,
    refundable: i64,
    reward_lamports: i64,
) -> CapacityDecision {
    if allocation_count >= max_rewarded_solvers {
        return CapacityDecision::Reject(CapacityReason::PoolExhausted);
    }
    if refundable < reward_lamports {
        return CapacityDecision::Reject(CapacityReason::InsufficientRefundable);
    }
    CapacityDecision::Reserve
}

impl CapacityReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PoolExhausted => "pool_exhausted",
            Self::InsufficientRefundable => "insufficient_refundable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_and_refundable_checks() {
        assert_eq!(
            capacity_decision(1, 1, 100, 10),
            CapacityDecision::Reject(CapacityReason::PoolExhausted)
        );
        assert_eq!(
            capacity_decision(0, 2, 5, 10),
            CapacityDecision::Reject(CapacityReason::InsufficientRefundable)
        );
        assert_eq!(capacity_decision(0, 2, 10, 10), CapacityDecision::Reserve);
    }
}
