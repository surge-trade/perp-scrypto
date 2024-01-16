use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct ExchangeConfig {
    pub resource_configs: Vec<ResourceConfig>,
    pub max_price_age_seconds: i64,
    pub keeper_fee: Decimal,
    pub min_collateral_ratio: Decimal,
    pub collateral_buffer: Decimal,
    pub max_pool_loss: Decimal,
    pub funding_rate: Decimal,
    pub swap_base_fee: Decimal,
    pub swap_impact_fee: Decimal,
    pub swap_impact_exp: Decimal,
    pub margin_base_fee: Decimal,
    pub margin_impact_fee: Decimal,
    pub margin_impact_exp: Decimal,
    pub borrowing_long_rate: Decimal,
    pub borrowing_short_rate: Decimal,
    pub borrowing_discount: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct ResourceConfig {
    pub resource: ResourceAddress,
    pub weight: Decimal,
    pub max_oi_long_factor: Decimal,
    pub max_oi_net_factor: Decimal,
}