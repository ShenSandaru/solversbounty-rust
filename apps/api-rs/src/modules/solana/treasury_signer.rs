//! Treasury key decode and payout memo helpers.
//! Full SystemProgram+Memo transaction bytes match Nest when a signer backend is configured.
//! Without an external Solana transaction builder, `sign_payout_transfer` returns `None`
//! after validating the key material (same failure Nest uses when signing cannot proceed).

use ed25519_dalek::SigningKey;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SignedPayout {
    pub signature: String,
    pub serialized: Vec<u8>,
    pub treasury: String,
}

pub fn payout_memo(payout_id: &str) -> String {
    format!("sb:payout:{payout_id}")
}

pub fn sign_payout_transfer(input: SignPayoutInput<'_>) -> Option<SignedPayout> {
    let expected = input.expected_treasury.trim();
    if expected.is_empty() {
        return None;
    }
    let secret = decode_secret(input.secret)?;
    if secret.len() != 64 {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&secret[..32]);
    let signing = SigningKey::from_bytes(&seed);
    let treasury = bs58::encode(signing.verifying_key().as_bytes()).into_string();
    if treasury != expected {
        return None;
    }
    let _destination = bs58::decode(input.destination).into_vec().ok()?;
    if input.lamports <= 0 {
        return None;
    }
    // Nest builds SystemProgram.transfer + MemoSq4… with solana web3.js.
    // Transaction construction is delegated to the gateway layer when available.
    // Returning None keeps the worker on the Nest-compatible "could not be signed" path
    // until a Solana tx builder crate is adopted for broadcast.
    let _ = (input.memo, input.recent_blockhash);
    None
}

pub struct SignPayoutInput<'a> {
    pub secret: &'a str,
    pub destination: &'a str,
    pub lamports: i64,
    pub memo: &'a str,
    pub recent_blockhash: &'a str,
    pub expected_treasury: &'a str,
}

pub fn decode_secret(secret: &str) -> Option<Vec<u8>> {
    let trimmed = secret.trim();
    if trimmed.starts_with('[') {
        let parsed: Value = serde_json::from_str(trimmed).ok()?;
        let arr = parsed.as_array()?;
        if arr.len() != 64 {
            return None;
        }
        let mut out = Vec::with_capacity(64);
        for item in arr {
            let n = item.as_u64()?;
            if n > 255 {
                return None;
            }
            out.push(n as u8);
        }
        return Some(out);
    }
    if trimmed.len() == 128 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex::decode(trimmed).ok();
    }
    if let Ok(bytes) = bs58::decode(trimmed).into_vec() {
        if bytes.len() == 64 {
            return Some(bytes);
        }
    }
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let bytes = STANDARD.decode(trimmed).ok()?;
    if bytes.len() == 64 {
        Some(bytes)
    } else {
        None
    }
}

pub fn treasury_address_from_secret(secret: &str) -> Option<String> {
    let bytes = decode_secret(secret)?;
    if bytes.len() != 64 {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes[..32]);
    let signing = SigningKey::from_bytes(&seed);
    Some(bs58::encode(signing.verifying_key().as_bytes()).into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_matches_nest() {
        assert_eq!(
            payout_memo("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"),
            "sb:payout:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn decodes_hex_secret_and_derives_address() {
        let mut secret = vec![7u8; 64];
        secret[0] = 1;
        let hex_secret = hex::encode(&secret);
        let addr = treasury_address_from_secret(&hex_secret).expect("address");
        assert!(!addr.is_empty());
        let signed = sign_payout_transfer(SignPayoutInput {
            secret: &hex_secret,
            destination: &addr,
            lamports: 1,
            memo: "sb:payout:x",
            recent_blockhash: "11111111111111111111111111111111",
            expected_treasury: &addr,
        });
        assert!(signed.is_none());
    }

    #[test]
    fn rejects_mismatched_treasury() {
        let secret = hex::encode([9u8; 64]);
        let addr = treasury_address_from_secret(&secret).unwrap();
        let signed = sign_payout_transfer(SignPayoutInput {
            secret: &secret,
            destination: &addr,
            lamports: 1,
            memo: "m",
            recent_blockhash: "1",
            expected_treasury: "not-the-treasury",
        });
        assert!(signed.is_none());
    }
}
