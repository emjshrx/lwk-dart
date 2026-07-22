//! Simplicity lending port — orchestration over the internal Simplicity stack.
//!
//! Contract sources live in the `vendor/simplicity-lending` git submodule.

#![allow(dead_code, unused_imports)]

mod asset_auth;
mod asset_auth_vault;
mod factory;
mod offer;
mod script_auth;
mod transaction;

pub(crate) mod simf;

pub(crate) use asset_auth::AssetAuth;
pub(crate) use asset_auth_vault::AssetAuthVault;
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
}
