use super::typ::*;
use anyhow::{anyhow, Result};
use js_sys::Uint8Array;
use num_bigint::BigUint;
use num_traits::Num;
use regex::Regex;
use wasm_bindgen::JsValue;

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct Note {
    currency: String,
    amount: String,
    net_id: u32,
    nullifier_hash: HashStr,
    commitment_hash: HashStr,
}

impl Note {
    pub fn new(note: &str, util: &TornadoUtil) -> Result<Self> {
        let re = Regex::new(NOTE_REGEX)?;
        let caps = re
            .captures(note)
            .ok_or(anyhow!("Tornado note `{note}` format is incorrect"))?;

        let currency = caps.name("currency").unwrap().as_str().into();
        let amount = caps.name("amount").unwrap().as_str().into();
        let net_id = caps.name("netId").unwrap().as_str().parse()?;
        let note = BigUint::from_str_radix(caps.name("note").unwrap().as_str(), 16)
            .unwrap()
            .to_bytes_be();
        let preimage = Uint8Array::from(&note[..]);
        let nullifier = Uint8Array::from(&note[..31]);
        let commitment_hash = hash_data(preimage, util)?;
        let nullifier_hash = hash_data(nullifier, util)?;

        Ok(Self {
            currency,
            amount,
            net_id,
            commitment_hash,
            nullifier_hash,
        })
    }

    pub async fn read_event_log(
        &self,
        typ: Option<EventLogType>,
        util: &TornadoUtil,
    ) -> Result<Vec<EventLog>> {
        let net_id = self.net_id;
        let base_dir = &format!(
            "{}/{}",
            EVENT_LOG_PATH,
            NET_NAME_MAP
                .get(&net_id)
                .ok_or(anyhow!("Net#{net_id} not support"))?
        );

        match typ {
            Some(typ @ (EventLogType::Deposit | EventLogType::Withdrawal)) => {
                let content = self.read_file(util, base_dir, typ).await?;
                Ok(serde_json::from_str(&content)?)
            }
            _ => {
                let content = self
                    .read_file(util, base_dir, EventLogType::Deposit)
                    .await?;
                let deposit_list: Vec<EventLog> = serde_json::from_str(&content)?;
                let content = self
                    .read_file(util, base_dir, EventLogType::Withdrawal)
                    .await?;
                let withdraw_list: Vec<EventLog> = serde_json::from_str(&content)?;
                Ok([deposit_list, withdraw_list].concat())
            }
        }
    }

    pub fn commitment(&self) -> &HashStr {
        &self.commitment_hash
    }

    async fn read_file(
        &self,
        util: &TornadoUtil,
        base_dir: &str,
        typ: EventLogType,
    ) -> Result<String> {
        // tornado event log cache file path, only cache data used for convenience
        let path = format!(
            "{}/{}_{}_{}.json",
            base_dir,
            serde_json::to_string(&typ).unwrap().replace("\"", ""),
            self.currency,
            self.amount
        );

        Uint8Array::from(
            util.read_file(JsValue::from_str(&path))
                .await
                .map_err(|err| {
                    anyhow!("Failed to read cache file, ensure that the file `{path}` exit.{err:?}")
                })?,
        )
        .to_string()
        .as_string()
        .ok_or(anyhow!(
            "Failed to read cache file, ensure that the file format correct."
        ))
    }
}

fn hash_data(data: Uint8Array, util: &TornadoUtil) -> Result<String> {
    Ok(format!(
        "{:0>64}",
        util.pedersen_hash(data)
            .to_string(16)
            .map_err(|err| anyhow!("String hash error: {err:?}"))?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tornado::Tornado;
    use wasm_bindgen_test::*;

    const NET_ID: u32 = 5;
    const CURRENCY: &str = "eth";
    const AMOUNT: &str = "0.1";

    // https://goerli.etherscan.io/tx/0x06e10a9ea49183e9127fb7581d4d54750290c1ecc7c7f1707953f706fe9ab959
    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const NULLIFIER_HASH: &str = "2d39004125a3df2cbb59ad3aa3dee045fac6f176376343632be7b9cc476ad423";
    const COMMITMENT_HASH: &str =
        "296137799075f986ce4c0bbbfdade2b96689d7720d2e8b59a84bf48f4afe9ad8";

    // https://goerli.etherscan.io/tx/0x7b5ee6c14b86509c2b401ee1ec15657f303494acfe5d786cc4081a6666f34414
    const COST_NOTE: &str = r"tornado-eth-0.1-5-0x4805479a68a261e0850509d4a0724877c9395be42d78146b05880d7fd4b9484e92c8de0dfc2df89aae1a7d87726da32eed131fde50bff26a0392ce2b6729";
    const COST_NULLIFIER_HASH: &str =
        "29e06ac32f5db0048ed954c971413c676bdf65bc318ee72bc52ce6162c76bf09";
    const COST_COMMITMENT_HASH: &str =
        "0129af81b9bdf54d834cdef1c6aab21c5ff95e4c40f10bc3a013bd929fbc38ac";

    #[wasm_bindgen_test]
    async fn test_parse_note() {
        let tornado = Tornado::new(vec![], vec![]).await.unwrap();
        let note = Note::new(NOTE, &tornado.util).unwrap();
        let cost_note = Note::new(COST_NOTE, &tornado.util).unwrap();

        assert_eq!(
            note,
            Note {
                currency: CURRENCY.into(),
                amount: AMOUNT.into(),
                net_id: NET_ID,
                nullifier_hash: NULLIFIER_HASH.into(),
                commitment_hash: COMMITMENT_HASH.into(),
            }
        );
        assert_eq!(
            cost_note,
            Note {
                currency: CURRENCY.into(),
                amount: AMOUNT.into(),
                net_id: NET_ID,
                nullifier_hash: COST_NULLIFIER_HASH.into(),
                commitment_hash: COST_COMMITMENT_HASH.into(),
            }
        );
    }

    #[wasm_bindgen_test]
    async fn test_read_event_log() {
        let tornado = Tornado::new(vec![], vec![]).await.unwrap();
        Note::new(NOTE, &tornado.util)
            .unwrap()
            .read_event_log(Some(EventLogType::Deposit), &tornado.util)
            .await
            .unwrap();
        Note::new(NOTE, &tornado.util)
            .unwrap()
            .read_event_log(Some(EventLogType::Deposit), &tornado.util)
            .await
            .unwrap();
    }
}
