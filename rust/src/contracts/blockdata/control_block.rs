use lwk_wollet::elements::taproot;

use crate::contracts::blockdata::XOnlyPublicKey;
use crate::api::error::LwkError;

/// Taproot control block for script-path spending.
#[derive(Clone, Debug)]
pub struct ControlBlock {
    pub(crate) inner: taproot::ControlBlock,
}

impl ControlBlock {
    pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<ControlBlock, LwkError> {
        taproot::ControlBlock::from_slice(&bytes)
            .map(ControlBlock::from)
            .map_err(|e| LwkError {
                msg: format!("Invalid control block: {e:?}"),
            })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.serialize()
    }

    pub fn leaf_version(&self) -> u8 {
        self.inner.leaf_version.as_u8()
    }

    pub fn internal_key(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from(self.inner.internal_key)
    }

    pub fn output_key_parity(&self) -> u8 {
        self.inner.output_key_parity.to_u8()
    }
}

impl From<taproot::ControlBlock> for ControlBlock {
    fn from(inner: taproot::ControlBlock) -> Self {
        ControlBlock { inner }
    }
}
