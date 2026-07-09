use lwk_wollet::elements::{self, hashes::Hash};
use std::str::FromStr;

use crate::api::error::LwkError;

/// Elements outpoint for covenant transaction building.
#[derive(Clone, Debug)]
pub struct ElementsOutPoint {
    pub(crate) inner: elements::OutPoint,
}

impl ElementsOutPoint {
    pub fn from_string(s: String) -> anyhow::Result<ElementsOutPoint, LwkError> {
        Ok(ElementsOutPoint {
            inner: s.parse()?,
        })
    }

    pub fn from_parts(txid_hex: String, vout: u32) -> anyhow::Result<ElementsOutPoint, LwkError> {
        let txid = elements::Txid::from_str(&txid_hex)?;
        Ok(ElementsOutPoint {
            inner: elements::OutPoint::new(txid, vout),
        })
    }

    pub fn txid(&self) -> String {
        self.inner.txid.to_string()
    }

    pub fn vout(&self) -> u32 {
        self.inner.vout
    }

    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }
}

impl From<elements::OutPoint> for ElementsOutPoint {
    fn from(inner: elements::OutPoint) -> Self {
        ElementsOutPoint { inner }
    }
}

impl From<&ElementsOutPoint> for elements::OutPoint {
    fn from(value: &ElementsOutPoint) -> Self {
        value.inner
    }
}
