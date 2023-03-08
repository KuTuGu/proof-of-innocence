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
    block_list: Vec<Hash>,
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

    pub async fn generate_proof(self) -> Result<Vec<Proof>> {
        let util = &self.util;
        let t = &SparseMerkleTree::new(self.block_list);
        let task_list = FuturesUnordered::new();

        for note in &self.note_list {
            task_list.push(async move {
                let (commitment, innocence_proof) = t.generate_proof(&note.commitment());
                let (accuracy_root, accuracy_proof) = note.generate_deposit_proof(&util).await?;

                Ok(Proof {
                    commitment,
                    commitment_tree_root: accuracy_root,
                    block_tree_root: t.root_hash(),
                    accuracy_proof_element: accuracy_proof.lemma().to_vec(),
                    accuracy_proof_index: accuracy_proof.path().to_vec(),
                    innocence_proof,
                })
            });
        }

        let proof_list = task_list
            .collect::<Vec<Result<Proof>>>()
            .await
            .into_iter()
            .map(|r| r.map_err(|err| anyhow!("Failed to generate a proof for some Notes.\n{err}")))
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
