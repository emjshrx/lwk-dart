//! Internal `script_auth` supporting contract.

use crate::api::error::LwkError;
use crate::contracts::simplicity::{SimplicityArguments, SimplicityTypedValue};

use super::covenant_program::CovenantProgram;
use super::simf::SCRIPT_AUTH;
use super::transaction::LendingTransaction;

#[derive(Debug, Clone, Copy)]
pub struct ScriptAuthParameters {
    pub script_hash: [u8; 32],
}

/// Script-hash authorization helper contract.
pub struct ScriptAuth {
    program: CovenantProgram,
}

impl ScriptAuth {
    pub fn new(parameters: ScriptAuthParameters) -> anyhow::Result<Self, LwkError> {
        let arguments = SimplicityArguments::new().add_value(
            "SCRIPT_HASH".into(),
            SimplicityTypedValue::u256(parameters.script_hash.to_vec())?,
        );
        Ok(Self {
            program: CovenantProgram::load(SCRIPT_AUTH, &arguments)?,
        })
    }

    pub fn from_script_hash(script_hash: [u8; 32]) -> anyhow::Result<Self, LwkError> {
        Self::new(ScriptAuthParameters { script_hash })
    }

    pub fn attach_creation(
        &self,
        tx: &mut LendingTransaction,
        asset_id: &str,
        amount: u64,
    ) -> anyhow::Result<(), LwkError> {
        self.program.add_output(tx, asset_id, amount)
    }

    pub(crate) fn script_pubkey(&self) -> anyhow::Result<crate::contracts::blockdata::Script, LwkError> {
        self.program.script_pubkey()
    }
}
