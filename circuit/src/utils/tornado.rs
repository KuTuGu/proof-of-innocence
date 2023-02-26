mod typ;

use anyhow::{anyhow, Result};
use hex::FromHex;
use js_sys::{BigInt, Uint8Array};
use regex::Regex;
pub use typ::*;

#[derive(Debug, PartialEq)]
pub struct Note {
    currency: String,
    amount: String,
    net_id: usize,
    nullifier_hash: BigInt,
    commitment_hash: BigInt,
}

pub struct Tornado {
    note_list: Vec<Note>,
    config: NetIdConfigMap,
}

impl Tornado {
    pub async fn new(list: Vec<&str>) -> Result<Self> {
        let tornado_util = TornadoUtil::new();
        tornado_util.init().await;

        let mut note_list = vec![];
        for note in list {
            let re = Regex::new(NOTE_REGEX).unwrap();
            let caps = re
                .captures(note)
                .ok_or(anyhow!("Tornado note format is incorrect"))?;

            let currency = caps.name("currency").unwrap().as_str().into();
            let amount = caps.name("amount").unwrap().as_str().into();
            let net_id = caps.name("netId").unwrap().as_str().parse().unwrap();
            let note = <[u8; 62]>::from_hex(caps.name("note").unwrap().as_str())?;
            let preimage = Uint8Array::from(&note[..]);
            let nullifier = Uint8Array::from(&note[..31]);

            let commitment_hash = tornado_util.pedersen_hash(preimage);
            let nullifier_hash = tornado_util.pedersen_hash(nullifier);

            note_list.push(Note {
                currency,
                amount,
                net_id,
                commitment_hash,
                nullifier_hash,
            })
        }

        Ok(Self {
            note_list,
            config: serde_json::from_str(CONFIG).unwrap(),
        })
    }

    pub async fn check_nullifier(&self, ind: Option<u8>) -> Result<bool> {
        todo!()
    }

    pub async fn check_block(&self, ind: Option<u8>) -> Result<bool> {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    // https://goerli.etherscan.io/tx/0x06e10a9ea49183e9127fb7581d4d54750290c1ecc7c7f1707953f706fe9ab959
    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const NULLIFIER_HASH: &str =
        "20454790225299856478795038962746880644063954504776542630455482831220240995363";
    const COMMITMENT_HASH: &str =
        "18716593830613547391848516730128799739861563597347942888893803512076728965848";

    #[wasm_bindgen_test]
    async fn test_parse_note() {
        let tornado = Tornado::new(vec![NOTE]).await.unwrap();
        assert_eq!(
            tornado.note_list[0],
            Note {
                currency: "eth".into(),
                amount: "0.1".into(),
                net_id: 5,
                nullifier_hash: JsValue::bigint_from_str(NULLIFIER_HASH).into(),
                commitment_hash: JsValue::bigint_from_str(COMMITMENT_HASH).into(),
            }
        );
    }
}
