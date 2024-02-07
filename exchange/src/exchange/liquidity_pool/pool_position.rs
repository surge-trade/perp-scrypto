use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct PoolPosition {
    pub oi_long: Decimal,
    pub oi_short: Decimal,
    pub cost: Decimal,
    pub funding_rate: Decimal,
    pub funding_long_index: Decimal,
    pub funding_short_index: Decimal,
    pub pnl_snap: Decimal,
    pub last_update: Instant,
}
