mod merkle;
mod note;
mod typ;

use anyhow::Result;
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

    pub async fn generate_proof(self) -> Vec<Proof> {
        let util = &self.util;
        let t = &SparseMerkleTree::new(self.block_list);
        let task_list = FuturesUnordered::new();

        for note in &self.note_list {
            task_list.push(async move {
                let (key, proof) = t.generate_proof(&note.commitment());
                let (accuracy_root, accuracy_proof) = note.generate_deposit_proof(&util).await?;

                Ok(Proof {
                    commitment: key,
                    commitment_tree_root: accuracy_root,
                    block_tree_root: t.root_hash(),
                    accuracy_proof_element: accuracy_proof.lemma().to_vec(),
                    accuracy_proof_index: accuracy_proof.path().to_vec(),
                    non_existence_proof: proof.compress().0,
                })
            });
        }

        let proof_list = task_list
            .collect::<Vec<Result<Proof>>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<Proof>>();

        assert_eq!(
            proof_list.len(),
            self.note_list.len(),
            "Failed to generate a proof for some Notes"
        );

        proof_list
    }

    fn parse_note(mut self, list: Vec<String>) -> Result<Self> {
        self.note_list = list
            .iter()
            .map(|note| Note::new(note, &self.util).unwrap())
            .collect();

        Ok(self)
    }

    fn set_block_list(mut self, block_list: Vec<String>) -> Self {
        self.block_list = block_list;
        self
    }
}
