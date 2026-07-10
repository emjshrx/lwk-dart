//! Internal `asset_auth` supporting contract.

use crate::api::error::LwkError;
use crate::contracts::simplicity::{SimplicityArguments, SimplicityTypedValue};

use super::covenant_program::{asset_id_argument, CovenantProgram};
use super::simf::ASSET_AUTH;
use super::transaction::LendingTransaction;

#[derive(Debug, Clone)]
pub struct AssetAuthParameters {
    pub asset_id: String,
    pub asset_amount: u64,
    pub with_asset_burn: bool,
}

/// Asset authorization helper contract.
pub struct AssetAuth {
    program: CovenantProgram,
}

impl AssetAuth {
    pub fn new(parameters: AssetAuthParameters) -> anyhow::Result<Self, LwkError> {
        let arguments = SimplicityArguments::new()
            .add_value("ASSET_ID".into(), asset_id_argument(&parameters.asset_id)?)
            .add_value(
                "ASSET_AMOUNT".into(),
                SimplicityTypedValue::u64(parameters.asset_amount),
            )
            .add_value(
                "WITH_ASSET_BURN".into(),
                SimplicityTypedValue::boolean(parameters.with_asset_burn),
            );
        Ok(Self {
            program: CovenantProgram::load(ASSET_AUTH, &arguments)?,
        })
    }

    pub fn script_hash(&self) -> anyhow::Result<[u8; 32], LwkError> {
        self.program.script_hash()
    }

    pub fn attach_creation(
        &self,
        tx: &mut LendingTransaction,
        asset_id: &str,
        amount: u64,
    ) -> anyhow::Result<(), LwkError> {
        self.program.add_output(tx, asset_id, amount)
    }
}
