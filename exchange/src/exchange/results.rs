use scrypto::prelude::*;
use common::*;

pub struct ResultOpenPosition {
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
}

pub struct ResultClosePosition {
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
}

pub struct ResultLiquidatePositions {
    pub pnl: Decimal,
    pub margin_positions: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub position_amounts: Vec<(PairId, Decimal)>,
    pub position_prices: Vec<(PairId, Decimal)>,
}

pub struct ResultLiquidateCollateral {
    pub collateral_value: Decimal,
    pub margin_collateral: Decimal,
    pub collateral_amounts: Vec<(ResourceAddress, Decimal)>,
    pub collateral_prices: Vec<(ResourceAddress, Decimal)>,
}

pub struct ResultAutoDeleverage {
    pub amount: Decimal,
    pub pnl_percent: Decimal,
    pub threshold: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub price: Decimal,
}
