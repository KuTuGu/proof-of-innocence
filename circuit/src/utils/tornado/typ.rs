use js_sys::{BigInt, Uint8Array};
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
}

// tornado note parse rule
pub const NOTE_REGEX: &str =
    r"^tornado-(?P<currency>\w+)-(?P<amount>[\d.]+)-(?P<netId>\d+)-0x(?P<note>[0-9a-fA-F]{124})$";
// tornado event log cache file path, only cache data used for convenience
pub const LOG_PATH: &str = r"../../../tornado_cli/cache";
// tornado contract related information
pub const CONFIG: &str = include_str!("./config.json");

pub type NetIdConfigMap = HashMap<String, NetIdConfig>;
pub type Address = String;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetIdConfig {
    pub eth: Option<TokenData>,
    pub dai: Option<TokenData>,
    pub cdai: Option<TokenData>,
    pub xdai: Option<TokenData>,
    pub usdc: Option<TokenData>,
    pub usdt: Option<TokenData>,
    pub wbtc: Option<TokenData>,
    pub matic: Option<TokenData>,
    pub bnb: Option<TokenData>,
    pub avax: Option<TokenData>,
    pub proxy: Option<Address>,
    pub multicall: Option<Address>,
    pub subgraph: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_address: Option<InstanceAddress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_block_number: Option<DeployedBlockNumber>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mining_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_address: Option<Address>,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub decimals: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAddress {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "1")]
    pub v1: Option<Address>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "10")]
    pub v10: Option<Address>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "100")]
    pub v100: Option<Address>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "0.1")]
    pub v01: Option<Address>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployedBlockNumber {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "1")]
    pub v1: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "10")]
    pub v10: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "100")]
    pub v100: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "0.1")]
    pub v01: Option<u32>,
}
