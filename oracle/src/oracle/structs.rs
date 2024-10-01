use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, Debug)]
pub struct Price {
    pub pair: String,
    pub quote: Decimal,
    pub timestamp: Instant,
}
