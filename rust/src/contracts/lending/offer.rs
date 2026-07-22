//! Lending offer orchestration — pending creation spike (#4).

use crate::api::error::LwkError;

use super::covenant_program::CovenantProgram;
use super::metadata::LendingOfferCreationMetadata;
use super::params::{LendingOfferParameters, OfferParameters};
use super::script_auth::ScriptAuth;
use super::simf::LENDING;
use super::transaction::LendingTransaction;

/// On-chain storage slots for a lending offer covenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LendingOfferStorage {
    pub is_active: bool,
    pub current_debt: u64,
}

impl LendingOfferStorage {
    fn storage_slots(&self) -> Vec<[u8; 32]> {
        let mut is_active_slot = [0u8; 32];
        is_active_slot[31] = u8::from(self.is_active);

        let mut debt_slot = [0u8; 32];
        debt_slot[24..32].copy_from_slice(&self.current_debt.to_be_bytes());

        vec![is_active_slot, debt_slot]
    }
}

/// Stateful lending offer (pending → active → repaid/liquidated).
pub struct LendingOffer {
    program: CovenantProgram,
    parameters: LendingOfferParameters,
    storage: LendingOfferStorage,
}

impl LendingOffer {
    pub fn new_pending(parameters: LendingOfferParameters) -> anyhow::Result<Self, LwkError> {
        let storage = LendingOfferStorage {
            is_active: false,
            current_debt: parameters.offer_parameters.total_amount_to_repay(),
        };
        Self::new(parameters, storage)
    }

    fn new(parameters: LendingOfferParameters, storage: LendingOfferStorage) -> anyhow::Result<Self, LwkError> {
        let arguments = parameters.build_lending_arguments()?;
        let program = CovenantProgram::load(LENDING, &arguments)?.with_storage(storage.storage_slots());
        Ok(Self {
            program,
            parameters,
            storage,
        })
    }

    pub fn parameters(&self) -> &LendingOfferParameters {
        &self.parameters
    }

    pub fn is_pending_offer(&self) -> bool {
        !self.storage.is_active
    }

    pub(crate) fn script_hash(&self) -> anyhow::Result<[u8; 32], LwkError> {
        self.program.script_hash()
    }

    pub(crate) fn script_pubkey(&self) -> anyhow::Result<crate::contracts::blockdata::Script, LwkError> {
        self.program.script_pubkey()
    }

    pub fn attach_creation(&self, tx: &mut LendingTransaction) -> anyhow::Result<(), LwkError> {
        if self.is_pending_offer() {
            let lender_nft_script_auth =
                ScriptAuth::from_script_hash(self.program.script_hash()?)?;
            lender_nft_script_auth.attach_creation(
                tx,
                &self.parameters.lender_nft_asset_id,
                1,
            )?;

            let metadata = self.build_creation_metadata()?;
            tx.add_metadata_output(metadata.encode())?;
        }

        self.program.add_output(
            tx,
            &self.parameters.collateral_asset_id,
            self.parameters.offer_parameters.collateral_amount,
        )
    }

    pub fn program_id() -> [u8; 4] {
        CovenantProgram::program_id(LENDING)
    }

    pub fn decode_creation_metadata(
        op_return_bytes: &[u8],
    ) -> anyhow::Result<LendingOfferCreationMetadata, LwkError> {
        LendingOfferCreationMetadata::decode(op_return_bytes)
    }

    fn build_creation_metadata(&self) -> anyhow::Result<LendingOfferCreationMetadata, LwkError> {
        let principal_asset_id = hex::decode(&self.parameters.principal_asset_id).map_err(|e| LwkError {
            msg: format!("invalid principal asset id hex: {e}"),
        })?;
        let principal_asset_id: [u8; 32] = principal_asset_id.try_into().map_err(|_| LwkError {
            msg: "principal asset id must be 32 bytes".into(),
        })?;

        Ok(LendingOfferCreationMetadata::new(
            Self::program_id(),
            principal_asset_id,
            self.parameters.offer_parameters,
        ))
    }
}

#[cfg(test)]
mod tests {
    use lwk_wollet::elements::hex::ToHex;
    use lwk_wollet::elements::pset::serialize::Deserialize;
    use lwk_wollet::elements::Transaction;

