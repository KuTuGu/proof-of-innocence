mod merkle;
mod note;
mod typ;

use anyhow::{anyhow, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
pub use merkle::*;
use note::Note;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub use typ::*;

pub struct Tornado {
    note_list: Vec<Note>,
    block_list: Vec<HashStr>,
    util: TornadoUtil,
}

impl Default for Tornado {
    fn default() -> Self {
        Self {
            note_list: vec![],
            block_list: vec![],
            util: TornadoUtil::new(),
        }
    }
}

impl Tornado {
    pub async fn new(note_list: Vec<String>, block_list: Vec<String>) -> Result<Self> {
        let s = Self::default().set_block_list(block_list);
        s.util.init().await;
        s.parse_note(note_list)
    }

    pub async fn prove(self) -> Result<Vec<Proof>> {
        let util = &self.util;
        let accuracy_tree_cache = Rc::new(RefCell::new(HashMap::new()));
        let innocence_tree = &SparseMerkleTree::new(self.block_list.clone());
        let task_list = FuturesUnordered::new();

        for note in &self.note_list {
            let accuracy_tree_cache = Rc::clone(&accuracy_tree_cache);
            task_list.push(async move {
                let mut accuracy_tree_cache = accuracy_tree_cache.borrow_mut();
                let accuracy_tree = match accuracy_tree_cache.get(note) {
                    Some(tree) => tree,
                    None => {
                        let log_list = note
                            .read_event_log(Some(EventLogType::Deposit), util)
                            .await?;
                        let leaves = log_list
                            .into_iter()
                            .map(|log| match log {
                                EventLog::Deposit(log) => {
                                    log.commitment.trim_start_matches("0x").into()
                                }
                                _ => unreachable!(),
                            })
                            .collect::<Vec<String>>();

                        let tree = TornadoMerkleTree::new(leaves);
                        accuracy_tree_cache.insert(note.clone(), tree);
                        accuracy_tree_cache.get(note).unwrap()
                    }
                };
                let commitment = to_hash(note.commitment());
                let index = accuracy_tree.leafs()
                    - accuracy_tree[..accuracy_tree.leafs()]
                        .iter()
                        .rev()
                        .position(|a| a == &commitment)
                        .ok_or(anyhow!(
                            "Deposit log not exist in history, please check the cache file."
                        ))?
                    - 1;
                let (accuracy_proof_element, accuracy_proof_index) = accuracy_tree.prove(index);

                Ok(Proof {
                    commitment,
                    accuracy_tree_root: accuracy_tree.root(),
                    innocence_tree_root: innocence_tree.root(),
                    accuracy_proof_element,
                    accuracy_proof_index,
                    innocence_proof: innocence_tree.prove(commitment),
                })
            });
        }

        let proof_list = task_list
            .collect::<Vec<Result<Proof>>>()
            .await
            .into_iter()
            .map(|r| r.map_err(|err| anyhow!("Failed to generate a proof for some Notes.{err}")))
            .collect::<Result<Vec<Proof>, _>>()?;

        Ok(proof_list)
    }

    fn parse_note(mut self, list: Vec<String>) -> Result<Self> {
        self.note_list = list
            .iter()
            .map(|note| Note::new(note, &self.util))
            .collect::<Result<Vec<Note>>>()?;

        Ok(self)
    }

    fn set_block_list(mut self, block_list: Vec<String>) -> Self {
        self.block_list = block_list;
        self
    }
}
