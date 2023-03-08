mod utils;

use anyhow::Result;
use merkle_light::proof::Proof as AccuracyProof;
use novasmt::CompressedProof;
pub use utils::tornado::Proof;
use utils::tornado::{MimcHasher, Tornado};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub async fn prove(note_list: Vec<JsValue>, block_list: Vec<JsValue>) -> Result<String, JsValue> {
    let note_list = note_list
        .iter()
        .map(|note| {
            note.as_string().ok_or(JsValue::from_str(
                "Parse note error, make sure you enter a string",
            ))
        })
        .collect::<Result<_, _>>()?;
    let block_list = block_list
        .iter()
        .map(|block| {
            block.as_string().ok_or(JsValue::from_str(
                "Parse block error, make sure you enter a string",
            ))
        })
        .collect::<Result<_, _>>()?;
    let tornado = Tornado::new(note_list, block_list)
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    let proof = tornado
        .generate_proof()
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    let proof_str = serde_json::to_string(&proof).unwrap();

    if verify(proof) {
        Ok(proof_str)
    } else {
        Err(JsValue::from_str(
            "The proof cannot be verified, please ensure the accuracy of input.",
        ))
    }
}

pub fn verify(proof_list: Vec<Proof>) -> bool {
    let mut res = true;

    for proof in proof_list {
        let accuracy_proof =
            AccuracyProof::new(proof.accuracy_proof_element, proof.accuracy_proof_index);
        let element = accuracy_proof.lemma();
        let innocence_proof = CompressedProof(proof.innocence_proof).decompress().unwrap();

        res = res
            && element[0] == proof.commitment
            && element[element.len() - 1] == proof.commitment_tree_root
            && accuracy_proof.validate::<MimcHasher>()
            && innocence_proof.verify(proof.block_tree_root, proof.commitment, &[]);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const COMMITMENT_HASH: &str =
        "296137799075f986ce4c0bbbfdade2b96689d7720d2e8b59a84bf48f4afe9ad8";
    const OTHER_HASH: &str = "296137799075f986ce4c0bbbfdade2b96689d7720d2e8b59a84bf48f4afe9ad9";

    #[wasm_bindgen_test]
    async fn test_circuit() {
        assert!(verify(
            serde_json::from_str(
                &prove(
                    vec![JsValue::from_str(NOTE)],
                    vec![JsValue::from_str(OTHER_HASH)]
                )
                .await
                .unwrap()
            )
            .unwrap()
        ));
    }

    #[wasm_bindgen_test]
    async fn test_fail_circuit() {
        assert!(prove(
            vec![JsValue::from_str(NOTE)],
            vec![JsValue::from_str(COMMITMENT_HASH)]
        )
        .await
        .is_err());
    }
}
