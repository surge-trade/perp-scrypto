use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct ExchangeConfig {
    pub tradable_resources: Vec<ResourceAddress>,
    pub max_price_age_seconds: i64,
    pub keeper_fee: Decimal,
    pub min_collateral_ratio: Decimal,
    pub max_skew: Decimal,
    pub base_fee: Decimal,
    pub skew_funding_delta: Decimal,
    pub skew_premium: Decimal,
}
