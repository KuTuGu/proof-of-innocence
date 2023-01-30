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
