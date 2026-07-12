//! Issuance factory orchestration for protocol-specific utility NFT setup.

mod error;
mod metadata;
mod params;
mod witness;

pub(crate) use error::IssuanceFactoryError;
pub(crate) use params::IssuanceFactoryParameters;
pub(crate) use witness::IssuanceFactoryWitnessBranch;

use lwk_wollet::elements::pset::serialize::Deserialize;
use lwk_wollet::elements::{Transaction, Txid};

use crate::api::types::LiquidNetwork;
use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut, Script};
use crate::contracts::lending::program::{
    burn_op_return_script, op_return_payload, program_id_from_source, program_script_pubkey,
    ProgramId,
};
use crate::contracts::lending::simf;
use crate::contracts::lending::transaction::LendingTransaction;
use crate::contracts::lending::utility_nft::UtilityNftIssuanceResult;
use crate::contracts::pset::IssuanceDetails;
use crate::contracts::simplicity::{SimplicityArguments, SimplicityProgram, SimplicityTypedValue};

const CREATION_METADATA_OUTPUT_INDEX: usize = 2;

/// Protocol-specific utility NFT issuance factory.
pub(crate) struct IssuanceFactory {
    program: SimplicityProgram,
    parameters: IssuanceFactoryParameters,
    script_pubkey: Script,
}

impl IssuanceFactory {
    pub(crate) fn new(parameters: IssuanceFactoryParameters) -> Result<Self, IssuanceFactoryError> {
        let arguments = SimplicityArguments::new()
            .add_value(
                "ISSUING_UTXOS_COUNT".into(),
                SimplicityTypedValue::u8(parameters.issuing_utxos_count),
            )
            .add_value(
                "REISSUANCE_FLAGS".into(),
                SimplicityTypedValue::u64(parameters.reissuance_flags),
            );
        let program =
            SimplicityProgram::load_with_arguments(simf::ISSUANCE_FACTORY.to_string(), &arguments)?;
        let script_pubkey = program_script_pubkey(&program, parameters.network)?;

        Ok(Self {
            program,
            parameters,
            script_pubkey,
        })
    }

    pub(crate) fn try_from_tx(
        tx_bytes: &[u8],
        network: LiquidNetwork,
    ) -> Result<(Self, String), IssuanceFactoryError> {
        let tx = Transaction::deserialize(tx_bytes)
            .map_err(|e| IssuanceFactoryError::Internal(e.to_string()))?;
        Self::try_from_transaction(&tx, network)
    }

    pub(crate) fn try_from_transaction(
        tx: &Transaction,
        network: LiquidNetwork,
    ) -> Result<(Self, String), IssuanceFactoryError> {
        let txid = tx.txid();

        if tx.output.len() <= CREATION_METADATA_OUTPUT_INDEX
            || !tx.output[CREATION_METADATA_OUTPUT_INDEX]
                .script_pubkey
                .is_op_return()
        {
            return Err(IssuanceFactoryError::NotAnIssuanceFactoryCreationTx(txid));
        }

        let op_return_bytes = op_return_payload(&tx.output[CREATION_METADATA_OUTPUT_INDEX].script_pubkey)
            .ok_or(IssuanceFactoryError::NotAnIssuanceFactoryCreationTx(txid))?;

        let creation_metadata = Self::decode_metadata_op_return(op_return_bytes)?;

        if creation_metadata.program_id.as_slice() != Self::program_id().as_slice() {
            return Err(IssuanceFactoryError::NotAnIssuanceFactoryCreationTx(txid));
        }

        let issuance_factory_parameters = IssuanceFactoryParameters {
            issuing_utxos_count: creation_metadata.issuing_utxos_count,
            reissuance_flags: creation_metadata.reissuance_flags,
            network,
        };

        let issuance_factory = Self::new(issuance_factory_parameters)?;
        let factory_asset_id =
            Self::validate_creation_outputs(tx, &issuance_factory.script_pubkey, txid)?;

        Ok((issuance_factory, factory_asset_id))
    }

    pub(crate) fn get_parameters(&self) -> &IssuanceFactoryParameters {
        &self.parameters
    }

