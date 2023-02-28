use js_sys::{BigInt, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/output/tornado_bundle.js")]
extern "C" {
    pub type TornadoUtil;

    #[wasm_bindgen(constructor)]
    pub fn new() -> TornadoUtil;

    #[wasm_bindgen(method)]
    pub async fn init(this: &TornadoUtil);

    #[wasm_bindgen(method)]
    pub fn pedersen_hash(this: &TornadoUtil, data: Uint8Array) -> BigInt;

    #[wasm_bindgen(method, catch)]
    pub async fn read_file(this: &TornadoUtil, path: JsValue) -> Result<JsValue, JsValue>;
}

// tornado note parse rule
pub const NOTE_REGEX: &str =
    r"^tornado-(?P<currency>\w+)-(?P<amount>[\d.]+)-(?P<netId>\d+)-0x(?P<note>[0-9a-fA-F]{124})$";
// tornado event log cache file path, only cache data used for convenience
// env::var("EVENT_LOG_DIR")?
pub const EVENT_LOG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tornado_cli/cache");
// net id -> cache dir name
pub const NET_NAME_MAPPING: [(&str, &str); 8] = [
    ("netId1", "ethereum"),
    ("netId5", "goerli"),
    ("netId56", "binancesmartchain"),
    ("netId100", "gnosischain"),
    ("netId137", "polygon"),
    ("netId42161", "arbitrum"),
    ("netId43114", "avalanche"),
    ("netId10", "optimism"),
];

pub type Address = String;
pub type Hash = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventLogType {
    #[serde(rename = "deposits")]
    Deposit,
    #[serde(rename = "withdrawals")]
    Withdrawal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventLog {
    Deposit(DepositLog),
    Withdraw(WithdrawLog),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositLog {
    #[serde(default)]
    pub block_number: u32,
    #[serde(default)]
    pub transaction_hash: Hash,
    #[serde(default)]
    pub commitment: Hash,
    #[serde(default)]
    pub leaf_index: u32,
    #[serde(default)]
    pub timestamp: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawLog {
    #[serde(default)]
    pub block_number: u32,
    #[serde(default)]
    pub transaction_hash: Hash,
    #[serde(default)]
    pub nullifier_hash: Hash,
    #[serde(default)]
    pub to: Address,
    #[serde(default)]
    pub fee: String,
}
