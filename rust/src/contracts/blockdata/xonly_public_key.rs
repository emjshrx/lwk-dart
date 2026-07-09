use lwk_wollet::elements::bitcoin::XOnlyPublicKey as ElementsXOnlyPublicKey;
use std::str::FromStr;

use crate::api::error::LwkError;

/// An x-only public key for Taproot / Simplicity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XOnlyPublicKey {
    pub(crate) inner: ElementsXOnlyPublicKey,
}

impl XOnlyPublicKey {
    pub fn from_string(s: String) -> anyhow::Result<XOnlyPublicKey, LwkError> {
        Ok(XOnlyPublicKey {
            inner: ElementsXOnlyPublicKey::from_str(&s)?,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<XOnlyPublicKey, LwkError> {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| LwkError {
                msg: "XOnlyPublicKey must be 32 bytes".into(),
            })?;
        Ok(XOnlyPublicKey {
            inner: ElementsXOnlyPublicKey::from_slice(&arr)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.serialize().to_vec()
    }

    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    pub(crate) fn to_simplicityhl(
        &self,
    ) -> anyhow::Result<lwk_simplicity::simplicityhl::simplicity::bitcoin::XOnlyPublicKey, LwkError>
    {
        Ok(lwk_simplicity::simplicityhl::simplicity::bitcoin::XOnlyPublicKey::from_slice(
            &self.to_bytes(),
        )?)
    }
}

impl From<ElementsXOnlyPublicKey> for XOnlyPublicKey {
    fn from(inner: ElementsXOnlyPublicKey) -> Self {
        XOnlyPublicKey { inner }
    }
}
