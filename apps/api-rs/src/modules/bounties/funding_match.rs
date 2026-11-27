use crate::modules::solana::gateway::ObservedTransfer;

pub fn match_obligation(
    transfer: &ObservedTransfer,
    wallet: &str,
    treasury: &str,
    expected: i64,
    reference: &str,
) -> Option<(String, String, i64)> {
    if transfer.memo.as_deref() != Some(reference) || transfer.fee_payer != wallet {
        return None;
    }
    let matches: Vec<_> = transfer
        .transfers
        .iter()
        .filter(|t| t.source == wallet && t.destination == treasury && t.lamports == expected)
        .collect();
    if matches.len() == 1 {
        let t = matches[0];
        Some((t.source.clone(), t.destination.clone(), t.lamports))
    } else {
        None
    }
}