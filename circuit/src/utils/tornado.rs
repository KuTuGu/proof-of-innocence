use anyhow::{anyhow, Result};
use hex::FromHex;
use js_sys::{BigInt, Uint8Array};
use regex::Regex;
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

const NOTE_REGEX: &str =
    r"^tornado-(?P<currency>\w+)-(?P<amount>[\d.]+)-(?P<netId>\d+)-0x(?P<note>[0-9a-fA-F]{124})$";

#[derive(Debug, PartialEq)]
pub struct Note {
    currency: String,
    amount: String,
    net_id: usize,
    nullifier_hash: BigInt,
    commitment_hash: BigInt,
}

pub async fn parse_note(note: &str) -> Result<Note> {
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

    let tornado_util = TornadoUtil::new();
    tornado_util.init().await;
    let commitment_hash = tornado_util.pedersen_hash(preimage);
    let nullifier_hash = tornado_util.pedersen_hash(nullifier);

    Ok(Note {
        currency,
        amount,
        net_id,
        commitment_hash,
        nullifier_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // https://goerli.etherscan.io/tx/0x7b5ee6c14b86509c2b401ee1ec15657f303494acfe5d786cc4081a6666f34414
    const NOTE: &str = r"tornado-eth-0.1-5-0x4805479a68a261e0850509d4a0724877c9395be42d78146b05880d7fd4b9484e92c8de0dfc2df89aae1a7d87726da32eed131fde50bff26a0392ce2b6729";
    const NULLIFIER_HASH: &str =
        "18941337381714858446355925653430800737045061541595044917820195410400865861385";
    const COMMITMENT_HASH: &str =
        "525964881243906792375230974931225736378364691955935045809786306735191636140";

    #[wasm_bindgen_test]
    async fn test_parse_note() {
        let note = parse_note(NOTE).await.unwrap();
        assert_eq!(
            note,
            Note {
                currency: "eth".into(),
                amount: "0.1".into(),
                net_id: 5,
                nullifier_hash: JsValue::bigint_from_str(NULLIFIER_HASH).into(),
                commitment_hash: JsValue::bigint_from_str(COMMITMENT_HASH).into(),
            }
        )
    }
}
