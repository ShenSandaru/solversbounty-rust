use chrono::{DateTime, Utc};

use super::constants::LOGIN_CHAIN;

#[derive(Debug, Clone)]
pub struct LoginMessageInput {
    pub domain: String,
    pub uri: String,
    pub wallet_address: String,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub fn build_login_message(input: &LoginMessageInput) -> String {
    [
        "SolversBounty login",
        &format!("Wallet: {}", input.wallet_address),
        "This signature does not approve a SOL transfer.",
        &format!("Domain: {}", input.domain),
        &format!("URI: {}", input.uri),
        &format!("Nonce: {}", input.nonce),
        &format!(
            "Issued At: {}",
            input
                .issued_at
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        ),
        &format!(
            "Expiration Time: {}",
            input
                .expires_at
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        ),
        &format!("Chain: {LOGIN_CHAIN}"),
    ]
    .join("\n")
}

pub fn login_origin(configured_origin: &str) -> Result<(String, String), String> {
    let url = url::Url::parse(configured_origin).map_err(|e| e.to_string())?;
    let domain = url.host_str().unwrap_or("").to_string();
    let uri = url.origin().ascii_serialization();
    Ok((domain, uri))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn binds_wallet_nonce_and_devnet() {
        let message = build_login_message(&LoginMessageInput {
            domain: "localhost:3000".into(),
            uri: "http://localhost:3000".into(),
            wallet_address: "wallet".into(),
            nonce: "nonce-1".into(),
            issued_at: Utc.with_ymd_and_hms(2026, 11, 7, 12, 0, 0).unwrap(),
            expires_at: Utc.with_ymd_and_hms(2026, 11, 7, 12, 10, 0).unwrap(),
        });
        assert!(message.contains("SolversBounty login"));
        assert!(!message.contains("wants you to sign in with your Solana account"));
        assert!(message.contains("This signature does not approve a SOL transfer."));
        assert!(message.contains("Domain: localhost:3000"));
        assert!(message.contains("URI: http://localhost:3000"));
        assert!(message.contains("wallet"));
        assert!(message.contains("Nonce: nonce-1"));
        assert!(message.contains("Chain: solana:devnet"));
        assert!(!message.contains("mainnet"));
    }
}
