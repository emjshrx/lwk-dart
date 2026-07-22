//! `.simf` sources from the `vendor/simplicity-lending` git submodule.
//!
//! Pin upgrades by bumping the submodule commit in the parent repo.

pub const LENDING: &str = include_str!(
    "../../../../../vendor/simplicity-lending/crates/contracts/simf/lending.simf"
);
pub const ASSET_AUTH: &str = include_str!(
    "../../../../../vendor/simplicity-lending/crates/contracts/simf/asset_auth.simf"
);
pub const ASSET_AUTH_VAULT: &str = include_str!(
    "../../../../../vendor/simplicity-lending/crates/contracts/simf/asset_auth_vault.simf"
);
pub const ISSUANCE_FACTORY: &str = include_str!(
    "../../../../../vendor/simplicity-lending/crates/contracts/simf/issuance_factory.simf"
);
pub const SCRIPT_AUTH: &str = include_str!(
    "../../../../../vendor/simplicity-lending/crates/contracts/simf/script_auth.simf"
);
