mod typ;
pub use typ::*;

use anyhow::{anyhow, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use hex::FromHex;
use js_sys::{BigInt, Uint8Array};
use regex::Regex;
use std::env;
use wasm_bindgen::JsValue;
use web_sys::console;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    id: usize,
    currency: String,
    amount: String,
    net_id: u32,
    nullifier_hash: BigInt,
    commitment_hash: BigInt,
    event_log_type: Option<EventLogType>,
    event_log_list: EventLogList,
}

impl Note {
    pub async fn read_event_log(mut self, tor: &Tornado) -> Result<Self> {
        let net_id = self.net_id;
        let key = format!("netId{}", net_id);
        let config = tor
            .config
            .get(&key)
            .ok_or(anyhow!("net_id `{net_id}` not support"))?;
        let base_dir = &format!("{}/{}", EVENT_LOG_PATH, config.name);

        match self.event_log_type {
            Some(typ @ (EventLogType::Deposit | EventLogType::Withdrawal)) => {
                let content = self._read_file(tor, base_dir, typ).await?;
                self.event_log_list = serde_json::from_str(&content)?;
            }
            _ => {
                let content = self
                    ._read_file(tor, base_dir, EventLogType::Deposit)
                    .await?;
                let deposit_list: EventLogList = serde_json::from_str(&content)?;
                let content = self
                    ._read_file(tor, base_dir, EventLogType::Withdrawal)
                    .await?;
                let withdraw_list: EventLogList = serde_json::from_str(&content)?;
                self.event_log_list = [deposit_list, withdraw_list].concat();
            }
        }

        Ok(self)
    }

    async fn _read_file(&self, tor: &Tornado, base_dir: &str, typ: EventLogType) -> Result<String> {
        // tornado event log cache file path, only cache data used for convenience
        let path = format!(
            "{}/{}_{}_{}.json",
            base_dir,
            serde_json::to_string(&typ).unwrap().replace("\"", ""),
            self.currency,
            self.amount
        );

        Uint8Array::from(
            tor.util
                .read_file(JsValue::from_str(&path))
                .await
                .map_err(|err| {
                    anyhow!(
                        "Failed to read cache file, ensure that the file `{path}` exit.\n{err:?}"
                    )
                })?,
        )
        .to_string()
        .as_string()
        .ok_or(anyhow!(
            "Failed to read cache file, ensure that the file format correct."
        ))
    }
}

pub struct Tornado {
    note_list: Vec<Note>,
    config: NetIdConfigMap,
    util: TornadoUtil,
}

impl Default for Tornado {
    fn default() -> Self {
        Self {
            note_list: vec![],
            config: serde_json::from_str(CONFIG).unwrap(),
            util: TornadoUtil::new(),
        }
    }
}

impl Tornado {
    pub async fn new(list: Vec<&str>) -> Result<Self> {
        let s = Self::default();
        s.util.init().await;
        s.parse_note(list).await
    }

    pub async fn check_block(&self, ind: Option<u8>) -> Result<bool> {
        todo!()
    }

    pub async fn parse_note(mut self, list: Vec<&str>) -> Result<Self> {
        for (i, note) in list.iter().enumerate() {
            let re = Regex::new(NOTE_REGEX)?;
            let caps = re
                .captures(note)
                .ok_or(anyhow!("Tornado note format is incorrect"))?;

            let currency = caps.name("currency").unwrap().as_str().into();
            let amount = caps.name("amount").unwrap().as_str().into();
            let net_id = caps.name("netId").unwrap().as_str().parse()?;
            let note = <[u8; 62]>::from_hex(caps.name("note").unwrap().as_str())?;
            let preimage = Uint8Array::from(&note[..]);
            let nullifier = Uint8Array::from(&note[..31]);

            let commitment_hash = self.util.pedersen_hash(preimage);
            let nullifier_hash = self.util.pedersen_hash(nullifier);

            self.note_list.push(Note {
                id: i + 1,
                currency,
                amount,
                net_id,
                commitment_hash,
                nullifier_hash,
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![],
            });
        }

        Ok(self)
    }

    pub async fn read_event_log(mut self) -> Result<Self> {
        let task_list = FuturesUnordered::new();

        for note in self.note_list.clone() {
            task_list.push(note.read_event_log(&self));
        }

        self.note_list = task_list.collect::<Vec<Result<Note>>>().await.into_iter().map(|res| {
            match res {
                Ok(note) => {
                    console::info_1(&JsValue::from_str(&format!("Note#{} parses successfully.", note.id)));
                    Some(note)
                },
                Err(err) => {
                    console::error_1(&JsValue::from_str(&format!("Some notes fail to read the event log file, others will continue to run.\n{:?}", err)));
                    None
                }
            }
        }).flatten().collect::<Vec<Note>>();

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    const NET_ID: u32 = 5;
    const CURRENCY: &str = "eth";
    const AMOUNT: &str = "0.1";

    // https://goerli.etherscan.io/tx/0x06e10a9ea49183e9127fb7581d4d54750290c1ecc7c7f1707953f706fe9ab959
    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const NULLIFIER_HASH: &str =
        "20454790225299856478795038962746880644063954504776542630455482831220240995363";
    const COMMITMENT_HASH: &str =
        "18716593830613547391848516730128799739861563597347942888893803512076728965848";

    // https://goerli.etherscan.io/tx/0x7b5ee6c14b86509c2b401ee1ec15657f303494acfe5d786cc4081a6666f34414
    const COST_NOTE: &str = r"tornado-eth-0.1-5-0x4805479a68a261e0850509d4a0724877c9395be42d78146b05880d7fd4b9484e92c8de0dfc2df89aae1a7d87726da32eed131fde50bff26a0392ce2b6729";
    const COST_NULLIFIER_HASH: &str =
        "18941337381714858446355925653430800737045061541595044917820195410400865861385";
    const COST_COMMITMENT_HASH: &str =
        "525964881243906792375230974931225736378364691955935045809786306735191636140";

    #[wasm_bindgen_test]
    async fn test_parse_note() {
        let tornado = Tornado::new(vec![NOTE, COST_NOTE]).await.unwrap();
        assert_eq!(
            tornado.note_list[0],
            Note {
                id: 1,
                currency: CURRENCY.into(),
                amount: AMOUNT.into(),
                net_id: NET_ID,
                nullifier_hash: JsValue::bigint_from_str(NULLIFIER_HASH).into(),
                commitment_hash: JsValue::bigint_from_str(COMMITMENT_HASH).into(),
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![]
            }
        );
        assert_eq!(
            tornado.note_list[1],
            Note {
                id: 2,
                currency: CURRENCY.into(),
                amount: AMOUNT.into(),
                net_id: NET_ID,
                nullifier_hash: JsValue::bigint_from_str(COST_NULLIFIER_HASH).into(),
                commitment_hash: JsValue::bigint_from_str(COST_COMMITMENT_HASH).into(),
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![]
            }
        );
    }

    #[wasm_bindgen_test]
    async fn test_read_event_log() {
        Tornado::new(vec![NOTE, COST_NOTE])
            .await
            .unwrap()
            .read_event_log()
            .await
            .unwrap();
    }
}
