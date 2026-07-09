use lwk_simplicity::simplicityhl::simplicity;
use std::str::FromStr;

use crate::api::error::LwkError;

/// Commitment Merkle root of a compiled Simplicity program.
#[derive(Clone, Copy, Debug)]
pub struct Cmr {
    pub(crate) inner: simplicity::Cmr,
}

impl Cmr {
    pub fn from_string(s: String) -> anyhow::Result<Cmr, LwkError> {
        Ok(Cmr {
            inner: simplicity::Cmr::from_str(&s)?,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Cmr, LwkError> {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| LwkError {
                msg: "CMR must be 32 bytes".into(),
            })?;
        Ok(Cmr {
            inner: simplicity::Cmr::from_byte_array(arr),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_byte_array().to_vec()
    }

    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    pub(crate) fn inner(&self) -> simplicity::Cmr {
        self.inner
    }
}

impl From<simplicity::Cmr> for Cmr {
    fn from(inner: simplicity::Cmr) -> Self {
        Cmr { inner }
    }
}
