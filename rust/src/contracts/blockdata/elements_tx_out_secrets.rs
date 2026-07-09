use lwk_wollet::elements::{self, confidential};
use std::str::FromStr;

use crate::api::error::LwkError;

/// Unblinded output secrets for external UTXO construction.
#[derive(Clone, Debug)]
pub struct ElementsTxOutSecrets {
    pub(crate) inner: elements::TxOutSecrets,
}

impl ElementsTxOutSecrets {
    pub fn from_explicit(asset_id: String, value: u64) -> anyhow::Result<ElementsTxOutSecrets, LwkError> {
        let asset = elements::AssetId::from_str(&asset_id)?;
        Ok(ElementsTxOutSecrets {
            inner: elements::TxOutSecrets::new(
                asset,
                confidential::AssetBlindingFactor::zero(),
                value,
                confidential::ValueBlindingFactor::zero(),
            ),
        })
    }

    pub fn asset_id(&self) -> String {
        self.inner.asset.to_string()
    }

    pub fn value(&self) -> u64 {
        self.inner.value
    }
}

impl From<elements::TxOutSecrets> for ElementsTxOutSecrets {
    fn from(inner: elements::TxOutSecrets) -> Self {
        ElementsTxOutSecrets { inner }
    }
}

impl From<&ElementsTxOutSecrets> for elements::TxOutSecrets {
    fn from(value: &ElementsTxOutSecrets) -> Self {
        value.inner
    }
}
