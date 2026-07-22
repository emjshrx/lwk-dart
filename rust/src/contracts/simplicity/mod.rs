pub mod program;
pub mod run_result;
pub mod state_taproot;
pub mod utils;

// Re-exports form the intentional internal API surface (may be unused until callers land).
#[allow(unused_imports)]
pub use program::SimplicityProgram;
#[allow(unused_imports)]
pub use run_result::SimplicityRunResult;
#[allow(unused_imports)]
pub use state_taproot::{StateTaprootBuilder, StateTaprootSpendInfo, UNSPENDABLE_TAPROOT_PUBKEY};
#[allow(unused_imports)]
pub use utils::{derive_keypair, simplicity_control_block, xonly_to_simplicityhl};

#[cfg(test)]
mod integration_test;
