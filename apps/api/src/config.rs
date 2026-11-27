use std::env;

use url::Url;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub node_env: NodeEnv,
    pub port: u16,
    pub database_url: Option<String>,
    pub cors_origins: Vec<String>,
    pub json_body_limit_bytes: usize,
    pub auth_token_secret: Option<String>,
    pub answer_encryption_key: Option<String>,
    pub treasury_secret_key: Option<String>,
    pub treasury_public_key: Option<String>,
    pub payout_broadcast_enabled: bool,
    pub admin_wallet_allowlist: Vec<String>,
    pub solana_rpc_url: Option<String>,
    pub login_domain: String,
    pub login_uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeEnv {
    Development,
    Test,
    Production,
}

impl NodeEnv {
    pub fn secure_cookies(self) -> bool {
        matches!(self, NodeEnv::Production)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),
}

pub fn load_from_env() -> Result<AppConfig, ConfigError> {
    let node_env = parse_node_env(env_var("NODE_ENV").as_deref())?;
    let port: u16 = env_var("PORT")
        .unwrap_or_else(|| "3001".into())
        .parse()
        .map_err(|_| ConfigError::Message("PORT must be a positive integer.".into()))?;
    if port == 0 {
        return Err(ConfigError::Message(
            "PORT must be a positive integer.".into(),
        ));
    }

    let database_url = blank_to_none(env_var("DATABASE_URL"));
    if node_env == NodeEnv::Production && database_url.is_none() {
        return Err(ConfigError::Message(
            "DATABASE_URL is required when NODE_ENV is production.".into(),
        ));
    }

    let cors_raw = env_var("CORS_ORIGIN").unwrap_or_else(|| "http://localhost:3000".into());
    let cors_origins = parse_cors_origins(&cors_raw);
    validate_cors_origins(&cors_origins)?;

    let login_origin = cors_origins
        .first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:3000".into());
    let (login_domain, login_uri) = login_origin_from_url(&login_origin)?;

    let json_body_limit_bytes =
        parse_body_limit(env_var("JSON_BODY_LIMIT").as_deref().unwrap_or("64kb"));

    let payout_broadcast_enabled =
        parse_bool_env(env_var("PAYOUT_BROADCAST_ENABLED").as_deref()).unwrap_or(false);

    Ok(AppConfig {
        node_env,
        port,
        database_url,
        cors_origins,
        json_body_limit_bytes,
        auth_token_secret: blank_to_none(env_var("AUTH_TOKEN_SECRET")),
        answer_encryption_key: blank_to_none(env_var("ANSWER_ENCRYPTION_KEY")),
        treasury_secret_key: blank_to_none(env_var("TREASURY_SECRET_KEY")),
        treasury_public_key: blank_to_none(env_var("TREASURY_PUBLIC_KEY")),
        payout_broadcast_enabled,
        admin_wallet_allowlist: parse_allowlist(env_var("ADMIN_WALLET_ALLOWLIST").as_deref()),
        solana_rpc_url: blank_to_none(env_var("SOLANA_RPC_URL")),
        login_domain,
        login_uri,
    })
}

pub fn validate_cors_origins(origins: &[String]) -> Result<(), ConfigError> {
    if origins.is_empty() || origins.iter().any(|o| o == "*" || o.contains('*')) {
        return Err(ConfigError::Message(
            "CORS_ORIGIN must list explicit origins when session cookies are used.".into(),
        ));
    }
    Ok(())
}

fn login_origin_from_url(origin: &str) -> Result<(String, String), ConfigError> {
    let url = Url::parse(origin).map_err(|_| {
        ConfigError::Message(format!("CORS_ORIGIN entry is not a valid URL: {origin}"))
    })?;
    let domain = url.host_str().unwrap_or("").to_string();
    let uri = url.origin().ascii_serialization();
    Ok((domain, uri))
}

pub fn parse_cors_origins(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(String::from)
        .collect()
}

pub fn parse_allowlist(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(String::from)
        .collect()
}

fn parse_node_env(raw: Option<&str>) -> Result<NodeEnv, ConfigError> {
    match raw.unwrap_or("development") {
        "development" => Ok(NodeEnv::Development),
        "test" => Ok(NodeEnv::Test),
        "production" => Ok(NodeEnv::Production),
        other => Err(ConfigError::Message(format!("Invalid NODE_ENV: {other}"))),
    }
}

fn parse_bool_env(raw: Option<&str>) -> Option<bool> {
    let v = raw?.trim().to_lowercase();
    if v.is_empty() {
        return None;
    }
    match v.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_body_limit(raw: &str) -> usize {
    let trimmed = raw.trim().to_lowercase();
    if let Some(kb) = trimmed.strip_suffix("kb") {
        if let Ok(n) = kb.parse::<usize>() {
            return n * 1024;
        }
    }
    if let Ok(n) = trimmed.parse::<usize>() {
        return n;
    }
    64 * 1024
}

fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn blank_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

// Re-export auth timing constants for callers
pub const APP_NAME: &str = "SolversBounty";

pub mod reexport_auth {
    pub use super::super::modules::auth::constants::{
        CSRF_COOKIE, CSRF_HEADER, LOGIN_CHAIN, NONCE_RATE_LIMIT, NONCE_RATE_WINDOW_MS,
        NONCE_TTL_MS, SESSION_COOKIE, SESSION_TTL_MS,
    };
}

#[cfg(test)]
mod tests {
    use super::{parse_cors_origins, validate_cors_origins};

    #[test]
    fn rejects_star_origin() {
        let origins = parse_cors_origins("*");
        assert!(validate_cors_origins(&origins).is_err());
    }

    #[test]
    fn rejects_wildcard_subdomain() {
        let origins = parse_cors_origins("https://*.example.com");
        assert!(validate_cors_origins(&origins).is_err());
    }

    #[test]
    fn accepts_explicit_origin() {
        let origins = parse_cors_origins("http://localhost:3000");
        assert!(validate_cors_origins(&origins).is_ok());
    }
}
