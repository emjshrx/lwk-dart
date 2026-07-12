//! Shared lending program helpers ported from upstream `programs/program.rs`.

use lwk_wollet::elements::script::Instruction;
use lwk_wollet::elements::{self, opcodes, Script};
use lwk_wollet::hashes::{sha256, Hash};

use crate::api::error::LwkError;
use crate::api::types::LiquidNetwork;
use crate::contracts::blockdata::{Script as LwkScript, XOnlyPublicKey};
use crate::contracts::simplicity::{
    SimplicityProgram, StateTaprootBuilder, UNSPENDABLE_TAPROOT_PUBKEY,
};

pub const PROGRAM_ID_LENGTH: usize = 4;
pub type ProgramId = [u8; PROGRAM_ID_LENGTH];

pub trait CreationMetadata: Sized {
    type Error;

    const DATA_LENGTH: usize;

    fn decode(op_return_bytes: &[u8]) -> Result<Self, Self::Error>;

    fn encode(&self) -> Vec<u8>;

    fn validate_length(
        op_return_bytes: &[u8],
        invalid_length: impl FnOnce(usize, usize) -> Self::Error,
    ) -> Result<(), Self::Error> {
        if op_return_bytes.len() != Self::DATA_LENGTH {
            return Err(invalid_length(Self::DATA_LENGTH, op_return_bytes.len()));
        }

        Ok(())
    }

    fn decode_program_id(op_return_bytes: &[u8]) -> ProgramId {
        let mut program_id = [0; PROGRAM_ID_LENGTH];
        program_id.copy_from_slice(&op_return_bytes[..PROGRAM_ID_LENGTH]);
        program_id
    }
}

pub fn program_id_from_source(source: &str) -> ProgramId {
    let hash = sha256::Hash::hash(source.as_bytes());
    let mut program_id = [0; PROGRAM_ID_LENGTH];
    program_id.copy_from_slice(&hash.to_byte_array()[..PROGRAM_ID_LENGTH]);
    program_id
}

pub fn op_return_payload(script: &elements::Script) -> Option<Vec<u8>> {
    if !script.is_op_return() {
        return None;
    }

    let mut data = Vec::new();
    let mut seen_op_return = false;
    for instruction in script.instructions() {
        match instruction {
            Ok(Instruction::Op(op)) if op == opcodes::all::OP_RETURN => {
                seen_op_return = true;
            }
            Ok(Instruction::PushBytes(bytes)) if seen_op_return => {
                data.extend_from_slice(bytes);
            }
            Ok(Instruction::PushBytes(_)) => {}
            _ => return None,
        }
    }

    seen_op_return.then_some(data)
}

pub fn program_script_pubkey(
    program: &SimplicityProgram,
    _network: LiquidNetwork,
) -> anyhow::Result<LwkScript, LwkError> {
    let internal_key =
        XOnlyPublicKey::from_string(UNSPENDABLE_TAPROOT_PUBKEY.into())?;
    let spend = StateTaprootBuilder::new()
        .add_simplicity_leaf(0, &program.cmr())?
        .finalize(&internal_key)?;
    Ok(spend.script_pubkey())
}

pub fn burn_op_return_script() -> LwkScript {
    LwkScript::new_op_return(b"burn".to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_return_payload_roundtrip() {
        let script = Script::new_op_return(&[1, 2, 3, 4]);
        assert_eq!(op_return_payload(&script), Some(vec![1, 2, 3, 4]));
    }
}
