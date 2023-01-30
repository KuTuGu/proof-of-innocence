pub mod circuit;
pub mod utils;

use circuit::TheCircuit;
use dusk_plonk::prelude::*;
use rand_core::OsRng;

fn main() {
    let label = b"proof-of-innocence";
    let pp = PublicParameters::setup(1 << 12, &mut OsRng).expect("failed to setup");

    let circuit = TheCircuit::default()
        .source_list(vec!["0x0000000000000000000000000000000000000000"])
        .block_list(vec![
            "0x0000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000002",
            "0x0000000000000000000000000000000000000003",
        ]);

    // The size of the default circuit is different from the custom circuit, so we use `compile_with_circuit` instead.
    let (prover, verifier) = Compiler::compile_with_circuit::<TheCircuit>(&pp, label, &circuit)
        .expect("failed to compile circuit");

    // Generate the proof and its public inputs
    let (proof, mut public_inputs) = prover.prove(&mut OsRng, &circuit).expect("failed to prove");

    // public_inputs[0] = BlsScalar::zero();

    // Verify the generated proof
    verifier
        .verify(&proof, &public_inputs)
        .expect("failed to verify proof");
}
