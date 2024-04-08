use scrypto::prelude::*;

/// Decimal floating point number
///
/// This is a 16-bit floating point number with 5 bits for the exponent and 11 bits for the significand.
#[derive(ScryptoSbor, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DFloat16(u16);

impl DFloat16 {
    /// Creates a new `DFloat16` from a given exponent, and significand.
    ///
    /// # Arguments
    ///
    /// * `exp` - A 5-bit exponent as a `u8`.
    /// * `sig` - A 11-bit significand as a `u16`.
    ///
    /// # Returns
    ///
    /// A new `DFloat16`.
    ///
    /// # Panics
    ///
    /// This function will panic if `exp` has bits set outside of the 5-bit range, or if `sig` has bits set outside of the 11-bit range.
    pub fn new(exp: i16, sig: i16) -> Self {
        let exp: u16 = (exp + 15) as u16;
        let sig: u16 = (sig + 1023) as u16;
        assert!(exp < 32, "Exponent out of range");
        assert!(sig < 2048, "Significand out of range");

        let exp_bits = exp << 11;
        let sig_bits = sig;

        DFloat16(exp_bits | sig_bits)
    }
}

impl Into<Decimal> for DFloat16 {
    fn into(self) -> Decimal {
        Decimal(I192::from((self.0 & 2047) as i16 - 1023) * I192::TEN.pow((self.0 >> 11) as u32))
    }
}

impl From<Decimal> for DFloat16 {
    fn from(value: Decimal) -> Self {
        if value.is_zero() {
            return DFloat16::new(0, 0);
        }

        let pos_2: u32 = I192::BITS - value.0.abs().leading_zeros();
        let pos_10: u32 = (pos_2 * 30103 / 100000).max(3);

        let divisor: I192 = I192::TEN.pow(pos_10 - 3);
        let mut exp: i16 = pos_10 as i16 - 18;
        let mut sig: i16 = (value.0 / divisor).to_i16().unwrap();

        if sig > 1024 || sig < -1023 {
            sig /= 10;
            exp += 1;
        }

        Self::new(exp as i16, sig as i16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let d_float = DFloat16::new(0, 0);
        assert_eq!(d_float.0, 31743);

        let d_float = DFloat16::new(-15, -1023);
        assert_eq!(d_float.0, 0);

        let d_float = DFloat16::new(16, 1024);
        assert_eq!(d_float.0, 65535);
    }

    #[test]
    fn test_into() {
        let d_float = DFloat16::new(0, 0);
        let decimal: Decimal = d_float.into();
        assert_eq!(decimal, dec!(0));

        let d_float = DFloat16::new(-15, -1023);
        let decimal: Decimal = d_float.into();
        assert_eq!(decimal, dec!(-0.000000000000001023));

        let d_float = DFloat16::new(16, 1024);
        let decimal: Decimal = d_float.into();
        assert_eq!(decimal, dec!(10240000000000000));
    }

    #[test]
    fn test_from() {
        let decimal = dec!(0);
        let d_float: DFloat16 = decimal.into();
        assert_eq!(d_float.0, DFloat16::new(0, 0).0);

        let decimal = dec!(-0.000000000000001023);
        let d_float: DFloat16 = decimal.into();
        assert_eq!(d_float.0, DFloat16::new(-15, -1023).0);

        let decimal = dec!(10240000000000000);
        let d_float: DFloat16 = decimal.into();
        assert_eq!(d_float.0, DFloat16::new(16, 1024).0);
    }

    #[test]
    fn test_loop() {
        for exp in -15..17 {
            for sig in -1023..1025 {
                let dec = Decimal(I192::from(sig) * I192::TEN.pow((exp + 15) as u32));
                println!("Decimal: {}", dec);
                let d_float: DFloat16 = dec.into();

                let reversed: Decimal = d_float.into();
                assert_eq!(reversed, dec);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Exponent out of range")]
    fn test_new_panic_exponent_pos() {
        DFloat16::new(17, 0); // Exponent out of range
    }

    #[test]
    #[should_panic(expected = "Exponent out of range")]
    fn test_new_panic_exponent_neg() {
        DFloat16::new(-16, 0); // Exponent out of range
    }

    #[test]
    #[should_panic(expected = "Significand out of range")]
    fn test_new_panic_significand_pos() {
        DFloat16::new(0, 1025); // Significand out of range
    }

    #[test]
    #[should_panic(expected = "Significand out of range")]
    fn test_new_panic_significand_neg() {
        DFloat16::new(0, -1024); // Significand out of range
    }
}


