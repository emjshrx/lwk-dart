use lwk_wollet::elements::Txid;

use crate::api::error::LwkError;

#[derive(thiserror::Error, Debug)]
pub(crate) enum IssuanceFactoryError {
    #[error("Invalid creation OP_RETURN data length: expected - {expected}, actual - {actual}")]
    InvalidCreationMetadataLength { expected: usize, actual: usize },

    #[error("Invalid OP_RETURN metadata bytes: {0}")]
    InvalidMetadataBytes(String),

    #[error("Passed transaction is not an issuance factory creation transaction")]
    NotAnIssuanceFactoryCreationTx(Txid),

    #[error("Assets issuance can't be first attachment in transaction")]
    AssetsIssuanceRequiresPriorInput,

    #[error("{0}")]
    Internal(String),
}

impl From<LwkError> for IssuanceFactoryError {
    fn from(value: LwkError) -> Self {
        Self::Internal(value.msg)
    }
}

impl From<IssuanceFactoryError> for LwkError {
    fn from(value: IssuanceFactoryError) -> Self {
        LwkError {
            msg: value.to_string(),
        }
    }
}
