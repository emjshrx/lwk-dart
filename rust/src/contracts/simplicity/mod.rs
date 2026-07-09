pub mod arguments;
pub mod cmr;
pub mod log_level;
pub mod program;
pub mod run_result;
pub mod simplicity_type;
pub mod state_taproot;
pub mod typed_value;
pub mod utils;

pub use arguments::{SimplicityArguments, SimplicityWitnessValues};
pub use cmr::Cmr;
pub use log_level::SimplicityLogLevel;
pub use program::SimplicityProgram;
pub use run_result::SimplicityRunResult;
pub use simplicity_type::SimplicityType;
pub use state_taproot::{StateTaprootBuilder, StateTaprootSpendInfo, UNSPENDABLE_TAPROOT_PUBKEY};
pub use typed_value::SimplicityTypedValue;
pub use utils::{simplicity_control_block, simplicity_derive_xonly_pubkey};

#[cfg(test)]
mod integration_test;

/// Generate an ephemeral x-only public key for use as a Taproot internal key.
pub fn generate_simplicity_ephemeral_pubkey() -> anyhow::Result<String, crate::api::error::LwkError> {
    use lwk_wollet::elements::bitcoin::secp256k1::{self, Secp256k1};

    let secp = Secp256k1::new();
    let (_, keypair) = secp.generate_keypair(&mut secp256k1::rand::thread_rng());
    let (xonly, _) = keypair.x_only_public_key();
    Ok(hex::encode(xonly.serialize()))
}
