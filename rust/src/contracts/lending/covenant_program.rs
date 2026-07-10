//! Internal covenant program helper — compiles `.simf` sources and builds stateful Taproot outputs.

use lwk_wollet::elements::hashes::{sha256, Hash};

use crate::api::error::LwkError;
use crate::contracts::blockdata::{Script, XOnlyPublicKey};
use crate::contracts::blockdata::script::hash_script;
use crate::contracts::simplicity::{
    SimplicityArguments, SimplicityProgram, StateTaprootBuilder, StateTaprootSpendInfo,
    UNSPENDABLE_TAPROOT_PUBKEY,
};

use super::transaction::LendingTransaction;

pub(crate) const PROGRAM_ID_LENGTH: usize = 4;
pub(crate) type ProgramId = [u8; PROGRAM_ID_LENGTH];

/// Compiled Simplicity covenant with optional on-chain storage slots.
pub(crate) struct CovenantProgram {
    compiled: SimplicityProgram,
    storage: Vec<[u8; 32]>,
    internal_key: XOnlyPublicKey,
}

impl CovenantProgram {
    pub fn load(source: &'static str, arguments: &SimplicityArguments) -> anyhow::Result<Self, LwkError> {
        Ok(Self {
            compiled: SimplicityProgram::load_with_arguments(source.to_string(), arguments)?,
            storage: Vec::new(),
            internal_key: XOnlyPublicKey::from_string(UNSPENDABLE_TAPROOT_PUBKEY.into())?,
        })
    }

    pub fn with_storage(mut self, storage: Vec<[u8; 32]>) -> Self {
        self.storage = storage;
        self
    }

    pub fn program_id(source: &'static str) -> ProgramId {
        let digest = sha256::Hash::hash(source.as_bytes());
        let mut prefix = [0u8; PROGRAM_ID_LENGTH];
        prefix.copy_from_slice(&digest.as_byte_array()[..PROGRAM_ID_LENGTH]);
        prefix
    }

    pub fn script_pubkey(&self) -> anyhow::Result<Script, LwkError> {
        Ok(self.spend_info()?.script_pubkey())
    }

    pub fn script_hash(&self) -> anyhow::Result<[u8; 32], LwkError> {
        let hash = hash_script(&self.script_pubkey()?);
        hash.try_into().map_err(|_| LwkError {
            msg: "script hash must be 32 bytes".into(),
        })
    }

    pub fn add_output(
        &self,
        tx: &mut LendingTransaction,
        asset_id: &str,
        amount: u64,
    ) -> anyhow::Result<(), LwkError> {
        tx.add_explicit_output(&self.script_pubkey()?, amount, asset_id.to_string())
            .map(|_| ())
    }

    fn spend_info(&self) -> anyhow::Result<StateTaprootSpendInfo, LwkError> {
        let total_leaves = 1 + self.storage.len();
        let depths = taproot_leaf_depths(total_leaves);

        let mut builder = StateTaprootBuilder::new();
        builder = builder.add_simplicity_leaf(depths[0] as u8, &self.compiled.cmr())?;
        for (index, slot) in self.storage.iter().enumerate() {
            builder = builder.add_data_leaf(depths[index + 1] as u8, slot.to_vec())?;
        }
        builder.finalize(&self.internal_key)
    }
}

/// Depth assignment for a balanced Taproot tree (mirrors upstream smplx-sdk).
fn taproot_leaf_depths(total_leaves: usize) -> Vec<usize> {
    assert!(total_leaves > 0, "Taproot tree must contain at least one leaf");

    let next_pow2 = total_leaves.next_power_of_two();
    let depth = next_pow2.ilog2() as usize;
    let shallow_count = next_pow2 - total_leaves;
    let deep_count = total_leaves - shallow_count;

    let mut depths = Vec::with_capacity(total_leaves);
    depths.extend(std::iter::repeat_n(depth, deep_count));
    if depth > 0 {
        depths.extend(std::iter::repeat_n(depth - 1, shallow_count));
    }
    depths
}

pub(crate) fn asset_id_argument(asset_id: &str) -> anyhow::Result<crate::contracts::simplicity::SimplicityTypedValue, LwkError> {
    let bytes = hex::decode(asset_id).map_err(|e| LwkError {
        msg: format!("invalid asset id hex: {e}"),
    })?;
    crate::contracts::simplicity::SimplicityTypedValue::u256(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::simplicity::SimplicityTypedValue;

    #[test]
    fn single_leaf_program_uses_depth_zero() {
        assert_eq!(taproot_leaf_depths(1), vec![0]);
    }

    #[test]
    fn three_leaf_program_uses_expected_depths() {
        assert_eq!(taproot_leaf_depths(3), vec![2, 2, 1]);
    }

    #[test]
    fn program_id_is_first_four_sha256_bytes_of_source() {
        let id = CovenantProgram::program_id("fn main() {}");
        assert_eq!(id.len(), 4);
        let digest = sha256::Hash::hash("fn main() {}".as_bytes());
        assert_eq!(&id, &digest.as_byte_array()[..4]);
    }

    #[test]
    fn script_auth_compiles_to_covenant_script() {
        let args = SimplicityArguments::new().add_value(
            "SCRIPT_HASH".into(),
            SimplicityTypedValue::u256(vec![0u8; 32]).unwrap(),
        );
        let program = CovenantProgram::load(super::super::simf::SCRIPT_AUTH, &args).unwrap();
        assert!(!program.script_pubkey().unwrap().to_hex().is_empty());
    }
}
