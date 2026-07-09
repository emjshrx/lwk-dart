
use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut, ElementsTxOutSecrets};

/// An external UTXO owned outside the wallet (e.g. a covenant output).
#[derive(Clone)]
pub struct ExternalUtxo {
    pub(crate) inner: lwk_wollet::ExternalUtxo,
}

impl ExternalUtxo {
    pub fn from_unchecked_data(
        outpoint: &ElementsOutPoint,
        txout: &ElementsTxOut,
        unblinded: &ElementsTxOutSecrets,
        max_weight_to_satisfy: u32,
    ) -> ExternalUtxo {
        ExternalUtxo {
            inner: lwk_wollet::ExternalUtxo {
                outpoint: outpoint.inner,
                txout: txout.inner.clone(),
                tx: None,
                unblinded: unblinded.inner,
                max_weight_to_satisfy: max_weight_to_satisfy as usize,
            },
        }
    }
}

impl From<&ExternalUtxo> for lwk_wollet::ExternalUtxo {
    fn from(value: &ExternalUtxo) -> Self {
        value.inner.clone()
    }
}
