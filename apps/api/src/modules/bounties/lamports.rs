pub const LAMPORTS_PER_SOL: i64 = 1_000_000_000;
pub const MAX_REWARDED_SOLVERS: i32 = 100_000;
pub const MAX_PRINCIPAL_LAMPORTS: i64 = 1_000_000 * LAMPORTS_PER_SOL;

pub fn principal_lamports(reward_lamports: i64, max_rewarded_solvers: i32) -> i64 {
    reward_lamports.saturating_mul(max_rewarded_solvers as i64)
}

pub fn commission_lamports(principal: i64) -> i64 {
    (principal as i128 * 5 / 100) as i64
}

pub fn parse_sol_to_lamports(input: &str) -> Option<i64> {
    let trimmed = input.trim();
    let (whole, fraction) = match trimmed.split_once('.') {
        Some((w, f)) => (w, f),
        None => (trimmed, ""),
    };
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !fraction.chars().all(|c| c.is_ascii_digit()) || fraction.len() > 9 {
        return None;
    }
    let whole_lamports: i64 = whole.parse().ok()?;
    let frac = format!("{fraction:0<9}");
    let frac_lamports: i64 = frac[..9].parse().ok()?;
    Some(whole_lamports.saturating_mul(LAMPORTS_PER_SOL) + frac_lamports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_five_percent_commission() {
        let pool = 1_000_000_000i64;
        let commission = commission_lamports(pool);
        assert_eq!(commission, 50_000_000);
        assert_eq!(pool + commission, 1_050_000_000);
        assert_eq!(commission_lamports(1_000_000_001), 50_000_000);
        assert_eq!(commission_lamports(0), 0);
        assert_eq!(commission_lamports(19), 0);
        assert_eq!(commission_lamports(20), 1);
    }
}
