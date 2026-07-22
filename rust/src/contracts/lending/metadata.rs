//! Lending offer creation metadata encoding.

use crate::api::error::LwkError;

use super::covenant_program::{ProgramId, PROGRAM_ID_LENGTH};
use super::params::OfferParameters;

const LENDING_OFFER_CREATION_METADATA_LENGTH: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LendingOfferCreationMetadata {
    pub program_id: ProgramId,
    pub principal_asset_id: [u8; 32],
    pub principal_amount: u64,
    pub loan_expiration_time: u32,
    pub principal_interest_rate: u16,
}

impl LendingOfferCreationMetadata {
    pub fn new(
        program_id: ProgramId,
        principal_asset_id: [u8; 32],
        offer_parameters: OfferParameters,
    ) -> Self {
        Self {
            program_id,
            principal_asset_id,
            principal_amount: offer_parameters.principal_amount,
            loan_expiration_time: offer_parameters.loan_expiration_time,
            principal_interest_rate: offer_parameters.principal_interest_rate,
        }
    }

    pub fn decode(op_return_bytes: &[u8]) -> anyhow::Result<Self, LwkError> {
        if op_return_bytes.len() != LENDING_OFFER_CREATION_METADATA_LENGTH {
            return Err(LwkError {
                msg: format!(
                    "invalid creation metadata length: expected {}, got {}",
                    LENDING_OFFER_CREATION_METADATA_LENGTH,
                    op_return_bytes.len()
                ),
            });
        }

        let mut cursor = 0;
        let mut program_id = [0u8; PROGRAM_ID_LENGTH];
        program_id.copy_from_slice(&op_return_bytes[cursor..cursor + PROGRAM_ID_LENGTH]);
        cursor += PROGRAM_ID_LENGTH;

        let mut principal_asset_id = [0u8; 32];
        principal_asset_id.copy_from_slice(&op_return_bytes[cursor..cursor + 32]);
        cursor += 32;

        let principal_amount = u64::from_le_bytes(
            op_return_bytes[cursor..cursor + 8]
                .try_into()
                .expect("u64 length"),
        );
        cursor += 8;

        let loan_expiration_time = u32::from_le_bytes(
            op_return_bytes[cursor..cursor + 4]
                .try_into()
                .expect("u32 length"),
        );
        cursor += 4;

        let principal_interest_rate = u16::from_le_bytes(
            op_return_bytes[cursor..cursor + 2]
                .try_into()
                .expect("u16 length"),
        );

        Ok(Self {
            program_id,
            principal_asset_id,
            principal_amount,
            loan_expiration_time,
            principal_interest_rate,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(LENDING_OFFER_CREATION_METADATA_LENGTH);
        data.extend_from_slice(&self.program_id);
        data.extend_from_slice(&self.principal_asset_id);
        data.extend_from_slice(&self.principal_amount.to_le_bytes());
        data.extend_from_slice(&self.loan_expiration_time.to_le_bytes());
        data.extend_from_slice(&self.principal_interest_rate.to_le_bytes());
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_metadata_encoding() {
        let metadata = LendingOfferCreationMetadata::new(
            [1, 2, 3, 4],
            [7u8; 32],
            OfferParameters {
                collateral_amount: 3000,
                principal_amount: 10_000,
                loan_expiration_time: 100_060,
                principal_interest_rate: 1000,
            },
        );

        let encoded = metadata.encode();
        assert_eq!(encoded.len(), 50);
        let decoded = LendingOfferCreationMetadata::decode(&encoded).unwrap();
        assert_eq!(decoded, metadata);
    }
}