    pub(crate) fn script_pubkey_hex(&self) -> String {
        self.script_pubkey.to_hex()
    }

    pub(crate) fn program_id_hex(&self) -> String {
        hex::encode(Self::program_id())
    }

    pub(crate) fn program(&self) -> &SimplicityProgram {
        &self.program
    }

    pub(crate) fn attach_creation(
        &self,
        tx: &mut LendingTransaction,
        factory_asset_id: &str,
        factory_asset_amount: u64,
        policy_asset_id: &str,
    ) -> Result<(), IssuanceFactoryError> {
        self.add_program_output(tx, factory_asset_id, factory_asset_amount)?;

        let op_return_data = self.encode_metadata_op_return();
        tx.add_op_return_output(op_return_data, 0, policy_asset_id.into())?;

        Ok(())
    }

    pub(crate) fn attach_assets_issuance(
        &self,
        tx: &mut LendingTransaction,
        program_utxo_outpoint: &ElementsOutPoint,
        program_utxo: &ElementsTxOut,
        issuance_amount: u64,
        inflation_amount: u64,
        asset_entropy: [u8; 32],
        policy_asset_id: &str,
    ) -> Result<UtilityNftIssuanceResult, IssuanceFactoryError> {
        if tx.n_inputs() == 0 {
            return Err(IssuanceFactoryError::AssetsIssuanceRequiresPriorInput);
        }

        let auth_nft_output_index = tx.n_outputs().saturating_sub(1);
        let issuance_factory_asset = program_utxo
            .asset_id()
            .ok_or_else(|| IssuanceFactoryError::Internal("program UTXO must be explicit".into()))?;

        let witness_branch = IssuanceFactoryWitnessBranch::IssueAssets {
            output_index: auth_nft_output_index,
        };

        let _issuance_details = tx.add_program_issuance_input(
            program_utxo_outpoint,
            program_utxo,
            issuance_amount,
            inflation_amount,
            asset_entropy,
        )?;

        self.add_program_output(tx, &issuance_factory_asset, 1)?;

        Ok(UtilityNftIssuanceResult {
            borrower_nft_asset_id: _issuance_details.asset_id,
            witness_values: witness_branch.build_witness_values(),
            policy_asset_id: policy_asset_id.to_string(),
        })
    }

    pub(crate) fn attach_factory_removing(
        &self,
        tx: &mut LendingTransaction,
        program_utxo_outpoint: &ElementsOutPoint,
        program_utxo: &ElementsTxOut,
        _policy_asset_id: &str,
    ) -> Result<IssuanceFactoryWitnessBranch, IssuanceFactoryError> {
        let issuance_factory_asset = program_utxo
            .asset_id()
            .ok_or_else(|| IssuanceFactoryError::Internal("program UTXO must be explicit".into()))?;

        let issuance_factory_output_index = tx.n_outputs();
        let witness_branch = IssuanceFactoryWitnessBranch::RemoveFactory {
            output_index: issuance_factory_output_index,
        };

        tx.add_program_input(program_utxo_outpoint, program_utxo)?;

        tx.add_explicit_output(
            &burn_op_return_script(),
            1,
            issuance_factory_asset.clone(),
        )?;
        tx.add_explicit_output(
            &burn_op_return_script(),
            1,
            issuance_factory_asset,
        )?;

        Ok(witness_branch)
    }

    fn add_program_output(
        &self,
        tx: &mut LendingTransaction,
        asset_id: &str,
        asset_amount: u64,
    ) -> Result<(), IssuanceFactoryError> {
        tx.add_explicit_output(&self.script_pubkey, asset_amount, asset_id.into())?;
        Ok(())
    }