    use crate::api::types::LiquidNetwork;

    use super::*;

    const POLICY_ASSET: &str =
        "144c654344aa71608c11a65921ccb7398bd9734057d5c98e002dd31308e14d29";

    fn sample_asset(byte: u8) -> String {
        hex::encode([byte; 32])
    }

    fn sample_parameters() -> LendingOfferParameters {
        LendingOfferParameters {
            collateral_asset_id: POLICY_ASSET.into(),
            principal_asset_id: sample_asset(0x01),
            borrower_nft_asset_id: sample_asset(0x02),
            lender_nft_asset_id: sample_asset(0x03),
            protocol_fee_keeper_asset_id: sample_asset(0x04),
            offer_parameters: OfferParameters {
                collateral_amount: 3000,
                principal_amount: 10_000,
                loan_expiration_time: 100_060,
                principal_interest_rate: 1000,
            },
            network: LiquidNetwork::Testnet,
        }
    }

    fn op_return_payload(script_hex: &str) -> Vec<u8> {
        let script = crate::contracts::blockdata::Script::from_hex(script_hex.to_string()).unwrap();
        let bytes = script.to_bytes();
        // OP_RETURN <push>
        bytes[2..].to_vec()
    }

    #[test]
    fn new_pending_compiles_lending_program() {
        let offer = LendingOffer::new_pending(sample_parameters()).unwrap();
        assert!(offer.is_pending_offer());
        assert_eq!(
            offer.storage.current_debt,
            offer.parameters.offer_parameters.total_amount_to_repay()
        );
        assert!(!offer.script_pubkey().unwrap().to_hex().is_empty());
    }

    #[test]
    fn attach_creation_produces_expected_pending_offer_outputs() {
        use crate::contracts::blockdata::Script;

        let parameters = sample_parameters();
        let offer = LendingOffer::new_pending(parameters.clone()).unwrap();
        let lender_nft_script =
            ScriptAuth::from_script_hash(offer.script_hash().unwrap())
                .unwrap()
                .script_pubkey()
                .unwrap();

        let mut tx = LendingTransaction::new();

        // Simulate upstream setup outputs 0–2 before attach_creation.
        for index in 0..3u8 {
            let asset = sample_asset(0x10 + index);
            tx.add_explicit_output(&Script::new_op_return(vec![index]), 1, asset)
                .unwrap();
        }

        offer.attach_creation(&mut tx).unwrap();
        assert_eq!(tx.n_outputs(), 6);

        let pset = tx.build().unwrap();
        let extracted = Transaction::deserialize(&pset.extract_tx_bytes().unwrap()).unwrap();

        let lender_nft_output = &extracted.output[3];
        let metadata_output = &extracted.output[4];
        let pending_offer_output = &extracted.output[5];

        assert_eq!(
            lender_nft_output.script_pubkey.to_hex(),
            lender_nft_script.to_hex()
        );
        assert!(metadata_output.script_pubkey.is_op_return());
        assert_eq!(
            pending_offer_output.script_pubkey.to_hex(),
            offer.script_pubkey().unwrap().to_hex()
        );
        assert_eq!(
            pending_offer_output.value.explicit().unwrap(),
            parameters.offer_parameters.collateral_amount
        );
        assert_eq!(
            pending_offer_output.asset.explicit().unwrap().to_string(),
            parameters.collateral_asset_id
        );

        let metadata_bytes = op_return_payload(&metadata_output.script_pubkey.to_hex());
        assert_eq!(metadata_bytes.len(), 50);
        assert_eq!(&metadata_bytes[..4], LendingOffer::program_id().as_slice());

        let decoded = LendingOffer::decode_creation_metadata(&metadata_bytes).unwrap();
        assert_eq!(decoded.program_id, LendingOffer::program_id());
        assert_eq!(
            hex::encode(decoded.principal_asset_id),
            parameters.principal_asset_id
        );
        assert_eq!(decoded.principal_amount, parameters.offer_parameters.principal_amount);
        assert_eq!(
            decoded.loan_expiration_time,
            parameters.offer_parameters.loan_expiration_time
        );
        assert_eq!(
            decoded.principal_interest_rate,
            parameters.offer_parameters.principal_interest_rate
        );
    }
}
