use scrypto::prelude::*;
use super::requests::*;
use common::*;
use config::*;
use account::*;

#[derive(ScryptoSbor, Clone)]
pub struct PositionDetails {
    pub pair_id: PairId,
    pub amount: Decimal,
    pub margin_initial: Decimal,
    pub margin_maintenance: Decimal,
    pub cost: Decimal,
    pub funding: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct CollateralDetails {
    pub resource: ResourceAddress,
    pub amount: Decimal,
    pub amount_discounted: Decimal,
    pub margin: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestDetails {
    pub index: ListIndex,
    pub request: Request,
    pub submission: Instant,
    pub expiry: Instant,
    pub status: Status,
}

#[derive(ScryptoSbor, Clone)]
pub struct AccountDetails {
    pub virtual_balance: Decimal,
    pub positions: Vec<PositionDetails>,
    pub collaterals: Vec<CollateralDetails>,
    pub valid_requests_start: ListIndex,
    pub active_requests: Vec<RequestDetails>,
    pub requests_history: Vec<RequestDetails>,
}

#[derive(ScryptoSbor, Clone)]
pub struct PoolDetails {
    pub base_tokens_amount: Decimal,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub pnl_snap: Decimal,
    pub skew_ratio: Decimal,
    pub skew_ratio_cap: Decimal,
    pub lp_supply: Decimal,
    pub lp_price: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct PairDetails {
    pub pair_id: PairId,
    pub oi_long: Decimal,
    pub oi_short: Decimal,
    pub funding_1: Decimal,
    pub funding_2: Decimal,
    pub funding_long: Decimal,
    pub funding_short: Decimal,
    pub funding_share: Decimal,
    pub pair_config: PairConfig,
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

pub struct ResultValuePositions {
    pub pnl: Decimal,
    pub margin_positions: Decimal,
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

pub struct ResultValueCollateral {
    pub collateral_value_discounted: Decimal,
    pub margin_collateral: Decimal,
}

pub struct ResultLiquidateCollateral {
    pub collateral_value: Decimal,
    pub collateral_value_discounted: Decimal,
    pub margin_collateral: Decimal,
    pub collateral_amounts: Vec<(ResourceAddress, Decimal)>,
    pub collateral_prices: Vec<(ResourceAddress, Decimal)>,
}
