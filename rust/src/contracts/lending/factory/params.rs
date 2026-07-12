use crate::api::types::LiquidNetwork;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct IssuanceFactoryParameters {
    pub issuing_utxos_count: u8,
    pub reissuance_flags: u64,
    pub network: LiquidNetwork,
}
