use flutter_rust_bridge::frb;

use crate::api::types::LiquidNetwork;

#[derive(Clone, Debug)]
pub struct IssuanceFactoryParameters {
    pub issuing_utxos_count: u8,
    pub reissuance_flags: u64,
    pub network: LiquidNetwork,
}

impl From<IssuanceFactoryParameters> for crate::contracts::lending::IssuanceFactoryParameters {
    fn from(value: IssuanceFactoryParameters) -> Self {
        Self {
            issuing_utxos_count: value.issuing_utxos_count,
            reissuance_flags: value.reissuance_flags,
            network: value.network,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IssuanceDetails {
    pub asset_id: String,
    pub reissuance_token_id: String,
}

impl From<crate::contracts::pset::IssuanceDetails> for IssuanceDetails {
    fn from(value: crate::contracts::pset::IssuanceDetails) -> Self {
        Self {
            asset_id: value.asset_id,
            reissuance_token_id: value.reissuance_token_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UtilityNftIssuanceResult {
    pub borrower_nft_asset_id: String,
    pub policy_asset_id: String,
}

impl From<crate::contracts::lending::UtilityNftIssuanceResult> for UtilityNftIssuanceResult {
    fn from(value: crate::contracts::lending::UtilityNftIssuanceResult) -> Self {
        Self {
            borrower_nft_asset_id: value.borrower_nft_asset_id,
            policy_asset_id: value.policy_asset_id,
        }
    }
}

#[frb]
#[derive(Clone, Debug)]
pub enum IssuanceFactoryWitnessBranchKind {
    IssueAssets,
    RemoveFactory,
}

#[derive(Clone, Debug)]
pub struct IssuanceFactoryWitnessBranch {
    pub kind: IssuanceFactoryWitnessBranchKind,
    pub output_index: u32,
}

impl From<crate::contracts::lending::IssuanceFactoryWitnessBranch> for IssuanceFactoryWitnessBranch {
    fn from(value: crate::contracts::lending::IssuanceFactoryWitnessBranch) -> Self {
        match value {
            crate::contracts::lending::IssuanceFactoryWitnessBranch::IssueAssets { output_index } => {
                Self {
                    kind: IssuanceFactoryWitnessBranchKind::IssueAssets,
                    output_index,
                }
            }
            crate::contracts::lending::IssuanceFactoryWitnessBranch::RemoveFactory { output_index } => {
                Self {
                    kind: IssuanceFactoryWitnessBranchKind::RemoveFactory,
                    output_index,
                }
            }
        }
    }
}

#[frb]
pub struct TryFromIssuanceFactoryResult {
    factory: super::factory::IssuanceFactory,
    factory_asset_id: String,
}

impl TryFromIssuanceFactoryResult {
    pub(crate) fn new(factory: super::factory::IssuanceFactory, factory_asset_id: String) -> Self {
        Self {
            factory,
            factory_asset_id,
        }
    }

    #[frb(sync)]
    pub fn factory(self) -> super::factory::IssuanceFactory {
        self.factory
    }

    #[frb(sync)]
    pub fn factory_asset_id(&self) -> String {
        self.factory_asset_id.clone()
    }
}
