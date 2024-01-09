use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct Config {
    pub max_price_age_seconds: i64,
    pub keeper_fee: Decimal,
    pub min_collateral_ratio: Decimal,
    pub max_loss_factor: Decimal,
    pub max_oi_long_factor: Decimal,
    pub max_oi_short_factor: Decimal,
    pub max_oi_net_factor: Decimal,
    pub funding_rate_factor: Decimal,
    pub borrowing_long_rate_factor: Decimal,
    pub borrowing_short_rate_factor: Decimal,
    pub borrowing_smaller_side_discount_factor: Decimal,
    pub swap_fee_factor: Decimal,
    pub swap_impact_factor: Decimal,
    pub swap_impact_exp: Decimal,
    pub margin_fee_factor: Decimal,
    pub margin_impact_factor: Decimal,
    pub margin_impact_exp: Decimal,
}
