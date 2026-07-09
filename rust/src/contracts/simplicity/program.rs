use lwk_simplicity::runner;
use lwk_simplicity::scripts;
use lwk_simplicity::signer;
use lwk_simplicity::simplicityhl::{CompiledProgram, WitnessValues};
use lwk_wollet::elements::{
    self,
    pset::serialize::Deserialize,
    AddressParams, Transaction,
};

use crate::contracts::blockdata::XOnlyPublicKey;
use crate::api::error::LwkError;
use crate::contracts::signer::Signer;
use crate::contracts::simplicity::{
    Cmr, SimplicityArguments, SimplicityLogLevel, SimplicityRunResult, SimplicityWitnessValues,
};
use crate::api::types::LiquidNetwork;

use super::utils::{parse_utxos_hex, parse_xonly_pubkey_hex, to_lwk_common_network};

/// A compiled Simplicity program ready for covenant transactions.
pub struct SimplicityProgram {
    inner: CompiledProgram,
}

impl SimplicityProgram {
    /// Compile a SimplicityHL program from source and typed arguments.
    pub fn load_with_arguments(
        source: String,
        arguments: &SimplicityArguments,
    ) -> anyhow::Result<SimplicityProgram, LwkError> {
        let compiled = scripts::load_program(&source, arguments.to_inner()?)?;
        Ok(SimplicityProgram { inner: compiled })
    }

    pub fn cmr(&self) -> Cmr {
        self.inner.commit().cmr().into()
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
        let internal_key = parse_xonly_pubkey_hex(&internal_key_hex)?.to_simplicityhl()?;
        let cmr = self.inner.commit().cmr();
        let params: &'static AddressParams = match network {
            LiquidNetwork::Mainnet => &AddressParams::LIQUID,
            LiquidNetwork::Testnet => &AddressParams::LIQUID_TESTNET,
        };
        let address = scripts::create_p2tr_address(cmr, &internal_key, params);
        Ok(address.to_string())
    }

    pub fn get_sighash_all(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        network: LiquidNetwork,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = parse_xonly_pubkey_hex(&program_public_key_hex)?.to_simplicityhl()?;
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

    pub fn finalize_transaction_with_values(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        witness_values: &SimplicityWitnessValues,
        network: LiquidNetwork,
        log_level: SimplicityLogLevel,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        self.finalize_transaction_with_witness(
            tx_bytes,
            program_public_key_hex,
            utxos_hex,
            input_index,
            witness_values.to_inner()?,
            network,
            log_level,
        )
    }

    fn finalize_transaction_with_witness(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        witness_values: WitnessValues,
        network: LiquidNetwork,
        log_level: SimplicityLogLevel,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = parse_xonly_pubkey_hex(&program_public_key_hex)?.to_simplicityhl()?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let finalized_tx = signer::finalize_transaction(
            tx,
            &self.inner,
            &public_key,
            &utxos,
            input_index as usize,
            witness_values,
            to_lwk_common_network(network),
            log_level.into(),
        )?;
        Ok(elements::encode::serialize(&finalized_tx))
    }

    pub fn control_block(
        &self,
        internal_key_hex: String,
    ) -> anyhow::Result<String, LwkError> {
        let internal_key = parse_xonly_pubkey_hex(&internal_key_hex)?.to_simplicityhl()?;
        let cmr = self.inner.commit().cmr();
        let cb = scripts::control_block(cmr, internal_key);
        Ok(hex::encode(cb.serialize()))
    }

    pub fn control_block_for_key(
        &self,
        internal_key: &XOnlyPublicKey,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let internal = internal_key.to_simplicityhl()?;
        let cmr = self.inner.commit().cmr();
        let cb = scripts::control_block(cmr, internal);
        Ok(cb.serialize())
    }

    pub fn simplicity_leaf_version() -> u8 {
        scripts::simplicity_leaf_version().as_u8()
    }

    /// Execute the program in a transaction environment.
    pub fn run(
        &self,
        tx_bytes: Vec<u8>,
        program_public_key_hex: String,
        utxos_hex: Vec<String>,
        input_index: u32,
        witness_values: &SimplicityWitnessValues,
        network: LiquidNetwork,
        log_level: SimplicityLogLevel,
    ) -> anyhow::Result<SimplicityRunResult, LwkError> {
        let tx = Transaction::deserialize(&tx_bytes)?;
        let public_key = parse_xonly_pubkey_hex(&program_public_key_hex)?.to_simplicityhl()?;
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
            witness_values.to_inner()?,
            &env,
            log_level.into(),
        )?;
        Ok(SimplicityRunResult { pruned, value })
    }

    /// Create a Schnorr signature for a P2PK Simplicity input.
    pub fn create_p2pk_signature(
        &self,
        signer: &Signer,
        derivation_path: String,
        tx_bytes: Vec<u8>,
        utxos_hex: Vec<String>,
        input_index: u32,
        network: LiquidNetwork,
    ) -> anyhow::Result<Vec<u8>, LwkError> {
        let keypair = super::utils::derive_keypair_from_signer(signer, &derivation_path)?;
        let x_only_pubkey = keypair.x_only_public_key().0;
        let tx = Transaction::deserialize(&tx_bytes)?;
        let utxos = parse_utxos_hex(&utxos_hex)?;
        let sighash = signer::get_sighash_all(
            &tx,
            &self.inner,
            &x_only_pubkey,
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
    use crate::contracts::simplicity::SimplicityTypedValue;

    const P2PK_SOURCE: &str = concat!(
        "fn main() {\n",
        "    jet::bip_0340_verify(",
        "(param::ALICE_PUBLIC_KEY, jet::sig_all_hash()), ",
        "witness::ALICE_SIGNATURE)\n",
        "}\n",
    );

    fn p2pk_args(pubkey_hex: &str) -> SimplicityArguments {
        SimplicityArguments::new().add_value(
            "ALICE_PUBLIC_KEY".into(),
            SimplicityTypedValue::u256(hex::decode(pubkey_hex).unwrap()).unwrap(),
        )
    }

    #[test]
    fn test_load_p2pk_program() {
        let test_pubkey_hex = "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";
        let program = SimplicityProgram::load_with_arguments(
            P2PK_SOURCE.to_string(),
            &p2pk_args(test_pubkey_hex),
        )
        .unwrap();
        assert!(!program.cmr_hex().is_empty());
    }

    #[test]
    fn test_p2pk_address_generation() {
        let test_pubkey_hex = "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";
        let program = SimplicityProgram::load_with_arguments(
            P2PK_SOURCE.to_string(),
            &p2pk_args(test_pubkey_hex),
        )
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
