//! Devnet JSON-RPC gateway. Mirrors Nest `RpcSolanaTransferGateway`.

use async_trait::async_trait;
use serde_json::{json, Value};

use super::gateway::{
    ObservedTransfer, SolanaGateway, TransferLeg, TransferLookup, TransferOutcome,
};

pub struct RpcSolanaGateway {
    rpc_url: String,
    client: reqwest::Client,
}

impl RpcSolanaGateway {
    pub fn new(rpc_url: String) -> Option<Self> {
        if !is_devnet_rpc(&rpc_url) {
            return None;
        }
        Some(Self {
            rpc_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .ok()?,
        })
    }

    async fn rpc_method(&self, method: &str, params: Value) -> Option<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        response.json::<Value>().await.ok()
    }
}

#[async_trait]
impl SolanaGateway for RpcSolanaGateway {
    async fn recent_blockhash(&self) -> Option<String> {
        let payload = self
            .rpc_method("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))
            .await?;
        payload
            .pointer("/result/value/blockhash")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    async fn submit(&self, signed_tx: &[u8]) -> bool {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let encoded = STANDARD.encode(signed_tx);
        let payload = self
            .rpc_method(
                "sendTransaction",
                json!([encoded, { "encoding": "base64", "preflightCommitment": "confirmed" }]),
            )
            .await;
        matches!(payload, Some(v) if v.get("result").and_then(|r| r.as_str()).is_some())
    }

    async fn lookup(&self, signature: &str) -> TransferLookup {
        let payload = match self
            .rpc_method(
                "getTransaction",
                json!([
                    signature,
                    {
                        "commitment": "confirmed",
                        "encoding": "jsonParsed",
                        "maxSupportedTransactionVersion": 0
                    }
                ]),
            )
            .await
        {
            Some(p) => p,
            None => {
                return TransferLookup {
                    outcome: TransferOutcome::Unavailable,
                    transfer: None,
                }
            }
        };
        match read_parsed_transaction(&payload, "confirmed") {
            None => TransferLookup {
                outcome: TransferOutcome::NotFound,
                transfer: None,
            },
            Some(transfer) => TransferLookup {
                outcome: TransferOutcome::Observed,
                transfer: Some(transfer),
            },
        }
    }
}

pub fn is_devnet_rpc(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    let host = parsed.host_str().unwrap_or("").to_lowercase();
    let text = format!("{}{}{}", host, parsed.path(), parsed.query().unwrap_or("")).to_lowercase();
    text.contains("devnet") && !text.contains("mainnet")
}

pub fn read_parsed_transaction(payload: &Value, commitment: &str) -> Option<ObservedTransfer> {
    let result = payload.get("result")?;
    if result.is_null() {
        return None;
    }
    let meta = result.get("meta")?;
    let message = result.pointer("/transaction/message")?;
    let fee_payer = message
        .get("accountKeys")?
        .as_array()?
        .first()?
        .get("pubkey")?
        .as_str()?
        .to_string();
    let instructions = message.get("instructions")?.as_array()?;
    Some(ObservedTransfer {
        cluster: "devnet".into(),
        commitment: commitment.into(),
        failed: !meta.get("err").map(|e| e.is_null()).unwrap_or(true),
        fee_payer,
        memo: read_memo(instructions),
        transfers: read_transfers(instructions),
    })
}

fn read_transfers(instructions: &[Value]) -> Vec<TransferLeg> {
    const SYSTEM: &str = "11111111111111111111111111111111";
    let mut out = Vec::new();
    for instruction in instructions {
        let system = instruction.get("program").and_then(|p| p.as_str()) == Some("system")
            || instruction.get("programId").and_then(|p| p.as_str()) == Some(SYSTEM);
        if !system {
            continue;
        }
        let parsed = match instruction.get("parsed") {
            Some(p) => p,
            None => continue,
        };
        if parsed.get("type").and_then(|t| t.as_str()) != Some("transfer") {
            continue;
        }
        let info = match parsed.get("info") {
            Some(i) => i,
            None => continue,
        };
        let source = match info.get("source").and_then(|s| s.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let destination = match info.get("destination").and_then(|s| s.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let lamports = match info.get("lamports") {
            Some(Value::Number(n)) => n.as_i64().filter(|v| *v >= 0),
            Some(Value::String(s)) => s.parse::<i64>().ok().filter(|v| *v >= 0),
            _ => None,
        };
        let Some(lamports) = lamports else {
            continue;
        };
        out.push(TransferLeg {
            source,
            destination,
            lamports,
        });
    }
    out
}

fn read_memo(instructions: &[Value]) -> Option<String> {
    let mut memos = Vec::new();
    for instruction in instructions {
        let program_id = instruction
            .get("programId")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        let is_memo = instruction.get("program").and_then(|p| p.as_str()) == Some("spl-memo")
            || program_id.starts_with("Memo");
        if !is_memo {
            continue;
        }
        if let Some(s) = instruction.get("parsed").and_then(|p| p.as_str()) {
            memos.push(s.to_string());
        } else if let Some(s) = instruction.pointer("/parsed/info").and_then(|p| p.as_str()) {
            memos.push(s.to_string());
        }
    }
    if memos.len() == 1 {
        memos.pop()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_devnet_host() {
        assert!(is_devnet_rpc("https://api.devnet.solana.com"));
        assert!(!is_devnet_rpc("https://api.mainnet-beta.solana.com"));
    }

    #[test]
    fn parses_null_result_as_missing() {
        let payload = json!({ "result": null });
        assert!(read_parsed_transaction(&payload, "confirmed").is_none());
    }
}
