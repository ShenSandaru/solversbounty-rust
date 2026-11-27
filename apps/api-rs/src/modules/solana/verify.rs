use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[derive(Debug, thiserror::Error)]
#[error("Wallet address is not a Solana public key.")]
pub struct WalletAddressError;

#[derive(Debug, thiserror::Error)]
#[error("Signature could not be read.")]
pub struct SignatureError;

pub fn canonical_wallet_address(value: &str) -> Result<String, WalletAddressError> {
    let decoded = bs58::decode(value)
        .into_vec()
        .map_err(|_| WalletAddressError)?;
    if decoded.len() != 32 {
        return Err(WalletAddressError);
    }
    Ok(bs58::encode(decoded).into_string())
}

pub fn decode_signature(value: &str) -> Result<Vec<u8>, SignatureError> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| SignatureError)?;
    if bytes.len() != 64 {
        return Err(SignatureError);
    }
    let encoded = URL_SAFE_NO_PAD.encode(&bytes);
    if encoded != value {
        return Err(SignatureError);
    }
    Ok(bytes)
}

pub fn verify_wallet_signature(wallet_address: &str, message: &str, signature: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let pubkey = match decode_wallet_pubkey(wallet_address) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let Ok(verifying) = VerifyingKey::from_bytes(&pubkey) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(signature) else {
        return false;
    };
    verifying.verify(message.as_bytes(), &sig).is_ok()
}

fn decode_wallet_pubkey(wallet_address: &str) -> Result<[u8; 32], WalletAddressError> {
    let decoded = bs58::decode(wallet_address)
        .into_vec()
        .map_err(|_| WalletAddressError)?;
    if decoded.len() != 32 {
        return Err(WalletAddressError);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    #[test]
    fn accepts_message_and_rejects_tamper() {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        let wallet = bs58::encode(verifying.as_bytes()).into_string();
        let address = canonical_wallet_address(&wallet).unwrap();
        let message = "SolversBounty login message";
        let signature = signing.sign(message.as_bytes()).to_bytes();
        assert!(verify_wallet_signature(&address, message, &signature));
        assert!(!verify_wallet_signature(
            &address,
            &format!("{message} tampered"),
            &signature
        ));
        let other = SigningKey::generate(&mut OsRng);
        let other_wallet = bs58::encode(other.verifying_key().as_bytes()).into_string();
        assert!(!verify_wallet_signature(&other_wallet, message, &signature));
        assert!(!verify_wallet_signature(
            &address,
            message,
            &signature[..32]
        ));
        assert!(decode_signature(&URL_SAFE_NO_PAD.encode(&signature[..32])).is_err());
    }
}
