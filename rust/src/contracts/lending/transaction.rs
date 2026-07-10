//! Lending-specific transaction builder wrapping internal PSET machinery.

use crate::api::error::LwkError;
use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut, Script};
use crate::contracts::pset::{
    CovenantPsetInput, CovenantPsetOutput, Pset, PsetBuilder, PsetInputBuilder, PsetOutputBuilder,
};

/// Passed to `LendingOffer::attach*` methods; callers extract a partial PSET via
/// [`build`](Self::build) for wallet signing and broadcast.
pub struct LendingTransaction {
    builder: PsetBuilder,
    n_inputs: u32,
    n_outputs: u32,
}

impl LendingTransaction {
    pub fn new() -> Self {
        Self {
            builder: PsetBuilder::new_v2(),
            n_inputs: 0,
            n_outputs: 0,
        }
    }

    pub fn n_inputs(&self) -> u32 {
        self.n_inputs
    }

    pub fn n_outputs(&self) -> u32 {
        self.n_outputs
    }

    /// Wallet-owned input (native signing handled separately via `Wallet::sign_tx`).
    pub fn add_wallet_input(
        &mut self,
        outpoint: &ElementsOutPoint,
        witness_utxo: &ElementsTxOut,
    ) -> anyhow::Result<(), LwkError> {
        let input_builder = PsetInputBuilder::from_prevout(outpoint);
        input_builder.witness_utxo(witness_utxo)?;
        self.add_covenant_input(&input_builder.build()?)
    }

    /// Wallet-owned input with an explicit nSequence (e.g. liquidation timelocks).
    pub fn add_wallet_input_with_sequence(
        &mut self,
        outpoint: &ElementsOutPoint,
        witness_utxo: &ElementsTxOut,
        sequence: u32,
    ) -> anyhow::Result<(), LwkError> {
        let input_builder = PsetInputBuilder::from_prevout(outpoint);
        input_builder.witness_utxo(witness_utxo)?;
        input_builder.sequence(sequence)?;
        self.add_covenant_input(&input_builder.build()?)
    }

    /// Covenant/program input (Simplicity witness attached during finalization).
    pub fn add_covenant_input(&mut self, input: &CovenantPsetInput) -> anyhow::Result<(), LwkError> {
        self.builder.add_input(input)?;
        self.n_inputs += 1;
        Ok(())
    }

    /// Explicit-value output; returns the output index before insertion.
    pub fn add_explicit_output(
        &mut self,
        script: &Script,
        satoshi: u64,
        asset_id: String,
    ) -> anyhow::Result<u32, LwkError> {
        let output_index = self.n_outputs;
        let output = PsetOutputBuilder::new_explicit(script, satoshi, asset_id)?.build()?;
        self.add_output(&output)?;
        Ok(output_index)
    }

    /// OP_RETURN output; returns the output index before insertion.
    pub fn add_op_return_output(
        &mut self,
        data: Vec<u8>,
        satoshi: u64,
        asset_id: String,
    ) -> anyhow::Result<u32, LwkError> {
        let output_index = self.n_outputs;
        let output = PsetOutputBuilder::new_op_return(data, satoshi, asset_id)?.build()?;
        self.add_output(&output)?;
        Ok(output_index)
    }

    pub fn add_output(&mut self, output: &CovenantPsetOutput) -> anyhow::Result<(), LwkError> {
        self.builder.add_output(output)?;
        self.n_outputs += 1;
        Ok(())
    }

    pub fn set_fallback_locktime(&self, height: u32) -> anyhow::Result<(), LwkError> {
        self.builder.set_fallback_locktime(height)
    }

    /// Upstream invariant: repayment attachments require at least one prior input.
    pub fn require_prior_attachment(&self) -> anyhow::Result<(), LwkError> {
        if self.n_inputs == 0 {
            return Err(LwkError {
                msg: "Repayment can't be first attachment in transaction".into(),
            });
        }
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<Pset, LwkError> {
        self.builder.build()
    }
}

impl Default for LendingTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwk_wollet::elements::pset::serialize::Deserialize;
    use lwk_wollet::elements::Transaction;

    const POLICY_ASSET: &str =
        "144c654344aa71608c11a65921ccb7398bd9734057d5c98e002dd31308e14d29";

    fn sample_outpoint() -> ElementsOutPoint {
        ElementsOutPoint::from_parts("ab".repeat(32), 0).unwrap()
    }

    fn sample_utxo() -> ElementsTxOut {
        ElementsTxOut::from_explicit(
            Script::new_op_return(vec![0x01]).to_hex(),
            POLICY_ASSET.into(),
            100_000,
        )
        .unwrap()
    }

    #[test]
    fn new_transaction_is_empty() {
        let tx = LendingTransaction::new();
        assert_eq!(tx.n_inputs(), 0);
        assert_eq!(tx.n_outputs(), 0);
    }

    #[test]
    fn add_wallet_input_and_output_updates_counts() {
        let mut tx = LendingTransaction::new();
        tx.add_wallet_input(&sample_outpoint(), &sample_utxo()).unwrap();
        assert_eq!(tx.n_inputs(), 1);

        tx.add_explicit_output(&Script::new_op_return(vec![0x01]), 1, POLICY_ASSET.into())
            .unwrap();
        assert_eq!(tx.n_outputs(), 1);
    }

    #[test]
    fn build_produces_pset_with_expected_io_counts() {
        let mut tx = LendingTransaction::new();
        tx.add_wallet_input(&sample_outpoint(), &sample_utxo()).unwrap();
        tx.add_explicit_output(&Script::new_op_return(vec![0x01]), 1, POLICY_ASSET.into())
            .unwrap();

        let pset = tx.build().unwrap();
        let tx_bytes = pset.extract_tx_bytes().unwrap();
        let extracted = Transaction::deserialize(&tx_bytes).unwrap();

        assert_eq!(extracted.input.len(), 1);
        assert_eq!(extracted.output.len(), 1);
    }

    #[test]
    fn require_prior_attachment_rejects_empty_transaction() {
        let tx = LendingTransaction::new();
        let err = tx.require_prior_attachment().unwrap_err();
        assert!(err.msg.contains("Repayment can't be first attachment"));
    }

    #[test]
    fn require_prior_attachment_accepts_transaction_with_inputs() {
        let mut tx = LendingTransaction::new();
        tx.add_wallet_input(&sample_outpoint(), &sample_utxo()).unwrap();
        tx.require_prior_attachment().unwrap();
    }

    #[test]
    fn build_empty_transaction_produces_pset() {
        let tx = LendingTransaction::new();
        tx.build().unwrap();
    }
}
