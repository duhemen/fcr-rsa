//! FCR Core - Gaussian Integer Mapping (v0.6.0 Simplified)

use crypto_bigint::U2048;
use crypto_bigint::modular::runtime_mod::{DynResidue, DynResidueParams};

pub fn map_gaussian(value: &U2048, q: &U2048) -> U2048 {
    // SOLUSI FINAL v0.5.5: Gunakan DynResidue
    let params = Option::from(DynResidueParams::new(q)).expect("Modulus must be odd");
    let val_res = DynResidue::new(value, params);
    val_res.square().retrieve()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fib::Q_2048;

    #[test]
    fn test_gaussian_map_basic() {
        let val = U2048::from(5u32);
        let result = map_gaussian(&val, &Q_2048);
        assert_eq!(result, U2048::from(25u32));
    }
}