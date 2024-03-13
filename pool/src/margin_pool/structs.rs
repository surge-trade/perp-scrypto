use scrypto::prelude::*;

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
}

#[derive(ScryptoSbor)]
pub struct MarginPoolInfo {
    pub base_tokens_amount: Decimal,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
    pub lp_token_manager: ResourceManager,
}

#[derive(ScryptoSbor)]
pub struct MarginPoolUpdates {
    pub position_updates: HashMap<u64, PoolPosition>,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
}