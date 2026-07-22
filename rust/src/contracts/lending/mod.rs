//! Simplicity lending port — orchestration over the internal Simplicity stack.
//!
//! Contract sources live in the `vendor/simplicity-lending` git submodule.

#![allow(dead_code, unused_imports)]

mod asset_auth;
mod asset_auth_vault;
mod covenant_program;
mod factory;
mod metadata;
mod offer;
mod params;
mod script_auth;
mod transaction;
mod utils;

pub(crate) mod simf;

pub(crate) use asset_auth::AssetAuth;
pub(crate) use asset_auth_vault::{ActiveAssetAuthVault, FinalizedAssetAuthVault};
pub(crate) use factory::IssuanceFactory;
pub(crate) use offer::LendingOffer;
pub(crate) use script_auth::ScriptAuth;
pub(crate) use transaction::LendingTransaction;

#[cfg(test)]
mod tests {
    use super::simf;
    use crate::contracts::simplicity::{SimplicityArguments, SimplicityProgram, SimplicityTypedValue};

    #[test]
    fn submodule_simf_sources_are_present() {
        for source in [
            simf::LENDING,
            simf::ASSET_AUTH,
            simf::ASSET_AUTH_VAULT,
            simf::ISSUANCE_FACTORY,
            simf::SCRIPT_AUTH,
        ] {
            assert!(!source.is_empty());
            assert!(source.contains("fn main()"));
        }
    }

    #[test]
    fn script_auth_simf_compiles() {
        let args = SimplicityArguments::new().add_value(
            "SCRIPT_HASH".into(),
            SimplicityTypedValue::u256(vec![0u8; 32]).unwrap(),
        );
        SimplicityProgram::load_with_arguments(simf::SCRIPT_AUTH.to_string(), &args).unwrap();
    }

    #[test]
    fn lending_simf_compiles_with_fixture_arguments() {
        use super::offer::LendingOffer;
        use crate::api::types::LiquidNetwork;
        use crate::contracts::lending::params::{LendingOfferParameters, OfferParameters};

        let parameters = LendingOfferParameters {
            collateral_asset_id:
                "144c654344aa71608c11a65921ccb7398bd9734057d5c98e002dd31308e14d29".into(),
            principal_asset_id: hex::encode([1u8; 32]),
            borrower_nft_asset_id: hex::encode([2u8; 32]),
            lender_nft_asset_id: hex::encode([3u8; 32]),
            protocol_fee_keeper_asset_id: hex::encode([4u8; 32]),
            offer_parameters: OfferParameters {
                collateral_amount: 3000,
                principal_amount: 10_000,
                loan_expiration_time: 100_060,
                principal_interest_rate: 1000,
            },
            network: LiquidNetwork::Testnet,
        };

        LendingOffer::new_pending(parameters).unwrap();
    }
}
