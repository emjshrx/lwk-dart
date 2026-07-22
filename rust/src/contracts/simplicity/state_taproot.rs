use lwk_simplicity::scripts::{simplicity_leaf_version, tap_data_hash};
use lwk_simplicity::simplicityhl::simplicity::Cmr;
use lwk_wollet::elements::bitcoin::XOnlyPublicKey;
use lwk_wollet::elements::hex::ToHex;
use lwk_wollet::elements::taproot::{self, ControlBlock};
use lwk_wollet::elements_miniscript::ToPublicKey;
use lwk_wollet::{elements, EC};

use crate::api::error::LwkError;
use crate::contracts::simplicity::utils::xonly_to_simplicityhl;

/// BIP-341 unspendable internal key used by covenant protocols.
pub const UNSPENDABLE_TAPROOT_PUBKEY: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Taproot builder for Simplicity state-management trees (program leaf + data leaves).
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
        cmr: Cmr,
    ) -> anyhow::Result<StateTaprootBuilder, LwkError> {
        let (script, version) = script_version(cmr);
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

    /// Finalize the taproot tree.
    ///
    /// For stateful covenants, use [`StateTaprootSpendInfo::output_key`] as the
    /// program public key when calling `get_sighash_all` / `finalize_transaction`.
    pub fn finalize(
        &self,
        internal_key: &XOnlyPublicKey,
    ) -> anyhow::Result<StateTaprootSpendInfo, LwkError> {
        let x_only_key = xonly_to_simplicityhl(internal_key)?;
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
        self.inner.output_key().into_inner()
    }

    pub fn internal_key(&self) -> XOnlyPublicKey {
        self.inner.internal_key().to_x_only_pubkey()
    }

    pub fn control_block(&self, cmr: Cmr) -> anyhow::Result<ControlBlock, LwkError> {
        self.inner
            .control_block(&script_version(cmr))
            .ok_or_else(|| LwkError {
                msg: "CMR is not part of this taproot spend info".into(),
            })
    }

    pub fn script_pubkey(&self) -> elements::Script {
        elements::Script::new_v1_p2tr_tweaked(self.inner.output_key())
    }

    pub fn script_pubkey_hex(&self) -> String {
        self.script_pubkey().to_hex()
    }
}

fn script_version(cmr: Cmr) -> (elements::Script, taproot::LeafVersion) {
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

        let cmr = Cmr::from_str(
            "cbd8d3d0cc95384237c1bf20334c30b579f22058563c37731a3ab2bc76d5a248",
        )
        .unwrap();
        let internal_key = XOnlyPublicKey::from_str(UNSPENDABLE_TAPROOT_PUBKEY).unwrap();

        let expected_script_pubkey =
            "51205920ca2ef73fa8c0378b50e99e4518b72fee1c413c1f1c52acbde479b3ec0a21";

        let builder = StateTaprootBuilder::new()
            .add_simplicity_leaf(1, cmr)
            .unwrap()
            .add_data_leaf(1, state.to_vec())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();

        assert_eq!(builder.script_pubkey_hex(), expected_script_pubkey);
        assert_ne!(builder.output_key().to_string(), internal_key.to_string());
        assert!(builder.control_block(cmr).is_ok());
    }
}
