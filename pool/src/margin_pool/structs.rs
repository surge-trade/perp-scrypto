use scrypto::prelude::*;
use common::PairId;

#[derive(ScryptoSbor, Clone)]
pub struct PoolPosition {
    pub oi_long: Decimal,
    pub oi_short: Decimal,
    pub cost: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
    pub funding_2_rate: Decimal,
    pub funding_long_index: Decimal,
    pub funding_short_index: Decimal,
    pub last_update: Instant,
    pub last_price: Decimal,
}

impl Default for PoolPosition {
    fn default() -> Self {
        PoolPosition {
            oi_long: dec!(0),
            oi_short: dec!(0),
            cost: dec!(0),
            skew_abs_snap: dec!(0),
            pnl_snap: dec!(0),
            funding_2_rate: dec!(0),
            funding_long_index: dec!(0),
            funding_short_index: dec!(0),
            last_update: Clock::current_time_rounded_to_seconds(),
            last_price: dec!(1),
        }
    }
}

#[derive(ScryptoSbor)]
pub struct MarginPoolInfo {
    pub positions: HashMap<PairId, PoolPosition>,
    pub base_tokens_amount: Decimal,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginPoolUpdates {
    pub position_updates: HashMap<PairId, PoolPosition>,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
}