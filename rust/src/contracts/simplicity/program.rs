use lwk_simplicity::runner;
use lwk_simplicity::scripts;
use lwk_simplicity::signer;
use lwk_simplicity::simplicityhl::simplicity::Cmr;
use lwk_simplicity::simplicityhl::tracker::TrackerLogLevel;
use lwk_simplicity::simplicityhl::{Arguments, CompiledProgram, WitnessValues};
use lwk_wollet::elements::{
    self,
    pset::serialize::Deserialize,
    AddressParams, Transaction,
};
use lwk_wollet::secp256k1::Keypair;

use crate::api::error::LwkError;
use crate::api::types::LiquidNetwork;
use crate::contracts::simplicity::SimplicityRunResult;

use super::utils::{parse_utxos_hex, parse_xonly_pubkey_hex, to_lwk_common_network, xonly_to_simplicityhl};

/// A compiled Simplicity program ready for covenant transactions.
#[allow(dead_code)] // Internal API surface; not all methods are exercised by current tests.
pub struct SimplicityProgram {
    inner: CompiledProgram,
}

#[allow(dead_code)] // Internal API surface; not all methods are exercised by current tests.
impl SimplicityProgram {
    /// Compile a SimplicityHL program from source and typed arguments.
    pub fn load_with_arguments(
        source: String,
        arguments: &Arguments,
    ) -> anyhow::Result<SimplicityProgram, LwkError> {
        let compiled = scripts::load_program(&source, arguments.clone())?;
        Ok(SimplicityProgram { inner: compiled })
    }

    pub fn cmr(&self) -> Cmr {
        self.inner.commit().cmr()
    }

    /// Hex-encoded CMR.
    pub fn cmr_hex(&self) -> String {
        self.inner.commit().cmr().to_string()
    }

