use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct PoolPosition {
    pub long_oi: Decimal,
    pub short_oi: Decimal,
    pub interest_long_checkpoint: Decimal,
    pub interest_short_checkpoint: Decimal,
}
