use scrypto::prelude::*;
use account::KeeperRequest;
use common::{PairId, ListIndex};
use pool::PoolPosition;
use ::config::*;
use super::requests::PriceLimit;

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventSignalUpgrade {
    pub new_exchange: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventExchangeConfigUpdate {
    pub config: ExchangeConfig,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventPairConfigUpdates {
    pub configs: Vec<PairConfig>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventCollateralConfigUpdates {
    pub configs: Vec<(ResourceAddress, CollateralConfig)>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventCollateralConfigRemoval {
    pub resource: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventPairUpdates {
    pub updates: Vec<(PairId, PoolPosition)>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventAccountCreation {
    pub account: ComponentAddress,
    pub referral_id: Option<NonFungibleLocalId>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventRequests {
    pub account: ComponentAddress,
    pub requests: Vec<(ListIndex, KeeperRequest)>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventValidRequestsStart {
    pub account: ComponentAddress,
    pub valid_requests_start: ListIndex,
}

// ---

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventLiquidityChange {
    pub lp_price: Decimal,
    pub lp_amount: Decimal,
    pub amount: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventAddCollateral {
    pub account: ComponentAddress,
    pub amounts: Vec<(ResourceAddress, Decimal)>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventRemoveCollateral {
    pub account: ComponentAddress,
    pub target_account: ComponentAddress,
    pub amounts: Vec<(ResourceAddress, Decimal)>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventMarginOrder {
    pub account: ComponentAddress,
    pub pair_id: PairId,
    pub price: Decimal,
    pub price_limit: PriceLimit,
    pub amount_close: Decimal,
    pub amount_open: Decimal,
    pub pnl: Decimal,
    pub funding: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub activated_requests: Vec<ListIndex>,
    pub cancelled_requests: Vec<ListIndex>,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventSwapDebt {
    pub account: ComponentAddress,
    pub resource: ResourceAddress,
    pub amount: Decimal,
    pub price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventLiquidate { 
    pub account: ComponentAddress,
    pub position_prices: Vec<(PairId, Decimal)>,
    pub collateral_prices: Vec<(ResourceAddress, Decimal)>,
    pub account_value: Decimal,
    pub margin: Decimal,
    pub virtual_balance: Decimal,
    pub position_amounts: Vec<(PairId, Decimal)>,
    pub positions_pnl: Decimal,
    pub collateral_amounts: Vec<(ResourceAddress, Decimal)>,
    pub collateral_value: Decimal,
    pub collateral_value_discounted: Decimal,
    pub funding: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub pool_loss: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventLiquidateToMargin {
    pub account: ComponentAddress,
    pub receiver: ComponentAddress,
    pub collateral_prices: Vec<(ResourceAddress, Decimal)>,
    pub collateral_amounts: Vec<(ResourceAddress, Decimal)>,
    pub collateral_value: Decimal,
    pub collateral_value_discounted: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct EventAutoDeleverage {
    pub account: ComponentAddress,
    pub pair_id: PairId,
    pub price: Decimal,
    pub amount_close: Decimal,
    pub pnl: Decimal,
    pub funding: Decimal,
    pub fee_pool: Decimal,
    pub fee_protocol: Decimal,
    pub fee_treasury: Decimal,
    pub fee_referral: Decimal,
    pub pnl_percent: Decimal,
    pub threshold: Decimal,
}