    fn validate_creation_outputs(
        tx: &Transaction,
        program_script_pubkey: &Script,
        txid: Txid,
    ) -> Result<String, IssuanceFactoryError> {
        let program_script = &program_script_pubkey.inner;

        let factory_asset_ids = tx
            .output
            .iter()
            .filter(|output| output.script_pubkey == *program_script)
            .filter_map(|output| {
                let asset_id = output.asset.explicit()?;
                let amount = output.value.explicit()?;
                (amount == 1).then_some(asset_id)
            })
            .collect::<Vec<_>>();

        let &[factory_asset_id] = factory_asset_ids.as_slice() else {
            return Err(IssuanceFactoryError::NotAnIssuanceFactoryCreationTx(txid));
        };

        let auth_output_count = tx
            .output
            .iter()
            .filter(|output| {
                let (Some(asset_id), Some(amount)) =
                    (output.asset.explicit(), output.value.explicit())
                else {
                    return false;
                };

                asset_id == factory_asset_id
                    && amount == 1
                    && !output.script_pubkey.is_op_return()
                    && output.script_pubkey != *program_script
            })
            .count();

        if auth_output_count != 1 {
            return Err(IssuanceFactoryError::NotAnIssuanceFactoryCreationTx(txid));
        }

        Ok(factory_asset_id.to_string())
    }

