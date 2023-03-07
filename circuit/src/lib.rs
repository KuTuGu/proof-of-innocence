mod utils;

use merkle_light::proof::Proof as AccuracyProof;
use novasmt::CompressedProof;
use utils::tornado::{MimcHasher, Proof};

pub fn circuit(proof_list: Vec<Proof>) -> bool {
    let mut res = true;

    for proof in proof_list {
        let accuracy_proof =
            AccuracyProof::new(proof.accuracy_proof_element, proof.accuracy_proof_index);
        let element = accuracy_proof.lemma();
        let non_existence_proof = CompressedProof(proof.non_existence_proof)
            .decompress()
            .unwrap();

        res = res
            && element[0] == proof.commitment
            && element[element.len() - 1] == proof.commitment_tree_root
            && accuracy_proof.validate::<MimcHasher>()
            && non_existence_proof.verify(proof.block_tree_root, proof.commitment, &[]);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::{circuit, utils::tornado::Tornado};
    use wasm_bindgen_test::*;

    const NOTE: &str = r"tornado-eth-0.1-5-0xebcf5edb762e52e6eb0f33818c647cdceb75d1cd6609847ec56b750445de0b659a11796781c60aaf3ba5d693b360a77d5cff360c982ed9dc2fd419b858d3";
    const COMMITMENT_HASH: &str =
        "296137799075f986ce4c0bbbfdade2b96689d7720d2e8b59a84bf48f4afe9ad8";
    const OTHER_HASH: &str = "296137799075f986ce4c0bbbfdade2b96689d7720d2e8b59a84bf48f4afe9ad9";

    #[wasm_bindgen_test]
    async fn test_success_circuit() {
        let source_list = vec![NOTE.into()];
        let block_list = vec![OTHER_HASH.into()];
        let tornado = Tornado::new(source_list, block_list).await.unwrap();
        let proof = tornado.generate_proof().await;
        assert!(circuit(proof));
    }

    #[wasm_bindgen_test]
    async fn test_fail_circuit() {
        let source_list = vec![NOTE.into()];
        let block_list = vec![COMMITMENT_HASH.into()];
        let tornado = Tornado::new(source_list, block_list).await.unwrap();
        let proof = tornado.generate_proof().await;
        assert!(circuit(proof) == false);
    }
}
