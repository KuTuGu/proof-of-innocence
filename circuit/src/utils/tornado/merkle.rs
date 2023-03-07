use ff::*;
use merkle_light::hash::Algorithm;
use merkle_light::merkle::MerkleTree;
use mimc_sponge_rs::{Fr, MimcSponge};
use novasmt::{Database, FullProof, InMemoryCas, Tree};
use num_bigint::BigUint;
use num_traits::Num;
use regex::Regex;
use std::hash::Hasher;
use std::ops::Deref;

pub const LEVEL: usize = 20;
// keccak256("tornado") % BN254_FIELD_SIZE
pub const ZERO_ELEMENT: &str = "2fe54c60d3acabf3343a35b6eba15db4821b340f76e741e2249685ed4899af6c";
// hash type
const LEAF: u8 = 0x00;
const INTERIOR: u8 = 0x01;

pub struct TornadoMerkleTree(MerkleTree<[u8; 32], MimcHasher>);

impl Deref for TornadoMerkleTree {
    type Target = MerkleTree<[u8; 32], MimcHasher>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TornadoMerkleTree {
    pub fn new(list: Vec<String>) -> Self {
        Self(
            MerkleTree::new(list.iter().map(|s| raw_data(s)))
                .fixed_level(LEVEL, raw_data(ZERO_ELEMENT))
                .build(),
        )
    }
}

pub struct SparseMerkleTree(Tree<InMemoryCas>);

impl Deref for SparseMerkleTree {
    type Target = Tree<InMemoryCas>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SparseMerkleTree {
    pub fn new(list: Vec<String>) -> Self {
        let forest = Database::new(InMemoryCas::default());
        let mut tree = forest.get_tree([0; 32]).unwrap();

        for item in list {
            tree.insert(raw_data(&item), &raw_data("1"));
        }
        assert!(tree.root_hash() != [0; 32], "Empty Sparse Merkle Tree");

        Self(tree)
    }

    pub fn generate_proof(&self, key: &str) -> ([u8; 32], FullProof) {
        let key = raw_data(key);
        let (_val, proof) = self.get_with_proof(key);
        (key, proof)
    }
}

pub struct MimcHasher {
    inner: MimcSponge,
    data: Vec<u8>,
}

impl Default for MimcHasher {
    fn default() -> Self {
        Self {
            inner: MimcSponge::default(),
            data: vec![],
        }
    }
}

impl Deref for MimcHasher {
    type Target = MimcSponge;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Hasher for MimcHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        unimplemented!()
    }
}

impl Algorithm<[u8; 32]> for MimcHasher {
    #[inline]
    fn hash(&mut self) -> [u8; 32] {
        match self.data[0] {
            LEAF => (&self.data[1..=32]).try_into().unwrap(),
            INTERIOR => {
                let res = self.multi_hash(
                    &[fr(&self.data[1..=32]), fr(&self.data[33..=64])],
                    Fr::zero(),
                    1,
                );
                let re = Regex::new(r"0x([0-9a-fA-F]+)").unwrap();
                raw_data(&re.captures(&res[0].to_string()).unwrap()[1])
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.data = vec![];
    }
}

fn fr(data: &[u8]) -> Fr {
    Fr::from_str(&BigUint::from_bytes_be(data).to_str_radix(10)).unwrap()
}
fn raw_data(str: &str) -> [u8; 32] {
    to32(
        BigUint::from_str_radix(str, 16)
            .unwrap()
            .to_bytes_be()
            .as_ref(),
    )
}
fn to32(data: &[u8]) -> [u8; 32] {
    let len = data.len();
    if len == 32 {
        data.try_into().unwrap()
    } else if len < 32 {
        [&vec![0; 32 - data.len()][..], &data[..]]
            .concat()
            .try_into()
            .unwrap()
    } else {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merkle_light::proof::Proof;
    use wasm_bindgen_test::*;

    const LEAF: &str = "09ee41e2a667251b7bedc2032977ab5ce9d2b2b79e158e252c13025820804dc1";
    const ROOT: &str = "29316f2a7749ea8161528e6b42cc35591d8ccddd01911028c460a7930ae00458";

    #[wasm_bindgen_test]
    async fn test_tornado_merkle_tree() {
        let t = TornadoMerkleTree::new(vec![LEAF.into()]);
        assert_eq!(
            BigUint::from_bytes_be(t.root().as_ref()).to_str_radix(16),
            ROOT
        );

        let proof = t.gen_proof(0);
        assert!(proof.validate::<MimcHasher>());

        let mut fake_path = proof.path().to_vec();
        fake_path[0] = !fake_path[0];
        let fake_proof = Proof::new(proof.lemma().to_vec(), fake_path);
        assert!(fake_proof.validate::<MimcHasher>() == false);
    }
}
