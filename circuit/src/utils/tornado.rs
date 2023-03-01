mod net;
mod typ;
pub use net::*;
pub use typ::*;

use anyhow::{anyhow, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use hex::FromHex;
use js_sys::Uint8Array;
use regex::Regex;
use wasm_bindgen::JsValue;
use web_sys::console;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    id: usize,
    currency: String,
    amount: String,
    net_id: u32,
    nullifier_hash: Hash,
    commitment_hash: Hash,
    event_log_type: Option<EventLogType>,
    event_log_list: Vec<EventLog>,
    deposit_event_log: Option<EventLog>,
}

impl Note {
    pub async fn read_event_log(mut self, tor: &Tornado) -> Result<Self> {
        let net_id = self.net_id;
        let base_dir = &format!(
            "{}/{}",
            EVENT_LOG_PATH,
            NET_INFO_MAP
                .get(&net_id)
                .ok_or(anyhow!("Net#{net_id} not support"))?
                .name
        );

        match self.event_log_type {
            Some(typ @ (EventLogType::Deposit | EventLogType::Withdrawal)) => {
                let content = self._read_file(tor, base_dir, typ).await?;
                self.event_log_list = serde_json::from_str(&content)?;
            }
            _ => {
                let content = self
                    ._read_file(tor, base_dir, EventLogType::Deposit)
                    .await?;
                let deposit_list: Vec<EventLog> = serde_json::from_str(&content)?;
                let content = self
                    ._read_file(tor, base_dir, EventLogType::Withdrawal)
                    .await?;
                let withdraw_list: Vec<EventLog> = serde_json::from_str(&content)?;
                self.event_log_list = [deposit_list, withdraw_list].concat();
            }
        }

        Ok(self)
    }

    pub fn search_deposit_log(mut self) -> Self {
        self.deposit_event_log = self
            .event_log_list
            .iter()
            .rev()
            .find(|log| match log {
                EventLog::Deposit(log) => {
                    log.commitment.trim_start_matches("0x") == self.commitment_hash
                }
                _ => unreachable!(),
            })
            .cloned();
        self
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
    util: TornadoUtil,
}

impl Default for Tornado {
    fn default() -> Self {
        Self {
            note_list: vec![],
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

    pub async fn check_block(&self) -> Result<bool> {
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
            let commitment_hash = self._hash_data(preimage).await?;
            let nullifier_hash = self._hash_data(nullifier).await?;

            self.note_list.push(Note {
                id: i + 1,
                currency,
                amount,
                net_id,
                commitment_hash,
                nullifier_hash,
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![],
                deposit_event_log: None,
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

    pub fn search_deposit_log(mut self) -> Self {
        self.note_list = self
            .note_list
            .into_iter()
            .map(|note| note.search_deposit_log())
            .collect();

        self
    }

    async fn _hash_data(&self, data: Uint8Array) -> Result<String> {
        Ok(format!(
            "{:0>64}",
            self.util
                .pedersen_hash(data)
                .to_string(16)
                .map_err(|err| anyhow!("String hash error: {err:?}"))?
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use num_bigint::BigUint;
    use std::str::FromStr;
    use wasm_bindgen_test::*;

    const NET_ID: u32 = 5;
    const CURRENCY: &str = "eth";
    const AMOUNT: &str = "0.1";

    // https://goerli.etherscan.io/tx/0x06e10a9ea49183e9127fb7581d4d54750290c1ecc7c7f1707953f706fe9ab959
    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const NOTE_DEPOSIT_BLOCK_NUMBER: u32 = 8559256;
    const NULLIFIER_HASH: &str =
        "20454790225299856478795038962746880644063954504776542630455482831220240995363";
    const COMMITMENT_HASH: &str =
        "18716593830613547391848516730128799739861563597347942888893803512076728965848";

    // https://goerli.etherscan.io/tx/0x7b5ee6c14b86509c2b401ee1ec15657f303494acfe5d786cc4081a6666f34414
    const COST_NOTE: &str = r"tornado-eth-0.1-5-0x4805479a68a261e0850509d4a0724877c9395be42d78146b05880d7fd4b9484e92c8de0dfc2df89aae1a7d87726da32eed131fde50bff26a0392ce2b6729";
    const COST_NOTE_DEPOSIT_BLOCK_NUMBER: u32 = 8558607;
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
                nullifier_hash: format!(
                    "{:0>64}",
                    BigUint::from_str(NULLIFIER_HASH).unwrap().to_str_radix(16)
                ),
                commitment_hash: format!(
                    "{:0>64}",
                    BigUint::from_str(COMMITMENT_HASH).unwrap().to_str_radix(16)
                ),
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![],
                deposit_event_log: None,
            }
        );
        assert_eq!(
            tornado.note_list[1],
            Note {
                id: 2,
                currency: CURRENCY.into(),
                amount: AMOUNT.into(),
                net_id: NET_ID,
                nullifier_hash: format!(
                    "{:0>64}",
                    BigUint::from_str(COST_NULLIFIER_HASH)
                        .unwrap()
                        .to_str_radix(16)
                ),
                commitment_hash: format!(
                    "{:0>64}",
                    BigUint::from_str(COST_COMMITMENT_HASH)
                        .unwrap()
                        .to_str_radix(16)
                ),
                event_log_type: Some(EventLogType::Deposit),
                event_log_list: vec![],
                deposit_event_log: None,
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

    #[wasm_bindgen_test]
    async fn test_search_deposit_log() {
        let tornado = Tornado::new(vec![NOTE, COST_NOTE])
            .await
            .unwrap()
            .read_event_log()
            .await
            .unwrap()
            .search_deposit_log();

        match &tornado.note_list[0].deposit_event_log {
            Some(EventLog::Deposit(log)) => log.block_number == NOTE_DEPOSIT_BLOCK_NUMBER,
            _ => unreachable!(),
        };

        match &tornado.note_list[1].deposit_event_log {
            Some(EventLog::Deposit(log)) => log.block_number == COST_NOTE_DEPOSIT_BLOCK_NUMBER,
            _ => unreachable!(),
        };
    }
}
