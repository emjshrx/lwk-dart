use lwk_simplicity::simplicityhl;

use crate::contracts::simplicity::Cmr;

/// Result of running a Simplicity program in a transaction environment.
pub struct SimplicityRunResult {
    pub(crate) pruned:
        std::sync::Arc<simplicityhl::simplicity::RedeemNode<simplicityhl::simplicity::jet::Elements>>,
    pub(crate) value: simplicityhl::simplicity::Value,
}

impl SimplicityRunResult {
    pub fn program_bytes(&self) -> Vec<u8> {
        self.pruned.to_vec_with_witness().0
    }

    pub fn witness_bytes(&self) -> Vec<u8> {
        self.pruned.to_vec_with_witness().1
    }

    pub fn cmr(&self) -> Cmr {
        self.pruned.cmr().into()
    }

    pub fn value(&self) -> String {
        format!("{:?}", self.value)
    }
}
