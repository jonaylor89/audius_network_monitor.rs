use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default)]
pub struct ContentNode {
    pub endpoint: String,
    pub spid: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletClockPair {
    pub walletPublicKey: String,
    pub clock: i32,
}