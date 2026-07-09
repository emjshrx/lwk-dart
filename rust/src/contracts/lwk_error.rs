//! Error conversions used by internal contract code only.
use crate::api::error::LwkError;

impl From<lwk_simplicity::error::ProgramError> for LwkError {
    fn from(value: lwk_simplicity::error::ProgramError) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<lwk_wollet::elements::taproot::TaprootBuilderError> for LwkError {
    fn from(value: lwk_wollet::elements::taproot::TaprootBuilderError) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<lwk_wollet::elements::bitcoin::taproot::TaprootBuilderError> for LwkError {
    fn from(value: lwk_wollet::elements::bitcoin::taproot::TaprootBuilderError) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<lwk_wollet::elements::bitcoin::bip32::Error> for LwkError {
    fn from(value: lwk_wollet::elements::bitcoin::bip32::Error) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<lwk_wollet::elements::bitcoin::secp256k1::Error> for LwkError {
    fn from(value: lwk_wollet::elements::bitcoin::secp256k1::Error) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<lwk_wollet::elements_miniscript::psbt::Error> for LwkError {
    fn from(value: lwk_wollet::elements_miniscript::psbt::Error) -> Self {
        LwkError {
            msg: format!("{value:?}"),
        }
    }
}

impl From<std::array::TryFromSliceError> for LwkError {
    fn from(value: std::array::TryFromSliceError) -> Self {
        LwkError {
            msg: value.to_string(),
        }
    }
}
