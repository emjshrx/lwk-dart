use lwk_signer::SwSigner;
use lwk_simplicity::scripts;
use lwk_simplicity::simplicityhl::simplicity::bitcoin::XOnlyPublicKey as SimplicityXOnlyPublicKey;
use lwk_wollet::elements::bitcoin::bip32::DerivationPath;
use lwk_wollet::elements::bitcoin::XOnlyPublicKey;
use lwk_wollet::elements::taproot::ControlBlock;
use lwk_wollet::secp256k1::Keypair;
use lwk_wollet::EC;
use std::str::FromStr;

use crate::api::error::LwkError;
use crate::api::types::LiquidNetwork;
use lwk_simplicity::simplicityhl::simplicity::Cmr;

/// Convert an Elements x-only key to the simplicityhl key type.
pub fn xonly_to_simplicityhl(
    key: &XOnlyPublicKey,
) -> anyhow::Result<SimplicityXOnlyPublicKey, LwkError> {
    Ok(SimplicityXOnlyPublicKey::from_slice(&key.serialize())?)
}

/// Derive a secp256k1 keypair from a mnemonic and BIP32 path.
pub fn derive_keypair(
    mnemonic: &str,
    network: LiquidNetwork,
    derivation_path: &str,
) -> anyhow::Result<Keypair, LwkError> {
    let is_mainnet = matches!(network, LiquidNetwork::Mainnet);
    let signer = SwSigner::new(mnemonic, is_mainnet)?;
    let derived = signer.derive_xprv(&DerivationPath::from_str(derivation_path)?)?;
    Ok(Keypair::from_secret_key(&EC, &derived.private_key))
}

/// Compute the Taproot control block for a Simplicity program CMR and internal key.
pub fn simplicity_control_block(
    cmr: Cmr,
    internal_key: &XOnlyPublicKey,
) -> anyhow::Result<ControlBlock, LwkError> {
    let internal = xonly_to_simplicityhl(internal_key)?;
    Ok(scripts::control_block(cmr, internal))
}

pub(crate) fn parse_xonly_pubkey_hex(hex_str: &str) -> Result<XOnlyPublicKey, LwkError> {
    XOnlyPublicKey::from_str(hex_str).map_err(|e| LwkError { msg: e.to_string() })
}

pub(crate) fn parse_utxos_hex(
    utxos_hex: &[String],
) -> Result<Vec<lwk_wollet::elements::TxOut>, LwkError> {
    use lwk_wollet::elements::pset::serialize::Deserialize;

    utxos_hex
        .iter()
        .enumerate()
        .map(|(i, hex_str)| {
            let bytes = hex::decode(hex_str).map_err(|e| LwkError {
                msg: format!("Invalid hex for UTXO {i}: {e}"),
            })?;
            lwk_wollet::elements::TxOut::deserialize(&bytes).map_err(|e| LwkError {
                msg: format!("Failed to deserialize UTXO {i}: {e}"),
            })
        })
        .collect()
}

pub(crate) fn to_lwk_common_network(network: LiquidNetwork) -> lwk_common::Network {
    match network {
        LiquidNetwork::Mainnet => lwk_common::Network::Liquid,
        LiquidNetwork::Testnet => lwk_common::Network::TestnetLiquid,
    }
}
