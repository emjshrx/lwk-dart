use lwk_wollet::elements;
use lwk_wollet::elements::pset::{Input, Output, PartiallySignedTransaction};
use lwk_wollet::elements::BlockHash;
use lwk_wollet::elements::hex::ToHex;
use lwk_wollet::elements_miniscript::psbt::finalize;
use lwk_wollet::hashes::Hash;
use lwk_wollet::EC;
use std::str::FromStr;
use std::sync::Mutex;

use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut, Script};
use crate::api::error::LwkError;

/// Partially signed Elements transaction.
#[derive(Clone)]
pub struct Pset {
    pub(crate) inner: PartiallySignedTransaction,
}

impl Pset {
    pub fn from_string(base64: String) -> anyhow::Result<Pset, LwkError> {
        Ok(Pset {
            inner: base64.trim().parse()?,
        })
    }

    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    pub fn extract_tx_bytes(&self) -> anyhow::Result<Vec<u8>, LwkError> {
        let tx = self.inner.clone().extract_tx()?;
        Ok(elements::encode::serialize(&tx))
    }

    pub fn finalize(&self) -> anyhow::Result<Vec<u8>, LwkError> {
        let mut pset = self.inner.clone();
        finalize(&mut pset, &EC, BlockHash::all_zeros())?;
        let tx = pset.extract_tx()?;
        Ok(elements::encode::serialize(&tx))
    }

    pub fn combine(&self, other: &Pset) -> anyhow::Result<Pset, LwkError> {
        let mut pset = self.inner.clone();
        pset.merge(other.inner.clone())?;
        Ok(Pset { inner: pset })
    }
}

impl From<PartiallySignedTransaction> for Pset {
    fn from(inner: PartiallySignedTransaction) -> Self {
        Pset { inner }
    }
}

/// Builder for constructing a PSET from scratch.
pub struct PsetBuilder {
    inner: Mutex<Option<PartiallySignedTransaction>>,
}

impl PsetBuilder {
    pub fn new_v2() -> PsetBuilder {
        PsetBuilder {
            inner: Mutex::new(Some(PartiallySignedTransaction::new_v2())),
        }
    }

    pub fn add_input(&self, input: &CovenantPsetInput) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let pset = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetBuilder already consumed".into(),
        })?;
        pset.add_input(input.inner.clone());
        Ok(())
    }

    pub fn add_output(&self, output: &CovenantPsetOutput) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let pset = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetBuilder already consumed".into(),
        })?;
        pset.add_output(output.inner.clone());
        Ok(())
    }

    pub fn set_fallback_locktime(&self, height: u32) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let pset = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetBuilder already consumed".into(),
        })?;
        pset.global.tx_data.fallback_locktime =
            Some(elements::LockTime::from_height(height).map_err(|e| LwkError {
                msg: format!("Invalid lock time height: {e:?}"),
            })?);
        Ok(())
    }

    pub fn build(&self) -> anyhow::Result<Pset, LwkError> {
        let mut lock = self.inner.lock()?;
        let pset = lock.take().ok_or_else(|| LwkError {
            msg: "PsetBuilder already consumed".into(),
        })?;
        Ok(Pset { inner: pset })
    }
}

/// Asset identifiers derived from an issuance input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IssuanceDetails {
    pub asset_id: String,
    pub reissuance_token_id: String,
}

/// PSET input for covenant transaction building.
#[derive(Clone)]
pub struct CovenantPsetInput {
    pub(crate) inner: Input,
}

impl CovenantPsetInput {
    pub fn previous_vout(&self) -> u32 {
        self.inner.previous_output_index
    }
}

/// Builder for PSET inputs.
pub struct PsetInputBuilder {
    inner: Mutex<Option<Input>>,
}

impl PsetInputBuilder {
    pub fn from_prevout(outpoint: &ElementsOutPoint) -> PsetInputBuilder {
        PsetInputBuilder {
            inner: Mutex::new(Some(Input::from_prevout(outpoint.inner))),
        }
    }

    pub fn witness_utxo(&self, utxo: &ElementsTxOut) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetInputBuilder already consumed".into(),
        })?;
        inner.witness_utxo = Some(utxo.inner.clone());
        Ok(())
    }

    pub fn sequence(&self, sequence: u32) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetInputBuilder already consumed".into(),
        })?;
        inner.sequence = Some(elements::Sequence::from_consensus(sequence));
        Ok(())
    }

    pub fn explicit_issuance(
        &self,
        issuance_amount: u64,
        inflation_amount: u64,
        asset_entropy: [u8; 32],
    ) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetInputBuilder already consumed".into(),
        })?;
        inner.issuance_value_amount = Some(issuance_amount);
        inner.issuance_inflation_keys = Some(inflation_amount);
        inner.issuance_asset_entropy = Some(asset_entropy);
        inner.blinded_issuance = Some(0x00);
        Ok(())
    }

    pub fn issuance_details(&self) -> anyhow::Result<IssuanceDetails, LwkError> {
        let lock = self.inner.lock()?;
        let inner = lock.as_ref().ok_or_else(|| LwkError {
            msg: "PsetInputBuilder already consumed".into(),
        })?;
        let (asset_id, token_id) = inner.issuance_ids();
        Ok(IssuanceDetails {
            asset_id: asset_id.to_string(),
            reissuance_token_id: token_id.to_string(),
        })
    }

    pub fn build(&self) -> anyhow::Result<CovenantPsetInput, LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.take().ok_or_else(|| LwkError {
            msg: "PsetInputBuilder already consumed".into(),
        })?;
        Ok(CovenantPsetInput { inner })
    }
}

/// PSET output for covenant transaction building.
#[derive(Clone)]
pub struct CovenantPsetOutput {
    pub(crate) inner: Output,
}

/// Builder for PSET outputs.
pub struct PsetOutputBuilder {
    inner: Mutex<Option<Output>>,
}

impl PsetOutputBuilder {
    pub fn new_explicit(
        script: &Script,
        satoshi: u64,
        asset_id: String,
    ) -> anyhow::Result<PsetOutputBuilder, LwkError> {
        let asset = elements::AssetId::from_str(&asset_id)?;
        Ok(PsetOutputBuilder {
            inner: Mutex::new(Some(Output {
                script_pubkey: script.inner.clone(),
                amount: Some(satoshi),
                asset: Some(asset),
                ..Default::default()
            })),
        })
    }

    pub fn new_op_return(
        data: Vec<u8>,
        satoshi: u64,
        asset_id: String,
    ) -> anyhow::Result<PsetOutputBuilder, LwkError> {
        let script = Script::new_op_return(data);
        Self::new_explicit(&script, satoshi, asset_id)
    }

    pub fn blinder_index(&self, index: u32) -> anyhow::Result<(), LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.as_mut().ok_or_else(|| LwkError {
            msg: "PsetOutputBuilder already consumed".into(),
        })?;
        inner.blinder_index = Some(index);
        Ok(())
    }

    pub fn build(&self) -> anyhow::Result<CovenantPsetOutput, LwkError> {
        let mut lock = self.inner.lock()?;
        let inner = lock.take().ok_or_else(|| LwkError {
            msg: "PsetOutputBuilder already consumed".into(),
        })?;
        Ok(CovenantPsetOutput { inner })
    }
}
