//! Treasury key decode and SystemProgram transfer + Memo payout signing.
//! Builds legacy Solana transaction bytes (Nest-compatible intent: one transfer + memo).

use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

/// SPL Memo program id base58 (MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr).
const MEMO_PROGRAM_ID_B58: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

const SYSTEM_PROGRAM_ID: [u8; 32] = [0u8; 32];

fn memo_program_id() -> Option<[u8; 32]> {
    pubkey_from_base58(MEMO_PROGRAM_ID_B58)
}

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
    let treasury_bytes = signing.verifying_key().to_bytes();
    let treasury = bs58::encode(treasury_bytes).into_string();
    if treasury != expected {
        return None;
    }
    let destination = pubkey_from_base58(input.destination)?;
    if input.lamports <= 0 {
        return None;
    }
    let lamports = input.lamports as u64;
    let blockhash = pubkey_from_base58(input.recent_blockhash)?;
    let memo_program = memo_program_id()?;
    let memo_bytes = input.memo.as_bytes();
    if memo_bytes.is_empty() || memo_bytes.len() > 566 {
        return None;
    }

    // Account keys: treasury (signer+writable), destination (writable), system, memo.
    let account_keys: [[u8; 32]; 4] =
        [treasury_bytes, destination, SYSTEM_PROGRAM_ID, memo_program];

    let transfer_data = {
        let mut data = Vec::with_capacity(12);
        data.extend_from_slice(&2u32.to_le_bytes()); // SystemInstruction::Transfer
        data.extend_from_slice(&lamports.to_le_bytes());
        data
    };

    // Compiled instructions: program_id_index, accounts, data
    let mut message = Vec::with_capacity(256);
    // header: 1 required signature, 0 readonly signed, 2 readonly unsigned (system + memo)
    message.push(1);
    message.push(0);
    message.push(2);
    push_short_vec_len(&mut message, account_keys.len());
    for key in &account_keys {
        message.extend_from_slice(key);
    }
    message.extend_from_slice(&blockhash);
    push_short_vec_len(&mut message, 2); // two instructions

    // SystemProgram.transfer: accounts [treasury, destination]
    message.push(2); // program_id_index = system
    push_short_vec_len(&mut message, 2);
    message.push(0);
    message.push(1);
    push_short_vec_len(&mut message, transfer_data.len());
    message.extend_from_slice(&transfer_data);

    // Memo: accounts [treasury] as signer reference (optional but Nest-compatible)
    message.push(3); // program_id_index = memo
    push_short_vec_len(&mut message, 1);
    message.push(0);
    push_short_vec_len(&mut message, memo_bytes.len());
    message.extend_from_slice(memo_bytes);

    let signature = signing.sign(&message);
    let sig_bytes = signature.to_bytes();
    let signature_b58 = bs58::encode(sig_bytes).into_string();

    let mut serialized = Vec::with_capacity(1 + 64 + message.len());
    push_short_vec_len(&mut serialized, 1);
    serialized.extend_from_slice(&sig_bytes);
    serialized.extend_from_slice(&message);

    Some(SignedPayout {
        signature: signature_b58,
        serialized,
        treasury,
    })
}

pub struct SignPayoutInput<'a> {
    pub secret: &'a str,
    pub destination: &'a str,
    pub lamports: i64,
    pub memo: &'a str,
    pub recent_blockhash: &'a str,
    pub expected_treasury: &'a str,
}

fn pubkey_from_base58(value: &str) -> Option<[u8; 32]> {
    let bytes = bs58::decode(value.trim()).into_vec().ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

fn push_short_vec_len(buf: &mut Vec<u8>, len: usize) {
    // Solana shortvec / compact-u16 encoding for lengths that fit in practice.
    let mut rem = len;
    loop {
        let mut elem = (rem & 0x7f) as u8;
        rem >>= 7;
        if rem == 0 {
            buf.push(elem);
            break;
        }
        elem |= 0x80;
        buf.push(elem);
    }
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
    fn signs_system_transfer_with_memo() {
        let mut secret = vec![7u8; 64];
        secret[0] = 1;
        let hex_secret = hex::encode(&secret);
        let addr = treasury_address_from_secret(&hex_secret).expect("address");
        // Valid-looking 32-byte blockhash as base58 (all zeros is the system program id encoding).
        let blockhash = bs58::encode([2u8; 32]).into_string();
        let signed = sign_payout_transfer(SignPayoutInput {
            secret: &hex_secret,
            destination: &addr,
            lamports: 1_000,
            memo: "sb:payout:x",
            recent_blockhash: &blockhash,
            expected_treasury: &addr,
        })
        .expect("signed");
        assert!(!signed.signature.is_empty());
        assert_eq!(signed.treasury, addr);
        assert!(signed.serialized.len() > 64);
        // shortvec(1) + 64-byte sig + message header byte
        assert_eq!(signed.serialized[0], 1);
        assert_eq!(
            &signed.serialized[1..65],
            &bs58::decode(&signed.signature).into_vec().unwrap()[..]
        );
    }

    #[test]
    fn rejects_mismatched_treasury() {
        let secret = hex::encode([9u8; 64]);
        let addr = treasury_address_from_secret(&secret).unwrap();
        let blockhash = bs58::encode([3u8; 32]).into_string();
        let signed = sign_payout_transfer(SignPayoutInput {
            secret: &secret,
            destination: &addr,
            lamports: 1,
            memo: "m",
            recent_blockhash: &blockhash,
            expected_treasury: "not-the-treasury",
        });
        assert!(signed.is_none());
    }

    #[test]
    fn memo_program_id_matches_known_base58() {
        let id = memo_program_id().expect("memo id");
        assert_eq!(bs58::encode(id).into_string(), MEMO_PROGRAM_ID_B58);
        assert_eq!(id.len(), 32);
    }
}
