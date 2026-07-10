//! Lending offer parameter types ported from upstream `lending-contracts`.

use crate::api::types::LiquidNetwork;

use super::asset_auth::{AssetAuth, AssetAuthParameters};
use super::asset_auth_vault::{ActiveAssetAuthVault, FinalizedAssetAuthVault, FinalizedAssetAuthVaultParameters};
use crate::contracts::simplicity::{SimplicityArguments, SimplicityTypedValue};

use super::covenant_program::{asset_id_argument, CovenantProgram, ProgramId};
use super::simf::LENDING;
use super::utils::apply_basis_points;

/// Economic terms of a lending offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfferParameters {
    pub collateral_amount: u64,
    pub principal_amount: u64,
    pub loan_expiration_time: u32,
    pub principal_interest_rate: u16,
}

impl OfferParameters {
    pub fn total_fee(&self) -> u64 {
        apply_basis_points(self.principal_amount, self.principal_interest_rate)
    }

    pub fn total_amount_to_repay(&self) -> u64 {
        self.principal_amount + self.total_fee()
    }
}

/// Full protocol parameter set for a lending offer covenant.
#[derive(Debug, Clone)]
pub struct LendingOfferParameters {
    pub collateral_asset_id: String,
    pub principal_asset_id: String,
    pub borrower_nft_asset_id: String,
    pub lender_nft_asset_id: String,
    pub protocol_fee_keeper_asset_id: String,
    pub offer_parameters: OfferParameters,
    pub network: LiquidNetwork,
}

impl LendingOfferParameters {
    pub fn build_lending_arguments(&self) -> anyhow::Result<SimplicityArguments, crate::api::error::LwkError> {
        let offer = &self.offer_parameters;
        Ok(SimplicityArguments::new()
            .add_value("COLLATERAL_ASSET_ID".into(), asset_id_argument(&self.collateral_asset_id)?)
            .add_value("PRINCIPAL_ASSET_ID".into(), asset_id_argument(&self.principal_asset_id)?)
            .add_value(
                "BORROWER_NFT_ASSET_ID".into(),
                asset_id_argument(&self.borrower_nft_asset_id)?,
            )
            .add_value(
                "LENDER_NFT_ASSET_ID".into(),
                asset_id_argument(&self.lender_nft_asset_id)?,
            )
            .add_value(
                "COLLATERAL_AMOUNT".into(),
                SimplicityTypedValue::u64(offer.collateral_amount),
            )
            .add_value(
                "PRINCIPAL_AMOUNT".into(),
                SimplicityTypedValue::u64(offer.principal_amount),
            )
            .add_value(
                "PRINCIPAL_INTEREST_RATE".into(),
                SimplicityTypedValue::u64(u64::from(offer.principal_interest_rate)),
            )
            .add_value(
                "LOAN_EXPIRATION_TIME".into(),
                SimplicityTypedValue::u32(offer.loan_expiration_time),
            )
            .add_value(
                "LENDER_VAULT_COV_HASH".into(),
                SimplicityTypedValue::u256(self.active_lender_vault()?.script_hash()?.to_vec())?,
            )
            .add_value(
                "FINALIZED_LENDER_VAULT_COV_HASH".into(),
                SimplicityTypedValue::u256(self.finalized_lender_vault()?.script_hash()?.to_vec())?,
            )
            .add_value(
                "PROTOCOL_FEE_VAULT_COV_HASH".into(),
                SimplicityTypedValue::u256(self.active_protocol_fee_vault()?.script_hash()?.to_vec())?,
            )
            .add_value(
                "FINALIZED_PROTOCOL_FEE_VAULT_COV_HASH".into(),
                SimplicityTypedValue::u256(
                    self.finalized_protocol_fee_vault()?.script_hash()?.to_vec(),
                )?,
            )
            .add_value(
                "PRINCIPAL_OUTPUT_SCRIPT_HASH".into(),
                SimplicityTypedValue::u256(self.principal_output_asset_auth()?.script_hash()?.to_vec())?,
            ))
    }

    fn principal_output_asset_auth(&self) -> anyhow::Result<AssetAuth, crate::api::error::LwkError> {
        AssetAuth::new(AssetAuthParameters {
            asset_id: self.borrower_nft_asset_id.clone(),
            asset_amount: 1,
            with_asset_burn: false,
        })
    }

    fn active_lender_vault(&self) -> anyhow::Result<ActiveAssetAuthVault, crate::api::error::LwkError> {
        ActiveAssetAuthVault::from_finalized(self.finalized_lender_vault()?)
    }

    fn active_protocol_fee_vault(
        &self,
    ) -> anyhow::Result<ActiveAssetAuthVault, crate::api::error::LwkError> {
        ActiveAssetAuthVault::from_finalized(self.finalized_protocol_fee_vault()?)
    }

    fn finalized_lender_vault(
        &self,
    ) -> anyhow::Result<FinalizedAssetAuthVault, crate::api::error::LwkError> {
        FinalizedAssetAuthVault::new(FinalizedAssetAuthVaultParameters {
            vault_asset_id: self.principal_asset_id.clone(),
            keeper_asset_id: self.lender_nft_asset_id.clone(),
            keeper_min_asset_amount: 1,
            with_keeper_asset_burn: true,
            supplier_asset_id: self.borrower_nft_asset_id.clone(),
            with_supplier_asset_burn: true,
        })
    }

    fn finalized_protocol_fee_vault(
        &self,
    ) -> anyhow::Result<FinalizedAssetAuthVault, crate::api::error::LwkError> {
        FinalizedAssetAuthVault::new(FinalizedAssetAuthVaultParameters {
            vault_asset_id: self.principal_asset_id.clone(),
            keeper_asset_id: self.protocol_fee_keeper_asset_id.clone(),
            keeper_min_asset_amount: 1,
            with_keeper_asset_burn: false,
            supplier_asset_id: self.borrower_nft_asset_id.clone(),
            with_supplier_asset_burn: true,
        })
    }

    pub fn lending_program_id(&self) -> ProgramId {
        CovenantProgram::program_id(LENDING)
    }
}
