use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;

pub const ANSWER_KEY_ID: &str = "primary";

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct CipherError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}

pub fn parse_answer_key(value: Option<&str>) -> Result<Vec<u8>, CipherError> {
    let value = value.ok_or_else(|| CipherError {
        status: 503,
        code: "NOT_READY",
        message: "Answer protection is not configured.".into(),
    })?;
    let key = decode_key(value).map_err(|_| CipherError {
        status: 503,
        code: "NOT_READY",
        message: "Answer protection is not configured.".into(),
    })?;
    if key.len() != 32 {
        return Err(CipherError {
            status: 503,
            code: "NOT_READY",
            message: "Answer protection is not configured.".into(),
        });
    }
    Ok(key)
}

pub fn encrypt_answer(plaintext: &str, key: &[u8]) -> Result<String, CipherError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| internal())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| internal())?;
    let tag_len = 16;
    if ciphertext.len() < tag_len {
        return Err(internal());
    }
    let (ct, tag) = ciphertext.split_at(ciphertext.len() - tag_len);
    Ok(format!(
        "v1.{}.{}.{}",
        URL_SAFE_NO_PAD.encode(nonce_bytes),
        URL_SAFE_NO_PAD.encode(ct),
        URL_SAFE_NO_PAD.encode(tag)
    ))
}

pub fn decrypt_answer(payload: &str, key: &[u8]) -> Result<String, CipherError> {
    let parts: Vec<&str> = payload.split('.').collect();
    if parts.len() != 4 || parts[0] != "v1" || parts[1].is_empty() || parts[2].is_empty() || parts[3].is_empty() {
        return Err(internal());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| internal())?;
    let nonce_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| internal())?;
    let ct = URL_SAFE_NO_PAD.decode(parts[2]).map_err(|_| internal())?;
    let tag = URL_SAFE_NO_PAD.decode(parts[3]).map_err(|_| internal())?;
    let mut combined = ct;
    combined.extend_from_slice(&tag);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plain = cipher.decrypt(nonce, combined.as_ref()).map_err(|_| internal())?;
    String::from_utf8(plain).map_err(|_| internal())
}

fn decode_key(value: &str) -> Result<Vec<u8>, ()> {
    if value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex::decode(value).map_err(|_| ());
    }
    URL_SAFE_NO_PAD.decode(value).map_err(|_| ())
}

fn internal() -> CipherError {
    CipherError {
        status: 500,
        code: "INTERNAL",
        message: "Internal server error".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_key() {
        let err = parse_answer_key(None).unwrap_err();
        assert_eq!(err.status, 503);
        assert_eq!(err.code, "NOT_READY");
    }

    #[test]
    fn round_trip_base64url_key() {
        let key = parse_answer_key(Some(&URL_SAFE_NO_PAD.encode([7u8; 32]))).unwrap();
        let payload = encrypt_answer("door-A", &key).unwrap();
        assert!(payload.starts_with("v1."));
        assert!(!payload.contains("door-A"));
        assert_eq!(decrypt_answer(&payload, &key).unwrap(), "door-A");
    }
}