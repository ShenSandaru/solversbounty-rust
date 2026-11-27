use std::collections::HashMap;

pub struct InMemoryRateLimit {
    hits: HashMap<String, Vec<i64>>,
    now: fn() -> i64,
}

impl InMemoryRateLimit {
    pub fn new() -> Self {
        Self {
            hits: HashMap::new(),
            now: || chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_clock(now: fn() -> i64) -> Self {
        Self {
            hits: HashMap::new(),
            now,
        }
    }

    pub fn allow(&mut self, key: &str, limit: u32, window_ms: i64) -> bool {
        let current = (self.now)();
        let recent: Vec<i64> = self
            .hits
            .get(key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|time| current - time < window_ms)
            .collect();
        if recent.len() >= limit as usize {
            self.hits.insert(key.to_string(), recent);
            return false;
        }
        let mut updated = recent;
        updated.push(current);
        self.hits.insert(key.to_string(), updated);
        true
    }
}

impl Default for InMemoryRateLimit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_blocks_and_resets() {
        static mut NOW: i64 = 1_000;
        fn clock() -> i64 {
            unsafe { NOW }
        }
        let mut limiter = InMemoryRateLimit::with_clock(clock);
        assert!(limiter.allow("wallet", 2, 60_000));
        assert!(limiter.allow("wallet", 2, 60_000));
        assert!(!limiter.allow("wallet", 2, 60_000));
        unsafe { NOW += 60_001 };
        assert!(limiter.allow("wallet", 2, 60_000));
    }
}
