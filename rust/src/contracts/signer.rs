use lwk_signer::SwSigner;
use lwk_wollet::elements::bitcoin::bip32::DerivationPath;
use lwk_wollet::secp256k1::Keypair;
use lwk_wollet::EC;
use std::str::FromStr;
use std::sync::Mutex;

use crate::api::error::LwkError;
use crate::api::types::LiquidNetwork;

/// Software signer for Simplicity covenant signing.
pub struct Signer {
    pub(crate) inner: Mutex<SwSigner>,
}

impl Signer {
    pub fn new(mnemonic: String, network: LiquidNetwork) -> anyhow::Result<Signer, LwkError> {
        let is_mainnet = matches!(network, LiquidNetwork::Mainnet);
        let inner = SwSigner::new(&mnemonic, is_mainnet)?;
        Ok(Signer {
            inner: Mutex::new(inner),
        })
    }

    pub(crate) fn derive_keypair(&self, derivation_path: &str) -> Result<Keypair, LwkError> {
        let signer = self.inner.lock()?;
        let derived = signer.derive_xprv(&DerivationPath::from_str(derivation_path)?)?;
        Ok(Keypair::from_secret_key(&EC, &derived.private_key))
    }
}
