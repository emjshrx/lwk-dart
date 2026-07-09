use crate::contracts::blockdata::{ControlBlock, XOnlyPublicKey};
use crate::api::error::LwkError;
use crate::contracts::signer::Signer;
use crate::contracts::simplicity::Cmr;
use lwk_simplicity::scripts;

/// Derive an x-only public key from a signer and BIP32 path.
pub fn simplicity_derive_xonly_pubkey(
    signer: &Signer,
    derivation_path: String,
) -> anyhow::Result<XOnlyPublicKey, LwkError> {
    let keypair = signer.derive_keypair(&derivation_path)?;
    let (xonly, _) = keypair.x_only_public_key();
    Ok(XOnlyPublicKey::from(xonly))
}

/// Compute the Taproot control block for a Simplicity program CMR and internal key.
pub fn simplicity_control_block(
    cmr: &Cmr,
    internal_key: &XOnlyPublicKey,
) -> anyhow::Result<ControlBlock, LwkError> {
    let internal = internal_key.to_simplicityhl()?;
    let control_block = scripts::control_block(cmr.inner(), internal);
    Ok(ControlBlock::from(control_block))
}

pub(crate) fn parse_xonly_pubkey_hex(hex_str: &str) -> Result<XOnlyPublicKey, LwkError> {
    XOnlyPublicKey::from_string(hex_str.to_string())
}

pub(crate) fn parse_utxos_hex(utxos_hex: &[String]) -> Result<Vec<lwk_wollet::elements::TxOut>, LwkError> {
    use lwk_wollet::elements::pset::serialize::Deserialize;

    utxos_hex
        .iter()
        .enumerate()
        .map(|(i, hex_str)| {
            let bytes = hex::decode(hex_str)
                .map_err(|e| LwkError { msg: format!("Invalid hex for UTXO {i}: {e}") })?;
            lwk_wollet::elements::TxOut::deserialize(&bytes)
                .map_err(|e| LwkError { msg: format!("Failed to deserialize UTXO {i}: {e}") })
        })
        .collect()
}

pub(crate) fn to_lwk_common_network(network: crate::api::types::LiquidNetwork) -> lwk_common::Network {
    match network {
        crate::api::types::LiquidNetwork::Mainnet => lwk_common::Network::Liquid,
        crate::api::types::LiquidNetwork::Testnet => lwk_common::Network::TestnetLiquid,
    }
}

pub(crate) fn derive_keypair_from_signer(
    signer: &Signer,
    derivation_path: &str,
) -> Result<lwk_wollet::secp256k1::Keypair, LwkError> {
    signer.derive_keypair(derivation_path)
}
