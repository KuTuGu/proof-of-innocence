use super::Hash;
use ff::*;
use merkle_light::hash::Algorithm;
use merkle_light::merkle::MerkleTree;
use merkle_light::proof::Proof as AccuracyProof;
use mimc_sponge_rs::{Fr, MimcSponge};
use novasmt::{CompressedProof, Database, InMemoryCas, Tree};
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

pub struct TornadoMerkleTree(MerkleTree<Hash, MimcHasher>);

impl Deref for TornadoMerkleTree {
    type Target = MerkleTree<Hash, MimcHasher>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TornadoMerkleTree {
    pub fn new(list: Vec<String>) -> Self {
        Self(
            MerkleTree::new(list.iter().map(|s| to_hash(s)))
                .fixed_level(LEVEL, to_hash(ZERO_ELEMENT))
                .build(),
        )
    }

    pub fn root(&self) -> Hash {
        self.0.root()
    }

    pub fn prove(&self, i: usize) -> (Vec<Hash>, Vec<bool>) {
        let proof = self.gen_proof(i);
        (proof.lemma().to_vec(), proof.path().to_vec())
    }

    pub fn verify(root: Hash, key: Hash, element: Vec<Hash>, index: Vec<bool>) -> bool {
        let accuracy_proof = AccuracyProof::new(element, index);
        let element = accuracy_proof.lemma();

        element[0] == key
            && element[element.len() - 1] == root
            && accuracy_proof.validate::<MimcHasher>()
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
            tree.insert(to_hash(&item), &to_hash("1"));
        }

        Self(tree)
    }

    pub fn root(&self) -> Hash {
        self.root_hash()
    }

    pub fn prove(&self, key: Hash) -> Vec<u8> {
        let (_val, proof) = self.get_with_proof(key);
        proof.compress().0
    }

    pub fn verify(root: Hash, key: Hash, proof: Vec<u8>) -> bool {
        let innocence_proof = CompressedProof(proof).decompress().unwrap();
        innocence_proof.verify(root, key, &[])
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

impl Algorithm<Hash> for MimcHasher {
    #[inline]
    fn hash(&mut self) -> Hash {
        match self.data[0] {
            LEAF => (&self.data[1..=32]).try_into().unwrap(),
            INTERIOR => {
                let res = self.multi_hash(
                    &[fr(&self.data[1..=32]), fr(&self.data[33..=64])],
                    Fr::zero(),
                    1,
                );
                let re = Regex::new(r"0x([0-9a-fA-F]+)").unwrap();
                to_hash(&re.captures(&res[0].to_string()).unwrap()[1])
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
pub fn to_hash(str: &str) -> Hash {
    extend32(
        BigUint::from_str_radix(str, 16)
            .unwrap()
            .to_bytes_be()
            .as_ref(),
    )
}
fn extend32(data: &[u8]) -> Hash {
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
