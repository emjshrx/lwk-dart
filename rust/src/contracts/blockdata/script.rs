use lwk_wollet::elements::pset::serialize::Deserialize;
use lwk_wollet::elements::{self, hashes::sha256, hashes::Hash, hex::ToHex};
use lwk_wollet::hashes::hex::FromHex;

use crate::api::error::LwkError;

/// A Liquid script.
#[derive(Clone, Debug)]
pub struct Script {
    pub(crate) inner: elements::Script,
}

impl Script {
    pub fn from_hex(hex: String) -> anyhow::Result<Script, LwkError> {
        let bytes = Vec::<u8>::from_hex(&hex)?;
        Ok(Script {
            inner: elements::Script::deserialize(&bytes)?,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Script, LwkError> {
        Ok(Script {
            inner: elements::Script::deserialize(&bytes)?,
        })
    }

    pub fn empty() -> Script {
        Script {
            inner: elements::Script::new(),
        }
    }

    pub fn new_op_return(data: Vec<u8>) -> Script {
        Script {
            inner: elements::Script::new_op_return(&data),
        }
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    /// SHA256 of the script consensus bytes (jet script hash).
    pub fn jet_sha256_hex(&self) -> String {
        sha256::Hash::hash(self.inner.as_bytes()).to_hex()
    }

    pub fn is_provably_unspendable(&self) -> bool {
        self.inner.is_provably_unspendable()
    }
}

impl From<elements::Script> for Script {
    fn from(inner: elements::Script) -> Self {
        Script { inner }
    }
}

/// SHA256 hash of a script's consensus bytes (32 bytes).
pub fn hash_script(script: &Script) -> Vec<u8> {
    sha256::Hash::hash(script.inner.as_bytes())
        .to_byte_array()
        .to_vec()
}
