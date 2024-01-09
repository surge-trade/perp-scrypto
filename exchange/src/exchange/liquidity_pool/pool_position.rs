use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct PoolPosition {
    pub long_oi: Decimal,
    pub short_oi: Decimal,
    pub long_funding_checkpoint: Decimal,
    pub short_funding_checkpoint: Decimal,
    pub borrowing_checkpoint: Decimal,
}
