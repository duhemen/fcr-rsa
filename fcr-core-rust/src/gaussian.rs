//! FCR Core - Gaussian Integer Mapping (v0.6.1-hardened)

use crypto_bigint::modular::{FixedMontyForm, FixedMontyParams};
use crypto_bigint::{Odd, U2048};

pub fn map_gaussian(value: &U2048, q: &U2048) -> U2048 {
    let odd_q = Option::from(Odd::new(*q)).expect("Modulus must be odd");
    let params = FixedMontyParams::new(odd_q);
    let val_res = FixedMontyForm::new(value, &params);
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
