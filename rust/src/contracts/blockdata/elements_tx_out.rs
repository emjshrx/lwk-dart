use lwk_wollet::elements::{self, confidential, pset::serialize::Deserialize, TxOutWitness};
use lwk_wollet::elements::hex::ToHex;
use std::str::FromStr;

use crate::contracts::blockdata::Script;
use crate::api::error::LwkError;

/// Elements transaction output for covenant UTXO construction.
#[derive(Clone, Debug)]
pub struct ElementsTxOut {
    pub(crate) inner: elements::TxOut,
}

impl ElementsTxOut {
    pub fn from_explicit(
        script_pubkey_hex: String,
        asset_id: String,
        satoshi: u64,
    ) -> anyhow::Result<ElementsTxOut, LwkError> {
        let script = Script::from_hex(script_pubkey_hex)?;
        let asset = elements::AssetId::from_str(&asset_id)?;
        Ok(ElementsTxOut {
            inner: elements::TxOut {
                script_pubkey: script.inner,
                asset: confidential::Asset::Explicit(asset),
                value: confidential::Value::Explicit(satoshi),
                nonce: confidential::Nonce::Null,
                witness: TxOutWitness::default(),
            },
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<ElementsTxOut, LwkError> {
        Ok(ElementsTxOut {
            inner: elements::TxOut::deserialize(&bytes)?,
        })
    }

    pub fn script_pubkey_hex(&self) -> String {
        self.inner.script_pubkey.to_hex()
    }

    pub fn asset_id(&self) -> Option<String> {
        self.inner.asset.explicit().map(|a| a.to_string())
    }

    pub fn value(&self) -> Option<u64> {
        self.inner.value.explicit()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        elements::encode::serialize(&self.inner)
    }
}

impl From<elements::TxOut> for ElementsTxOut {
    fn from(inner: elements::TxOut) -> Self {
        ElementsTxOut { inner }
    }
}

impl From<&ElementsTxOut> for elements::TxOut {
    fn from(value: &ElementsTxOut) -> Self {
        value.inner.clone()
    }
}
