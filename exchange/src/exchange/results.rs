use scrypto::prelude::*;
use common::*;
use super::requests::Limit;

pub struct ResultAddLiquidity {
    pub lp_token: Bucket,
    pub amount: Decimal,
    pub lp_amount: Decimal,
    pub lp_price: Decimal,
    pub fee_liquidity: Decimal,
}

pub struct ResultRemoveLiquidity {
    pub token: Bucket,
    pub amount: Decimal,
    pub lp_amount: Decimal,
    pub lp_price: Decimal,
    pub fee_liquidity: Decimal,
}

pub struct ResultAddCollateral {
    pub amounts: Vec<(ResourceAddress, Decimal)>,
}

pub struct ResultRemoveCollateral {
    pub target_account: ComponentAddress,
    pub amounts: Vec<(ResourceAddress, Decimal)>,
}

pub enum ResultProcessRequest {
    RemoveCollateral(ResultRemoveCollateral),
    MarginOrder(ResultMarginOrder),
}

pub struct ResultMarginOrder {
    pub pair_id: PairId,
    pub price_limit: Limit,
    pub amount_close: Decimal,
    pub amount_open: Decimal,
    pub activated_requests: Vec<ListIndex>,
    pub cancelled_requests: Vec<ListIndex>,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub price: Decimal,
}

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

pub struct ResultSwapDebt {
    pub token: Bucket,
    pub remainder: Bucket,
    pub amount: Decimal,
    pub price: Decimal,
}

pub struct ResultLiquidate {
    pub tokens: Vec<Bucket>,
    pub position_amounts: Vec<(PairId, Decimal)>,
    pub collateral_amounts: Vec<(ResourceAddress, Decimal)>,
    pub collateral_value: Decimal,
    pub margin: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub position_prices: Vec<(PairId, Decimal)>,
    pub collateral_prices: Vec<(ResourceAddress, Decimal)>,
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
