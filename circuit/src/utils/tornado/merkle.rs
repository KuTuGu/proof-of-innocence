use ff::*;
use merkle_light::hash::Algorithm;
use merkle_light::merkle::MerkleTree;
use mimc_sponge_rs::{Fr, MimcSponge};
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
    let mut data = BigUint::from_str_radix(str, 16).unwrap().to_bytes_le();
    data.resize(32, 0);
    data.reverse();
    data.try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    }
}
