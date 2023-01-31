use super::utils::*;
use dusk_plonk::prelude::*;

#[derive(Debug, Default)]
pub struct TheCircuit {
    pub source_list: Vec<BlsScalar>,
    pub block_list: Vec<BlsScalar>,
}

// Don't think that we can format the Merkle tree outside, only check the inclusion relationship in this circuit.
// Because this circuit can only prove that the two lists have an exclusion relationship,
// and cant prove other things, such as whether each proof in correct format, double spending, etc.,
// so we have to handle these processes in this circuit.
impl Circuit for TheCircuit {
    fn circuit<C: Composer>(&self, composer: &mut C) -> Result<(), dusk_plonk::error::Error> {
        for source in self.source_list.to_vec() {
            for block in self.block_list.to_vec() {
                let source = composer.append_witness(source);
                let block = composer.append_public(block);
                let is_equal = equal::is_equal(composer, source, block);
                composer.assert_equal_constant(is_equal, BlsScalar::zero(), None);
            }
        }

        Ok(())
    }
}

impl TheCircuit {
    pub fn source_list(mut self, list: Vec<&str>) -> Self {
        self.source_list = self.into_scalar_list(list);
        self
    }

    pub fn block_list(mut self, list: Vec<&str>) -> Self {
        self.block_list = self.into_scalar_list(list);
        self
    }

    fn into_scalar_list(&self, list: Vec<&str>) -> Vec<BlsScalar> {
        list.iter()
            .filter_map(|i| scalar::from_addr(i).ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TheCircuit;
    use dusk_plonk::prelude::*;
    use rand_core::OsRng;

    #[test]
    fn test_correct_prove() {
        let label = b"test";
        let pp = PublicParameters::setup(1 << 12, &mut OsRng).expect("failed to setup");

        let circuit = TheCircuit::default()
            .source_list(vec!["0x0000000000000000000000000000000000000000"])
            .block_list(vec!["0x0000000000000000000000000000000000000001"]);

        // The size of the default circuit is different from the custom circuit, so we use `compile_with_circuit` instead.
        let (prover, verifier) = Compiler::compile_with_circuit::<TheCircuit>(&pp, label, &circuit)
            .expect("failed to compile circuit");

        // Generate the proof and its public inputs
        let (proof, public_inputs) = prover.prove(&mut OsRng, &circuit).expect("failed to prove");

        // Verify the generated proof
        verifier
            .verify(&proof, &public_inputs)
            .expect("failed to verify proof");
    }

    #[test]
    #[should_panic(expected = "failed to prove: PolynomialDegreeTooLarge")]
    fn test_fail_prove() {
        let label = b"test";
        let pp = PublicParameters::setup(1 << 12, &mut OsRng).expect("failed to setup");

        let circuit = TheCircuit::default()
            .source_list(vec!["0x0000000000000000000000000000000000000000"])
            .block_list(vec!["0x0000000000000000000000000000000000000000"]);

        // The size of the default circuit is different from the custom circuit, so we use `compile_with_circuit` instead.
        let (prover, verifier) = Compiler::compile_with_circuit::<TheCircuit>(&pp, label, &circuit)
            .expect("failed to compile circuit");

        // Generate the proof and its public inputs
        let (proof, public_inputs) = prover.prove(&mut OsRng, &circuit).expect("failed to prove");

        // Verify the generated proof
        verifier
            .verify(&proof, &public_inputs)
            .expect("failed to verify proof");
    }

    #[test]
    #[should_panic(expected = "failed to verify proof")]
    fn test_fail_proof() {
        let label = b"test";
        let pp = PublicParameters::setup(1 << 12, &mut OsRng).expect("failed to setup");

        let circuit = TheCircuit::default()
            .source_list(vec!["0x0000000000000000000000000000000000000000"])
            .block_list(vec!["0x0000000000000000000000000000000000000001"]);

        // The size of the default circuit is different from the custom circuit, so we use `compile_with_circuit` instead.
        let (prover, verifier) = Compiler::compile_with_circuit::<TheCircuit>(&pp, label, &circuit)
            .expect("failed to compile circuit");

        // Generate the proof and its public inputs
        let (proof, mut public_inputs) =
            prover.prove(&mut OsRng, &circuit).expect("failed to prove");

        public_inputs[0] = BlsScalar::zero();

        // Verify the generated proof
        verifier
            .verify(&proof, &public_inputs)
            .expect("failed to verify proof");
    }
}
