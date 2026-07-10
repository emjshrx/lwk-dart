//! Simplicity lending port — orchestration over the internal Simplicity stack.
//!
//! Contract sources are vendored under `simf/` and pinned to the rev in
//! `rust/LENDING_CONTRACTS_REV`.

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

/// Upstream `simplicity-lending` git rev the vendored `.simf` sources were copied from.
pub const LENDING_CONTRACTS_REV: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LENDING_CONTRACTS_REV"));

#[cfg(test)]
mod tests {
    use super::simf;
    use super::LENDING_CONTRACTS_REV;
    use crate::contracts::simplicity::{SimplicityArguments, SimplicityProgram, SimplicityTypedValue};

    #[test]
    fn lending_contracts_rev_is_pinned_git_sha() {
        let rev = LENDING_CONTRACTS_REV.trim();
        assert_eq!(rev.len(), 40);
        assert!(rev.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn vendored_simf_sources_are_present() {
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