    pub fn create_p2tr_address(
        &self,
        internal_key_hex: String,
        network: LiquidNetwork,
    ) -> anyhow::Result<String, LwkError> {
        let internal_key = xonly_to_simplicityhl(&parse_xonly_pubkey_hex(&internal_key_hex)?)?;
        let cmr = self.inner.commit().cmr();
        let params: &'static AddressParams = match network {
            LiquidNetwork::Mainnet => &AddressParams::LIQUID,
            LiquidNetwork::Testnet => &AddressParams::LIQUID_TESTNET,
        };
        let address = scripts::create_p2tr_address(cmr, &internal_key, params);
        Ok(address.to_string())
    }

    /// Compute the BIP-340 sighash message for signing a Simplicity input.
    ///
    /// Used by [`Self::create_p2pk_signature`] and by callers that sign externally.
    pub fn get_sighash_all(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        network: LiquidNetwork,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = xonly_to_simplicityhl(&parse_xonly_pubkey_hex(&program_public_key_hex)?)?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let message = signer::get_sighash_all(
            &tx,
            &self.inner,
            &public_key,
            &utxos,
            input_index as usize,
            to_lwk_common_network(network),
        )?;
        Ok(message.as_ref().to_vec())
    }

    /// Attach a Simplicity witness stack to a transaction input and return the
    /// serialized finalized transaction.
    ///
    /// This is not the same as [`Self::run`]: finalize mutates the tx witness;
    /// `run` only executes the program offline for verification.
    pub fn finalize_transaction(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        witness_values: &WitnessValues,
        network: LiquidNetwork,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = xonly_to_simplicityhl(&parse_xonly_pubkey_hex(&program_public_key_hex)?)?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let finalized_tx = signer::finalize_transaction(
            tx,
            &self.inner,
            &public_key,
            &utxos,
            input_index as usize,
            witness_values.clone(),
            to_lwk_common_network(network),
            TrackerLogLevel::Warning,
        )?;
        Ok(elements::encode::serialize(&finalized_tx))
    }

    pub fn control_block(&self, internal_key_hex: String) -> anyhow::Result<String, LwkError> {
        let internal_key = xonly_to_simplicityhl(&parse_xonly_pubkey_hex(&internal_key_hex)?)?;
        let cmr = self.inner.commit().cmr();
        let cb = scripts::control_block(cmr, internal_key);
        Ok(hex::encode(cb.serialize()))
    }

    pub fn simplicity_leaf_version() -> u8 {
        scripts::simplicity_leaf_version().as_u8()
    }

    /// Execute the program offline in a transaction environment (verify / debug).
    ///
    /// Does **not** attach a witness to the transaction — use [`Self::finalize_transaction`]
    /// for that. Returns the redeem-node encoding used to build the Simplicity
    /// witness stack.
    pub fn run(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        witness_values: &WitnessValues,
        network: LiquidNetwork,
    ) -> anyhow::Result<SimplicityRunResult, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = xonly_to_simplicityhl(&parse_xonly_pubkey_hex(&program_public_key_hex)?)?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let env = signer::get_and_verify_env(
            &tx,
            &self.inner,
            &public_key,
            &utxos,
            to_lwk_common_network(network),
            input_index as usize,
        )?;
        let (pruned, value) = runner::run_program(
            &self.inner,
            witness_values.clone(),
            &env,
            TrackerLogLevel::Warning,
        )?;
        Ok(SimplicityRunResult { pruned, value })
    }

    /// Create a Schnorr signature for a P2PK Simplicity input.
    pub fn create_p2pk_signature(
        &self,
        keypair: &Keypair,
        tx_bytes: Vec<u8>,
        utxos_hex: Vec<String>,
        input_index: u32,
        network: LiquidNetwork,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let x_only_pubkey = keypair.x_only_public_key().0;
        let tx = Transaction::deserialize(&tx_bytes)?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let public_key = xonly_to_simplicityhl(&x_only_pubkey)?;
        let sighash = signer::get_sighash_all(
            &tx,
            &self.inner,
            &public_key,
            &utxos,
            input_index as usize,
            to_lwk_common_network(network),
        )?;
        let signature = keypair.sign_schnorr(sighash);
        Ok(signature.serialize().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwk_simplicity::simplicityhl::num::U256;
    use lwk_simplicity::simplicityhl::parse::ParseFromStr;
    use lwk_simplicity::simplicityhl::str::WitnessName;
    use lwk_simplicity::simplicityhl::value::ValueConstructible;
    use lwk_simplicity::simplicityhl::Value;
    use std::collections::HashMap;

    const P2PK_SOURCE: &str = concat!(
        "fn main() {\n",
        "    jet::bip_0340_verify(",
        "(param::ALICE_PUBLIC_KEY, jet::sig_all_hash()), ",
        "witness::ALICE_SIGNATURE)\n",
        "}\n",
    );

    fn p2pk_args(pubkey_hex: &str) -> Arguments {
        let mut map = HashMap::new();
        map.insert(
            WitnessName::parse_from_str("ALICE_PUBLIC_KEY").unwrap(),
            Value::u256(U256::from_byte_array(
                hex::decode(pubkey_hex).unwrap().try_into().unwrap(),
            )),
        );
        Arguments::from(map)
    }

    #[test]
    fn test_load_p2pk_program() {
        let test_pubkey_hex = "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";
        let program =
            SimplicityProgram::load_with_arguments(P2PK_SOURCE.to_string(), &p2pk_args(test_pubkey_hex))
                .unwrap();
        assert!(!program.cmr_hex().is_empty());
    }

    #[test]
    fn test_p2pk_address_generation() {
        let test_pubkey_hex = "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";
        let program =
            SimplicityProgram::load_with_arguments(P2PK_SOURCE.to_string(), &p2pk_args(test_pubkey_hex))
                .unwrap();
        let address = program
            .create_p2tr_address(test_pubkey_hex.to_string(), LiquidNetwork::Testnet)
            .unwrap();
        assert!(!address.is_empty());
        assert!(address.starts_with("tex1") || address.starts_with("ert1"));
    }

    #[test]
    fn test_simplicity_leaf_version() {
        assert!(SimplicityProgram::simplicity_leaf_version() > 0);
    }
}
