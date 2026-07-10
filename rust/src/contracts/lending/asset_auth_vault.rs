//! Internal `asset_auth_vault` supporting contract.

use crate::api::error::LwkError;
use crate::contracts::simplicity::{SimplicityArguments, SimplicityTypedValue};

use super::covenant_program::{asset_id_argument, CovenantProgram};
use super::simf::ASSET_AUTH_VAULT;

#[derive(Debug, Clone)]
pub struct FinalizedAssetAuthVaultParameters {
    pub vault_asset_id: String,
    pub keeper_asset_id: String,
    pub supplier_asset_id: String,
    pub keeper_min_asset_amount: u64,
    pub with_keeper_asset_burn: bool,
    pub with_supplier_asset_burn: bool,
}

/// Finalized asset authorization vault helper contract.
pub struct FinalizedAssetAuthVault {
    program: CovenantProgram,
    parameters: FinalizedAssetAuthVaultParameters,
}

impl FinalizedAssetAuthVault {
    pub fn new(parameters: FinalizedAssetAuthVaultParameters) -> anyhow::Result<Self, LwkError> {
        Ok(Self {
            program: CovenantProgram::load(
                ASSET_AUTH_VAULT,
                &build_arguments(&parameters, false, [0u8; 32])?,
            )?,
            parameters,
        })
    }

    pub fn script_hash(&self) -> anyhow::Result<[u8; 32], LwkError> {
        self.program.script_hash()
    }

    pub fn parameters(&self) -> &FinalizedAssetAuthVaultParameters {
        &self.parameters
    }
}

/// Active asset authorization vault helper contract.
pub struct ActiveAssetAuthVault {
    program: CovenantProgram,
}

impl ActiveAssetAuthVault {
    pub fn from_finalized(
        finalized: FinalizedAssetAuthVault,
    ) -> anyhow::Result<Self, LwkError> {
        let finalized_hash = finalized.script_hash()?;
        let parameters = finalized.parameters();
        Ok(Self {
            program: CovenantProgram::load(
                ASSET_AUTH_VAULT,
                &build_arguments(parameters, true, finalized_hash)?,
            )?,
        })
    }

    pub fn script_hash(&self) -> anyhow::Result<[u8; 32], LwkError> {
        self.program.script_hash()
    }
}

fn build_arguments(
    parameters: &FinalizedAssetAuthVaultParameters,
    is_active: bool,
    finalized_vault_cov_hash: [u8; 32],
) -> anyhow::Result<SimplicityArguments, LwkError> {
    Ok(SimplicityArguments::new()
        .add_value(
            "VAULT_ASSET_ID".into(),
            asset_id_argument(&parameters.vault_asset_id)?,
        )
        .add_value(
            "KEEPER_AUTH_ASSET_ID".into(),
            asset_id_argument(&parameters.keeper_asset_id)?,
        )
        .add_value(
            "SUPPLIER_AUTH_ASSET_ID".into(),
            asset_id_argument(&parameters.supplier_asset_id)?,
        )
        .add_value(
            "KEEPER_AUTH_ASSET_AMOUNT".into(),
            SimplicityTypedValue::u64(parameters.keeper_min_asset_amount),
        )
        .add_value(
            "FINALIZED_VAULT_COV_HASH".into(),
            SimplicityTypedValue::u256(finalized_vault_cov_hash.to_vec())?,
        )
        .add_value("IS_ACTIVE".into(), SimplicityTypedValue::boolean(is_active))
        .add_value(
            "WITH_KEEPER_ASSET_BURN".into(),
            SimplicityTypedValue::boolean(parameters.with_keeper_asset_burn),
        )
        .add_value(
            "WITH_SUPPLIER_ASSET_BURN".into(),
            SimplicityTypedValue::boolean(parameters.with_supplier_asset_burn),
        ))
}