    pub(crate) fn program_id() -> ProgramId {
        program_id_from_source(simf::ISSUANCE_FACTORY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::L_TEST_ASSET_ID;
    use crate::contracts::blockdata::ElementsOutPoint;
    use crate::contracts::lending::utility_nft::attach_utility_nft_issuance;
    use lwk_wollet::elements::pset::serialize::Deserialize;

    const POLICY_ASSET: &str = L_TEST_ASSET_ID;

    fn sample_parameters() -> IssuanceFactoryParameters {
        IssuanceFactoryParameters {
            issuing_utxos_count: 3,
            reissuance_flags: 0x0102_0304_0506_0708,
            network: LiquidNetwork::Testnet,
        }
    }

    fn sample_wallet_utxo() -> (ElementsOutPoint, ElementsTxOut) {
        let outpoint = ElementsOutPoint::from_parts("ab".repeat(32), 0).unwrap();
        let utxo = ElementsTxOut::from_explicit(
            Script::new_op_return(vec![0x01]).to_hex(),
            POLICY_ASSET.into(),
            100_000,
        )
        .unwrap();
        (outpoint, utxo)
    }

    #[test]
    fn issuance_factory_simf_compiles_with_parameters() {
        let factory = IssuanceFactory::new(sample_parameters()).unwrap();
        assert!(!factory.script_pubkey_hex().is_empty());
        assert_eq!(factory.program_id_hex().len(), 8);
    }

    fn sample_auth_script() -> Script {
        Script::from_hex("0014d2bcde17e7744f6377466ca1bd35d212954674c8".into()).unwrap()
    }

    #[test]
    fn attach_creation_produces_expected_metadata_output() {
        let factory = IssuanceFactory::new(sample_parameters()).unwrap();
        let (wallet_outpoint, wallet_utxo) = sample_wallet_utxo();

        let mut tx = LendingTransaction::new();
        let issuance = tx
            .add_issuance_input(
                &wallet_outpoint,
                &wallet_utxo,
                2,
                0,
                [7u8; 32],
            )
            .unwrap();

        tx.add_explicit_output(&sample_auth_script(), 1, issuance.asset_id.clone())
            .unwrap();

        factory
            .attach_creation(&mut tx, &issuance.asset_id, 1, POLICY_ASSET)
            .unwrap();

        let pset = tx.build().unwrap();
        let extracted = Transaction::deserialize(&pset.extract_tx_bytes().unwrap()).unwrap();

        assert_eq!(extracted.output.len(), 3);
        assert!(extracted.output[2].script_pubkey.is_op_return());

        let metadata = op_return_payload(&extracted.output[2].script_pubkey).unwrap();
        assert_eq!(metadata.len(), 13);
        assert_eq!(&metadata[0..4], IssuanceFactory::program_id().as_slice());
        assert_eq!(metadata[4], sample_parameters().issuing_utxos_count);
        assert_eq!(
            &metadata[5..13],
            sample_parameters().reissuance_flags.to_le_bytes()
        );
    }

    #[test]
    fn try_from_tx_decodes_creation_metadata() {
        let parameters = sample_parameters();
        let factory = IssuanceFactory::new(parameters).unwrap();
        let (wallet_outpoint, wallet_utxo) = sample_wallet_utxo();

        let mut tx = LendingTransaction::new();
        let issuance = tx
            .add_issuance_input(&wallet_outpoint, &wallet_utxo, 2, 0, [9u8; 32])
            .unwrap();
        tx.add_explicit_output(&sample_auth_script(), 1, issuance.asset_id.clone())
            .unwrap();
        factory
            .attach_creation(&mut tx, &issuance.asset_id, 1, POLICY_ASSET)
            .unwrap();

        let tx_bytes = tx.build().unwrap().extract_tx_bytes().unwrap();
        let (decoded_factory, decoded_asset_id) =
            IssuanceFactory::try_from_tx(&tx_bytes, LiquidNetwork::Testnet).unwrap();

        assert_eq!(
            decoded_factory.get_parameters().issuing_utxos_count,
            parameters.issuing_utxos_count
        );
        assert_eq!(
            decoded_factory.get_parameters().reissuance_flags,
            parameters.reissuance_flags
        );
        assert_eq!(decoded_asset_id, issuance.asset_id);
    }

    #[test]
    fn attach_factory_removing_adds_burn_outputs() {
        let factory = IssuanceFactory::new(sample_parameters()).unwrap();
        let factory_asset_id = "11".repeat(32);
        let factory_outpoint = ElementsOutPoint::from_parts("cd".repeat(32), 0).unwrap();
        let factory_utxo = ElementsTxOut::from_explicit(
            factory.script_pubkey_hex(),
            factory_asset_id.clone(),
            1,
        )
        .unwrap();

        let mut tx = LendingTransaction::new();
        let witness_branch = factory
            .attach_factory_removing(&mut tx, &factory_outpoint, &factory_utxo, POLICY_ASSET)
            .unwrap();

        assert_eq!(tx.n_inputs(), 1);
        assert_eq!(tx.n_outputs(), 2);
        assert!(matches!(
            witness_branch,
            IssuanceFactoryWitnessBranch::RemoveFactory { output_index: 0 }
        ));

        let extracted = Transaction::deserialize(&tx.build().unwrap().extract_tx_bytes().unwrap())
            .unwrap();
        assert_eq!(extracted.output.len(), 2);
        assert!(extracted.output[0].script_pubkey.is_op_return());
        assert!(extracted.output[1].script_pubkey.is_op_return());
    }

    #[test]
    fn utility_nft_issuance_orchestration_shape() {
        let factory = IssuanceFactory::new(IssuanceFactoryParameters {
            issuing_utxos_count: 2,
            reissuance_flags: 0,
            network: LiquidNetwork::Testnet,
        })
        .unwrap();

        let factory_asset_id = "22".repeat(32);
        let factory_outpoint = ElementsOutPoint::from_parts("ef".repeat(32), 0).unwrap();
        let factory_utxo = ElementsTxOut::from_explicit(
            factory.script_pubkey_hex(),
            factory_asset_id.clone(),
            1,
        )
        .unwrap();

        let auth_outpoint = ElementsOutPoint::from_parts("12".repeat(32), 1).unwrap();
        let auth_utxo = ElementsTxOut::from_explicit(
            Script::new_op_return(vec![0x02]).to_hex(),
            factory_asset_id.clone(),
            1,
        )
        .unwrap();

        let mut tx = LendingTransaction::new();
        tx.add_wallet_input(&auth_outpoint, &auth_utxo).unwrap();
        tx.add_explicit_output(
            &Script::new_op_return(vec![0x03]),
            1,
            factory_asset_id.clone(),
        )
        .unwrap();

        let result = attach_utility_nft_issuance(
            &factory,
            &mut tx,
            &factory_outpoint,
            &factory_utxo,
            [3u8; 32],
            POLICY_ASSET,
        )
        .unwrap();

        assert!(!result.borrower_nft_asset_id.is_empty());
        assert_eq!(tx.n_inputs(), 2);
        assert_eq!(tx.n_outputs(), 2);
    }
}
