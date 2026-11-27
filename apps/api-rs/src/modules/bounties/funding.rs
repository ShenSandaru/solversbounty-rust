pub fn normalize_funding_purpose(segment: &str) -> Option<&'static str> {
    match segment {
        "reward-pool" => Some("reward_pool"),
        "commission" => Some("commission"),
        _ => None,
    }
}