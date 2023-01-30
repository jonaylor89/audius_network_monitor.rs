use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub struct ContentNode {
    pub endpoint: String,
    pub spid: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletClockPair {
    pub wallet_public_key: String,
    pub clock: i32,
}
