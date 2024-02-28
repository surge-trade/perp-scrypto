use scrypto::prelude::*;

use crate::utils::List;

#[derive(ScryptoSbor)]
pub struct ExchangeConfig {
    pub max_price_age_seconds: i64,
    pub keeper_fee: Decimal,
    pub positions_max: u64,
    pub skew_ratio_cap: Decimal,
    pub adl_offset: Decimal,
    pub adl_a: Decimal,
    pub adl_b: Decimal,
    pub fee_liquidity: Decimal,
    pub fee_max: Decimal,
    pub pairs: List<PairConfig>,
    pub collaterals: HashMap<ResourceAddress, CollateralConfig>
}

#[derive(ScryptoSbor, Clone)]
pub struct PairConfig {
    pub pair_id: u64,
    pub disabled: bool,
    pub margin_initial: Decimal,
    pub margin_maintenance: Decimal, // TODO: separate this into a separate struct
    pub funding_1: Decimal,
    pub funding_2: Decimal,
    pub funding_2_delta: Decimal,
    pub funding_pool_0: Decimal,
    pub funding_pool_1: Decimal,
    pub funding_share: Decimal,
    pub fee_0: Decimal,
    pub fee_1: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct CollateralConfig {
    pub disabled: bool,
    pub discount: Decimal,
}
