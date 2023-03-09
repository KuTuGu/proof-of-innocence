use js_sys::{BigInt, Uint8Array};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

lazy_static! {
    pub static ref NET_NAME_MAP: HashMap<u32, &'static str> = {
        let mut map = HashMap::new();
        map.insert(1, "ethereum");
        map.insert(5, "goerli");
        map.insert(56, "binancesmartchain");
        map.insert(100, "gnosischain");
        map.insert(137, "polygon");
        map.insert(42161, "arbitrum");
        map.insert(43114, "avalanche");
        map.insert(10, "optimism");
        map
    };
}

pub type Address = String;
pub type HashStr = String;
pub type Hash = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    pub commitment: Hash,
    pub accuracy_tree_root: Hash,
    pub innocence_tree_root: Hash,
    pub accuracy_proof_element: Vec<Hash>,
    pub accuracy_proof_index: Vec<bool>,
    pub innocence_proof: Vec<u8>,
}

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
    pub leaf_index: usize,
    #[serde(default)]
    pub transaction_hash: HashStr,
    #[serde(default)]
    pub commitment: HashStr,
    #[serde(default)]
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawLog {
    #[serde(default)]
    pub block_number: u32,
    #[serde(default)]
    pub transaction_hash: HashStr,
    #[serde(default)]
    pub nullifier_hash: HashStr,
    #[serde(default)]
    pub to: Address,
    #[serde(default)]
    pub fee: String,
}
