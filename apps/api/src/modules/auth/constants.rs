pub const NONCE_TTL_MS: i64 = 10 * 60 * 1000;
pub const SESSION_TTL_MS: i64 = 8 * 60 * 60 * 1000;
pub const NONCE_RATE_LIMIT: u32 = 10;
pub const NONCE_RATE_WINDOW_MS: i64 = 60 * 1000;
pub const SESSION_COOKIE: &str = "sb_session";
pub const CSRF_COOKIE: &str = "sb_csrf";
pub const CSRF_HEADER: &str = "x-csrf-token";
pub const LOGIN_CHAIN: &str = "solana:devnet";
