pub mod circuit;

use circuit::TestCircuit;
use dusk_jubjub::GENERATOR_EXTENDED;
use dusk_plonk::prelude::*;
use rand_core::OsRng;

fn main() {
    let label = b"transcript-arguments";
    let pp = PublicParameters::setup(1 << 12, &mut OsRng).expect("failed to setup");

    let (prover, verifier) =
        Compiler::compile::<TestCircuit>(&pp, label).expect("failed to compile circuit");

    let circuit = TestCircuit {
        a: BlsScalar::from(20u64),
        b: BlsScalar::from(5u64),
        c: BlsScalar::from(25u64),
        d: BlsScalar::from(100u64),
        e: JubJubScalar::from(2u64),
        f: JubJubAffine::from(GENERATOR_EXTENDED * JubJubScalar::from(2u64)),
    };

    // Generate the proof and its public inputs
    let (proof, mut public_inputs) = prover.prove(&mut OsRng, &circuit).expect("failed to prove");

    // public_inputs[0] = BlsScalar::zero();

    // Verify the generated proof
    verifier
        .verify(&proof, &public_inputs)
        .expect("failed to verify proof");
}
