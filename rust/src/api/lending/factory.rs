use flutter_rust_bridge::frb;

use crate::api::error::LwkError;
use crate::contracts::lending::{
    attach_utility_nft_issuance, IssuanceFactory as InnerIssuanceFactory,
};
use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut};

use super::transaction::LendingTransaction;
use super::types::{
    IssuanceFactoryParameters, IssuanceFactoryWitnessBranch, TryFromIssuanceFactoryResult,
    UtilityNftIssuanceResult,
};

#[frb(opaque)]
pub struct IssuanceFactory {
    inner: InnerIssuanceFactory,
}

impl IssuanceFactory {
    #[frb(sync)]
    pub fn new(parameters: IssuanceFactoryParameters) -> anyhow::Result<IssuanceFactory, LwkError> {
        Ok(Self {
            inner: InnerIssuanceFactory::new(parameters.into())?,
        })
    }

    #[frb(sync)]
    pub fn try_from_tx(
        tx_bytes: Vec<u8>,
        network: crate::api::types::LiquidNetwork,
    ) -> anyhow::Result<TryFromIssuanceFactoryResult, LwkError> {
        let (factory, factory_asset_id) =
            InnerIssuanceFactory::try_from_tx(&tx_bytes, network)?;
        Ok(TryFromIssuanceFactoryResult::new(
            IssuanceFactory { inner: factory },
            factory_asset_id,
        ))
    }

    #[frb(sync)]
    pub fn script_pubkey_hex(&self) -> String {
        self.inner.script_pubkey_hex()
    }

    #[frb(sync)]
    pub fn program_id_hex(&self) -> String {
        self.inner.program_id_hex()
    }

    #[frb(sync)]
    pub fn issuing_utxos_count(&self) -> u8 {
        self.inner.get_parameters().issuing_utxos_count
    }

    #[frb(sync)]
    pub fn reissuance_flags(&self) -> u64 {
        self.inner.get_parameters().reissuance_flags
    }

    #[frb(sync)]
    pub fn attach_creation(
        &self,
        tx: &mut LendingTransaction,
        factory_asset_id: String,
        factory_asset_amount: u64,
        policy_asset_id: String,
    ) -> anyhow::Result<(), LwkError> {
        self.inner
            .attach_creation(
                tx.inner_mut(),
                &factory_asset_id,
                factory_asset_amount,
                &policy_asset_id,
            )?;
        Ok(())
    }

    #[frb(sync)]
    pub fn attach_factory_removing(
        &self,
        tx: &mut LendingTransaction,
        program_utxo_txid: String,
        program_utxo_vout: u32,
        program_utxo_script_hex: String,
        program_utxo_asset_id: String,
        program_utxo_amount: u64,
        policy_asset_id: String,
    ) -> anyhow::Result<IssuanceFactoryWitnessBranch, LwkError> {
        let outpoint = ElementsOutPoint::from_parts(program_utxo_txid, program_utxo_vout)?;
        let utxo = ElementsTxOut::from_explicit(
            program_utxo_script_hex,
            program_utxo_asset_id,
            program_utxo_amount,
        )?;
        let branch = self.inner.attach_factory_removing(
            tx.inner_mut(),
            &outpoint,
            &utxo,
            &policy_asset_id,
        )?;
        Ok(branch.into())
    }

    #[frb(sync)]
    pub fn attach_utility_nft_issuance(
        &self,
        tx: &mut LendingTransaction,
        factory_utxo_txid: String,
        factory_utxo_vout: u32,
        factory_utxo_script_hex: String,
        factory_utxo_asset_id: String,
        factory_utxo_amount: u64,
        asset_entropy: Vec<u8>,
        policy_asset_id: String,
    ) -> anyhow::Result<UtilityNftIssuanceResult, LwkError> {
        let outpoint = ElementsOutPoint::from_parts(factory_utxo_txid, factory_utxo_vout)?;
        let utxo = ElementsTxOut::from_explicit(
            factory_utxo_script_hex,
            factory_utxo_asset_id,
            factory_utxo_amount,
        )?;
        let entropy: [u8; 32] = asset_entropy
            .try_into()
            .map_err(|_| LwkError {
                msg: "asset_entropy must be 32 bytes".into(),
            })?;
        let result = attach_utility_nft_issuance(
            &self.inner,
            tx.inner_mut(),
            &outpoint,
            &utxo,
            entropy,
            &policy_asset_id,
        )?;
        Ok(result.into())
    }
}
