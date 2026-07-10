//! Vendored `.simf` sources from simplicity-lending.
//!
//! Pinned rev: see `LENDING_CONTRACTS_REV` at the crate root.

pub const LENDING: &str = include_str!("lending.simf");
pub const ASSET_AUTH: &str = include_str!("asset_auth.simf");
pub const ASSET_AUTH_VAULT: &str = include_str!("asset_auth_vault.simf");
pub const ISSUANCE_FACTORY: &str = include_str!("issuance_factory.simf");
pub const SCRIPT_AUTH: &str = include_str!("script_auth.simf");
