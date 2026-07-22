//! Shared lending math helpers ported from upstream simplicity-lending.

pub const MAX_BASIS_POINTS: u64 = 10_000;

pub fn apply_basis_points(amount: u64, bps: u16) -> u64 {
    let amount_wide = u128::from(amount) * u128::from(bps);
    u64::try_from(amount_wide / u128::from(MAX_BASIS_POINTS)).expect("basis points overflow")
}
