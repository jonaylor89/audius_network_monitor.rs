use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::WalletClockPair;

const unhealthy_time_range_ms: i32 = 300_000; // 5min

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
    let client = reqwest::Client::new();
    let res = client.post(url).json(payload).send().await?;

    let js = res.json::<WalletBatchResponse>().await?;

    let wallet_batch: Vec<WalletClockPair> = serde_json::from_str(&js.data)?;

    Ok(wallet_batch)
}
