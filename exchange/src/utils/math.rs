use scrypto::prelude::*;

pub trait Math {
    fn pow(&self, exponent: Decimal) -> Decimal;
}

impl Math for Decimal {
    fn pow(&self, exponent: Decimal) -> Decimal {
        // TODO: implement this
        *self
    }
}
