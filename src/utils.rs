use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{collections::HashMap};

use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use crate::domain::WalletClockPair;

// const UNHEALTHY_TIME_RANGE_MS: i32 = 300_000; // 5min

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
) -> eyre::Result<Vec<WalletClockPair>> {
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
) -> eyre::Result<Vec<WalletClockPair>> {
    let client = reqwest::Client::new();
    let res = client.post(url).json(payload).send().await?;

    let js = res.json::<WalletBatchResponse>().await?;

    let wallet_batch: Vec<WalletClockPair> = serde_json::from_str(&js.data)?;

    Ok(wallet_batch)
}
