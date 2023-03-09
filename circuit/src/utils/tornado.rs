mod merkle;
mod note;
mod typ;

use anyhow::{anyhow, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
pub use merkle::*;
use note::Note;
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
        let t = &SparseMerkleTree::new(self.block_list);
        let task_list = FuturesUnordered::new();

        for note in &self.note_list {
            task_list.push(async move {
                let commitment = to_hash(note.commitment());
                let innocence_proof = t.prove(commitment);
                let (accuracy_root, accuracy_proof_element, accuracy_proof_index) =
                    note.prove(&util).await?;

                Ok(Proof {
                    commitment,
                    accuracy_tree_root: accuracy_root,
                    innocence_tree_root: t.root(),
                    accuracy_proof_element,
                    accuracy_proof_index,
                    innocence_proof,
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
