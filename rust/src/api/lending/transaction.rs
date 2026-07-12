use flutter_rust_bridge::frb;

use crate::api::error::LwkError;
use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut};
use crate::contracts::lending::LendingTransaction as InnerLendingTransaction;

use super::types::IssuanceDetails;

#[frb(opaque)]
pub struct LendingTransaction {
    inner: InnerLendingTransaction,
}

impl LendingTransaction {
    #[frb(sync)]
    pub fn new() -> LendingTransaction {
        Self {
            inner: InnerLendingTransaction::new(),
        }
    }

    #[frb(sync)]
    pub fn n_inputs(&self) -> u32 {
        self.inner.n_inputs()
    }

    #[frb(sync)]
    pub fn n_outputs(&self) -> u32 {
        self.inner.n_outputs()
    }

    #[frb(sync)]
    pub fn add_wallet_input(
        &mut self,
        txid: String,
        vout: u32,
        witness_utxo_script_hex: String,
        witness_utxo_asset_id: String,
        witness_utxo_amount: u64,
    ) -> anyhow::Result<(), LwkError> {
        let outpoint = ElementsOutPoint::from_parts(txid, vout)?;
        let utxo = ElementsTxOut::from_explicit(
            witness_utxo_script_hex,
            witness_utxo_asset_id,
            witness_utxo_amount,
        )?;
        self.inner.add_wallet_input(&outpoint, &utxo)?;
        Ok(())
    }

    #[frb(sync)]
    pub fn add_issuance_input(
        &mut self,
        txid: String,
        vout: u32,
        witness_utxo_script_hex: String,
        witness_utxo_asset_id: String,
        witness_utxo_amount: u64,
        issuance_amount: u64,
        inflation_amount: u64,
        asset_entropy: Vec<u8>,
    ) -> anyhow::Result<IssuanceDetails, LwkError> {
        let outpoint = ElementsOutPoint::from_parts(txid, vout)?;
        let utxo = ElementsTxOut::from_explicit(
            witness_utxo_script_hex,
            witness_utxo_asset_id,
            witness_utxo_amount,
        )?;
        let entropy: [u8; 32] = asset_entropy
            .try_into()
            .map_err(|_| LwkError {
                msg: "asset_entropy must be 32 bytes".into(),
            })?;
        let details = self.inner.add_issuance_input(
            &outpoint,
            &utxo,
            issuance_amount,
            inflation_amount,
            entropy,
        )?;
        Ok(details.into())
    }

    #[frb(sync)]
    pub fn add_explicit_output(
        &mut self,
        script_hex: String,
        satoshi: u64,
        asset_id: String,
    ) -> anyhow::Result<u32, LwkError> {
        let script = crate::contracts::blockdata::Script::from_hex(script_hex)?;
        self.inner
            .add_explicit_output(&script, satoshi, asset_id)
    }

    #[frb(sync)]
    pub fn build(&mut self) -> anyhow::Result<String, LwkError> {
        let replacement = InnerLendingTransaction::new();
        let old = std::mem::replace(&mut self.inner, replacement);
        let pset = old.build()?;
        Ok(pset.to_string_repr())
    }

    pub(crate) fn inner_mut(&mut self) -> &mut InnerLendingTransaction {
        &mut self.inner
    }
}

impl Default for LendingTransaction {
    fn default() -> Self {
        Self::new()
    }
}
