use lwk_simplicity::scripts::{simplicity_leaf_version, tap_data_hash};
use lwk_simplicity::simplicityhl;
use lwk_wollet::elements::taproot;
use lwk_wollet::elements_miniscript::ToPublicKey;
use lwk_wollet::hashes::{sha256, Hash};
use lwk_wollet::{elements, EC};

use crate::contracts::blockdata::{ControlBlock, Script, XOnlyPublicKey};
use crate::api::error::LwkError;
use crate::contracts::simplicity::Cmr;

/// BIP-341 unspendable internal key used by covenant protocols.
pub const UNSPENDABLE_TAPROOT_PUBKEY: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Taproot builder for Simplicity state-management trees.
#[derive(Clone, Debug)]
pub struct StateTaprootBuilder {
    inner: taproot::TaprootBuilder,
}

impl StateTaprootBuilder {
    pub fn new() -> StateTaprootBuilder {
        StateTaprootBuilder {
            inner: taproot::TaprootBuilder::new(),
        }
    }

    pub fn add_simplicity_leaf(
        &self,
        depth: u8,
        cmr: &Cmr,
    ) -> anyhow::Result<StateTaprootBuilder, LwkError> {
        let (script, version) = script_version(cmr.inner());
        let inner = self
            .inner
            .clone()
            .add_leaf_with_ver(usize::from(depth), script, version)?;
        Ok(StateTaprootBuilder { inner })
    }

    pub fn add_data_leaf(
        &self,
        depth: u8,
        data: Vec<u8>,
    ) -> anyhow::Result<StateTaprootBuilder, LwkError> {
        let hash = tap_data_hash(&data);
        let inner = self.inner.clone().add_hidden(usize::from(depth), hash)?;
        Ok(StateTaprootBuilder { inner })
    }

    pub fn add_hidden_hash(
        &self,
        depth: u8,
        hash: Vec<u8>,
    ) -> anyhow::Result<StateTaprootBuilder, LwkError> {
        let arr: [u8; 32] = hash
            .try_into()
            .map_err(|_| LwkError {
                msg: "Hidden hash must be 32 bytes".into(),
            })?;
        let hash = sha256::Hash::from_byte_array(arr);
        let inner = self.inner.clone().add_hidden(usize::from(depth), hash)?;
        Ok(StateTaprootBuilder { inner })
    }

    /// Finalize the taproot tree.
    ///
    /// For stateful covenants, use [`StateTaprootSpendInfo::output_key`] as the
    /// program public key when calling `get_sighash_all` / `finalize_transaction`.
    pub fn finalize(
        &self,
        internal_key: &XOnlyPublicKey,
    ) -> anyhow::Result<StateTaprootSpendInfo, LwkError> {
        let x_only_key = internal_key.to_simplicityhl()?;
        let spend_info = self.inner.clone().finalize(&EC, x_only_key)?;
        Ok(StateTaprootSpendInfo { inner: spend_info })
    }
}

/// Taproot spending information for a Simplicity covenant address.
pub struct StateTaprootSpendInfo {
    pub(crate) inner: taproot::TaprootSpendInfo,
}

impl StateTaprootSpendInfo {
    pub fn output_key(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from(self.inner.output_key().into_inner())
    }

    pub fn output_key_parity(&self) -> u8 {
        self.inner.output_key_parity().to_u8()
    }

    pub fn internal_key(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from(self.inner.internal_key().to_x_only_pubkey())
    }

    pub fn merkle_root(&self) -> Option<Vec<u8>> {
        self.inner
            .merkle_root()
            .map(|root| root.to_byte_array().to_vec())
    }

    pub fn control_block(&self, cmr: &Cmr) -> anyhow::Result<ControlBlock, LwkError> {
        self.inner
            .control_block(&script_version(cmr.inner()))
            .map(ControlBlock::from)
            .ok_or_else(|| LwkError {
                msg: "CMR is not part of this taproot spend info".into(),
            })
    }

    pub fn script_pubkey(&self) -> Script {
        elements::Script::new_v1_p2tr_tweaked(self.inner.output_key()).into()
    }
}

fn script_version(cmr: simplicityhl::simplicity::Cmr) -> (elements::Script, taproot::LeafVersion) {
    let script = elements::Script::from(cmr.as_ref().to_vec());
    (script, simplicity_leaf_version())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_state_management_flow() {
        let mut state = [0u8; 32];
        state[31] = 1;

        let cmr = Cmr::from_string(
            "cbd8d3d0cc95384237c1bf20334c30b579f22058563c37731a3ab2bc76d5a248".into(),
        )
        .unwrap();
        let internal_key = XOnlyPublicKey::from_string(UNSPENDABLE_TAPROOT_PUBKEY.into()).unwrap();

        let expected_script_pubkey =
            "51205920ca2ef73fa8c0378b50e99e4518b72fee1c413c1f1c52acbde479b3ec0a21";

        let builder = StateTaprootBuilder::new()
            .add_simplicity_leaf(1, &cmr)
            .unwrap()
            .add_data_leaf(1, state.to_vec())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();

        assert_eq!(builder.script_pubkey().to_hex(), expected_script_pubkey);
        assert_ne!(
            builder.output_key().to_string_repr(),
            internal_key.to_string_repr()
        );
        assert!(builder.control_block(&cmr).is_ok());
    }
}
