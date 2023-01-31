use ethereum_types::H256;
use once_cell::sync::Lazy;
use secp256k1::{All, Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{collections::HashMap, str::FromStr};
use tiny_keccak::{Hasher, Keccak};
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use crate::domain::WalletClockPair;

// const UNHEALTHY_TIME_RANGE_MS: i32 = 300_000; // 5min
static CONTEXT: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureParams {
    pub spid: u16,
    pub timestamp: SystemTime,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsignedParams {
    pub spid: u16,
    pub timestamp: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletBatchResponse {
    data: String,
    method: String,
    headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub wallet_public_keys: Vec<String>,
}

#[tracing::instrument]
pub async fn make_request(
    url: &str,
    payload: &UserStatusPayload,
) -> Result<Vec<WalletClockPair>, anyhow::Error> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // add jitter to delays
        .take(3); // limit to 3 retries

    let wallet_batch =
        Retry::spawn(retry_strategy, async || network_call(url, payload).await).await?;

    Ok(wallet_batch)
}

async fn network_call(
    url: &str,
    payload: &UserStatusPayload,
) -> Result<Vec<WalletClockPair>, anyhow::Error> {
    let client = reqwest::Client::new();
    let res = client.post(url).json(payload).send().await?;

    let js = res.json::<WalletBatchResponse>().await?;

    let wallet_batch: Vec<WalletClockPair> = serde_json::from_str(&js.data)?;

    Ok(wallet_batch)
}

pub async fn generate_signature_params(
    spid: u16,
    priv_key: String,
) -> Result<SignatureParams, anyhow::Error> {
    let timestamp = SystemTime::now();
    let to_sign_obj = UnsignedParams { spid, timestamp };

    let to_hash_str = serde_json::to_string(&to_sign_obj)?;
    let to_sign_hash = keccak256(to_hash_str.as_bytes());

    let mut eth_message =
        format!("\x19Ethereum Signed Message:\n{}", to_sign_hash.len(),).into_bytes();
    eth_message.extend_from_slice(&to_sign_hash);

    let message_hash = keccak256(&eth_message);

    let key = SecretKey::from_str(&priv_key)?;

    let signature = sign(key, &message_hash)?;

    let signed_response = SignatureParams {
        spid,
        timestamp,
        signature,
    };

    Ok(signed_response)
}

fn sign(key: SecretKey, message: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    
    let message = Message::from_slice(message)?;
    let (recovery_id, signature) = CONTEXT
        .sign_ecdsa_recoverable(&message, &key)
        .serialize_compact();

    let standard_v = recovery_id.to_i32() as u64;

    // convert to 'Electrum' notation.
    let v = standard_v + 27;
    let r = H256::from_slice(&signature[..32]);
    let s = H256::from_slice(&signature[32..]);

    let signature_bytes = {
        let mut bytes = Vec::with_capacity(65);
        bytes.extend_from_slice(r.as_bytes());
        bytes.extend_from_slice(s.as_bytes());
        bytes.push(v.try_into()?);
        bytes
    };

    Ok(signature_bytes)
}

/// Compute the Keccak-256 hash of input bytes.
fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}
