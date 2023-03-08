// TODO: Rename this file to change the name of this method from METHOD_NAME

#![no_main]
// #![no_std] // std support is experimental, but you can remove this to try it

use circuit::{verify, Proof};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    // TODO: Implement your guest code here
    let proof: Vec<Proof> = env::read();
    assert!(verify(proof));
}
