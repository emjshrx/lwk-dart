use std::sync::{Mutex, OnceLock};

use flutter_rust_bridge::frb;

use super::super::error::LwkError;
use super::super::types::LiquidNetwork;

/// Default testnet indexer base URL (local demo server).
pub const DEFAULT_TESTNET_INDEXER_BASE_URL: &str = "http://localhost:8000";

static LENDING_CONFIG: OnceLock<Mutex<Option<LendingConfig>>> = OnceLock::new();

fn lending_config_mutex() -> &'static Mutex<Option<LendingConfig>> {
    LENDING_CONFIG.get_or_init(|| Mutex::new(None))
}

/// Module-level lending configuration, set via [`lending_init`].
#[derive(Clone, Debug, PartialEq)]
pub struct LendingConfig {
    pub network: LiquidNetwork,
    pub allow_mainnet: bool,
    pub indexer_base_url: Option<String>,
}

/// Initialize or replace lending module configuration.
#[frb]
pub fn lending_init(config: LendingConfig) -> Result<(), LwkError> {
    set_lending_config(config)
}

pub(crate) fn set_lending_config(config: LendingConfig) -> Result<(), LwkError> {
    let mut guard = lending_config_mutex().lock()?;
    *guard = Some(config);
    Ok(())
}

pub(crate) fn with_lending_config<T>(
    f: impl FnOnce(&LendingConfig) -> Result<T, LwkError>,
) -> Result<T, LwkError> {
    let config = {
        let guard = lending_config_mutex().lock()?;
        guard
            .as_ref()
            .ok_or_else(|| {
                LwkError::from("Lending not initialized; call lending_init first".to_string())
            })?
            .clone()
    };
    ensure_network_allowed(&config)?;
    f(&config)
}

pub(crate) fn indexer_base_url(config: &LendingConfig) -> Result<String, LwkError> {
    if let Some(url) = &config.indexer_base_url {
        return Ok(url.trim_end_matches('/').to_string());
    }

    match config.network {
        LiquidNetwork::Testnet => Ok(DEFAULT_TESTNET_INDEXER_BASE_URL.to_string()),
        LiquidNetwork::Mainnet => Err(LwkError::from(
            "indexer_base_url is required for mainnet".to_string(),
        )),
    }
}

pub(crate) fn ensure_network_allowed(config: &LendingConfig) -> Result<(), LwkError> {
    if config.network == LiquidNetwork::Mainnet && !config.allow_mainnet {
        return Err(LwkError::from(
            "Mainnet lending requires allow_mainnet: true".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testnet_config(indexer_base_url: Option<&str>) -> LendingConfig {
        LendingConfig {
            network: LiquidNetwork::Testnet,
            allow_mainnet: false,
            indexer_base_url: indexer_base_url.map(str::to_string),
        }
    }

    #[test]
    fn mainnet_without_allow_mainnet_is_rejected() {
        let config = LendingConfig {
            network: LiquidNetwork::Mainnet,
            allow_mainnet: false,
            indexer_base_url: Some("http://indexer".to_string()),
        };

        let err = ensure_network_allowed(&config).expect_err("should reject");
        assert!(err.msg.contains("allow_mainnet"));
    }

    #[test]
    fn testnet_uses_default_indexer_url() {
        let url = indexer_base_url(&testnet_config(None)).expect("default url");
        assert_eq!(url, DEFAULT_TESTNET_INDEXER_BASE_URL);
    }

    #[test]
    fn indexer_base_url_trims_trailing_slash() {
        let config = testnet_config(Some("http://indexer/"));
        let url = indexer_base_url(&config).expect("url");
        assert_eq!(url, "http://indexer");
    }

    #[test]
    fn mainnet_requires_explicit_indexer_url() {
        let config = LendingConfig {
            network: LiquidNetwork::Mainnet,
            allow_mainnet: true,
            indexer_base_url: None,
        };

        let err = indexer_base_url(&config).expect_err("should require url");
        assert!(err.msg.contains("indexer_base_url"));
    }
}
