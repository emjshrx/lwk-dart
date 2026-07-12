use crate::contracts::lending::factory::{IssuanceFactory, IssuanceFactoryError, IssuanceFactoryParameters};
use crate::contracts::lending::program::{CreationMetadata, ProgramId, PROGRAM_ID_LENGTH};

const CREATION_OP_RETURN_DATA_LENGTH: usize =
    PROGRAM_ID_LENGTH + std::mem::size_of::<u8>() + std::mem::size_of::<u64>();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IssuanceFactoryCreationMetadata {
    pub program_id: Vec<u8>,
    pub issuing_utxos_count: u8,
    pub reissuance_flags: u64,
}

impl IssuanceFactoryCreationMetadata {
    pub(crate) fn new(program_id: ProgramId, issuing_utxos_count: u8, reissuance_flags: u64) -> Self {
        Self {
            program_id: program_id.to_vec(),
            issuing_utxos_count,
            reissuance_flags,
        }
    }
}

impl CreationMetadata for IssuanceFactoryCreationMetadata {
    type Error = IssuanceFactoryError;

    const DATA_LENGTH: usize = CREATION_OP_RETURN_DATA_LENGTH;

    fn decode(op_return_bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::validate_length(op_return_bytes, |expected, actual| {
            IssuanceFactoryError::InvalidCreationMetadataLength { expected, actual }
        })?;

        let mut cursor = 0;
        let _program_id = Self::decode_program_id(op_return_bytes);
        cursor += PROGRAM_ID_LENGTH;

        let issuing_utxos_count = op_return_bytes[cursor];
        cursor += std::mem::size_of::<u8>();

        let reissuance_flags = u64::from_le_bytes(
            op_return_bytes[cursor..cursor + std::mem::size_of::<u64>()]
                .try_into()
                .expect("reissuance flags length is fixed"),
        );

        Ok(Self {
            program_id: Self::decode_program_id(op_return_bytes).to_vec(),
            issuing_utxos_count,
            reissuance_flags,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut op_return_data = Vec::with_capacity(Self::DATA_LENGTH);
        op_return_data.extend_from_slice(&self.program_id);
        op_return_data.push(self.issuing_utxos_count);
        op_return_data.extend_from_slice(&self.reissuance_flags.to_le_bytes());
        op_return_data
    }
}

impl IssuanceFactory {
    pub(crate) fn build_metadata(&self) -> IssuanceFactoryCreationMetadata {
        let parameters = self.get_parameters();
        IssuanceFactoryCreationMetadata::new(
            Self::program_id(),
            parameters.issuing_utxos_count,
            parameters.reissuance_flags,
        )
    }

    pub(crate) fn encode_metadata_op_return(&self) -> Vec<u8> {
        self.build_metadata().encode()
    }

    pub(crate) fn decode_metadata_op_return(
        op_return_bytes: Vec<u8>,
    ) -> Result<IssuanceFactoryCreationMetadata, IssuanceFactoryError> {
        IssuanceFactoryCreationMetadata::decode(&op_return_bytes)
    }
}
