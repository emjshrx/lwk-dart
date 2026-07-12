//! Utility NFT issuance orchestration ported from upstream lending setup flow.

use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut};
use crate::contracts::lending::factory::{IssuanceFactory, IssuanceFactoryError};
use crate::contracts::lending::transaction::LendingTransaction;
use crate::contracts::simplicity::SimplicityWitnessValues;

pub(crate) struct UtilityNftIssuanceResult {
    pub borrower_nft_asset_id: String,
    pub witness_values: SimplicityWitnessValues,
    pub policy_asset_id: String,
}

/// Port of upstream `factory.attach_assets_issuance` in the pending-offer setup flow.
pub(crate) fn attach_utility_nft_issuance(
    factory: &IssuanceFactory,
    tx: &mut LendingTransaction,
    factory_utxo_outpoint: &ElementsOutPoint,
    factory_utxo: &ElementsTxOut,
    asset_entropy: [u8; 32],
    policy_asset_id: &str,
) -> Result<UtilityNftIssuanceResult, IssuanceFactoryError> {
    factory.attach_assets_issuance(
        tx,
        factory_utxo_outpoint,
        factory_utxo,
        1,
        0,
        asset_entropy,
        policy_asset_id,
    )
}
