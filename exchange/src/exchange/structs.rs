use scrypto::prelude::*;
use super::requests::*;
use common::*;
use ::config::*;
use account::*;
use pool::*;
use referral_generator::{ReferralData, ReferralAllocation, ReferralCode};

#[derive(ScryptoSbor, NonFungibleData, Clone, ManifestSbor)]
pub struct RecoveryKeyData {
    #[mutable] pub name: String,
    #[mutable] pub description: String,
    #[mutable] pub key_image_url: Url,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct PositionDetails {
    pub pair_id: PairId,
    pub amount: Decimal,
    pub margin_initial: Decimal,
    pub margin_maintenance: Decimal,
    pub cost: Decimal,
    pub funding: Decimal,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct CollateralDetails {
    pub pair_id: PairId,
    pub resource: ResourceAddress,
    pub amount: Decimal,
    pub discount: Decimal,
    pub margin: Decimal,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct RequestDetails {
    pub index: ListIndex,
    pub request: Request,
    pub submission: Instant,
    pub expiry: Instant,
    pub status: Status,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct AccountDetails {
    pub virtual_balance: Decimal,
    pub positions: Vec<PositionDetails>,
    pub collaterals: Vec<CollateralDetails>,
    pub valid_requests_start: ListIndex,
    pub active_requests: Vec<RequestDetails>,
    pub requests_history: Vec<RequestDetails>,
    pub requests_len: ListIndex,
    pub referral: Option<(NonFungibleGlobalId, ReferralData)>,
}

#[derive(ScryptoSbor, Clone, Debug)]
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

#[derive(ScryptoSbor, Clone, Debug)]
pub struct PairDetails {
    pub pair_id: PairId,
    pub pool_position: PoolPosition,
    pub pair_config: PairConfig,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct ReferralDetails {
    pub referral: ReferralData,
    pub allocations: Vec<ReferralAllocation>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct ReferralCodeDetails {
    pub referral_code: ReferralCode,
    pub referral: ReferralData,
}

pub struct ResultValuePositions {
    pub pnl: Decimal,
    pub margin_positions: Decimal,
}

pub struct ResultLiquidatePositions {
    pub pnl: Decimal,
    pub margin_positions: Decimal,
    pub funding_paid: Decimal,
    pub fee_paid: Decimal,
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
