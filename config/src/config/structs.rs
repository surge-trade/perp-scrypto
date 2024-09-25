use scrypto::prelude::*;
use common::{PairId, DFloat16};

pub struct ConfigInfo {
    pub exchange: ExchangeConfig,
    pub pair_configs: HashMap<PairId, Option<PairConfig>>,
    pub collaterals: HashMap<ResourceAddress, CollateralConfig>,
}

#[derive(ScryptoSbor)]
pub struct ConfigInfoCompressed {
    pub exchange: ExchangeConfigCompressed,
    pub pair_configs: HashMap<PairId, Option<PairConfigCompressed>>,
    pub collaterals: HashMap<ResourceAddress, CollateralConfigCompressed>,
}


impl ConfigInfoCompressed {
    pub fn decompress(&self) -> ConfigInfo {
        ConfigInfo {
            exchange: self.exchange.decompress(),
            pair_configs: self.pair_configs.iter().map(|(pair_id, config)| (pair_id.to_owned(), config.as_ref().map(|c| c.decompress()))).collect(),
            collaterals: self.collaterals.iter().map(|(resource, config)| (*resource, config.decompress())).collect(),
        }
    }
}

#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct ExchangeConfig {
    /// Maximum allowed age of the price in seconds
    pub max_price_age_seconds: i64,
    /// Maximum allowed number of positions per account
    pub positions_max: u16,
    /// Maximum allowed number of collaterals per account
    pub collaterals_max: u16,
    /// Maximum allowed number of active requests per account
    pub active_requests_max: u16,
    /// Maximum skew ratio allowed before skew increasing orders can not be made
    pub skew_ratio_cap: Decimal,
    /// ADL offset calculation parameter
    pub adl_offset: Decimal,
    /// ADL A calculation parameter
    pub adl_a: Decimal,
    /// ADL B calculation parameter
    pub adl_b: Decimal,
    /// Fee for adding liquidity
    pub fee_liquidity_add: Decimal,
    /// Fee for removing liquidity
    pub fee_liquidity_remove: Decimal,
    /// Share of fees that goes to the protocol
    pub fee_share_protocol: Decimal,
    /// Share of fees that goes to the treasury
    pub fee_share_treasury: Decimal,
    /// Share of fees that goes to the referrer
    pub fee_share_referral: Decimal,
    /// Maximum fee rate that can be charged
    pub fee_max: Decimal,
    /// Amount to burn for protocol fees
    pub protocol_burn_amount: Decimal,
    /// Keeper reward amount
    pub reward_keeper: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct ExchangeConfigCompressed {
    /// Maximum allowed age of the price in seconds
    pub max_price_age_seconds: u32,
    /// Maximum allowed number of positions per account
    pub positions_max: u16,
    /// Maximum allowed number of collaterals per account
    pub collaterals_max: u16,
    /// Maximum allowed number of active requests per account
    pub active_requests_max: u16,
    /// Maximum skew ratio allowed before skew increasing orders can not be made
    pub skew_ratio_cap: DFloat16,
    /// ADL offset calculation parameter
    pub adl_offset: DFloat16,
    /// ADL A calculation parameter
    pub adl_a: DFloat16,
    /// ADL B calculation parameter
    pub adl_b: DFloat16,
    /// Fee for adding liquidity
    pub fee_liquidity_add: DFloat16,
    /// Fee for removing liquidity
    pub fee_liquidity_remove: DFloat16,
    /// Share of fees that goes to the protocol
    pub fee_share_protocol: DFloat16,
    /// Share of fees that goes to the treasury
    pub fee_share_treasury: DFloat16,
    /// Share of fees that goes to the referrer
    pub fee_share_referral: DFloat16,
    /// Maximum fee rate that can be charged
    pub fee_max: DFloat16,
    /// Amount to burn for protocol fees
    pub protocol_burn_amount: DFloat16,
    /// Keeper reward amount
    pub keeper_reward: DFloat16,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            max_price_age_seconds: 5,
            positions_max: 30,
            collaterals_max: 5,
            active_requests_max: 100,
            skew_ratio_cap: dec!(0.15),
            adl_offset: dec!(0.2),
            adl_a: dec!(0.07),
            adl_b: dec!(0.07),
            fee_liquidity_add: dec!(0.001),
            fee_liquidity_remove: dec!(0.001),
            fee_share_protocol: dec!(0.22),
            fee_share_treasury: dec!(0.08),
            fee_share_referral: dec!(1),
            fee_max: dec!(0.01),
            protocol_burn_amount: dec!(10000),
            reward_keeper: dec!(1),
        }
    }
}

impl ExchangeConfig {
    pub fn validate(&self) {
        assert!(self.max_price_age_seconds > 0, "Invalid max price age");
        assert!(self.positions_max > 0, "Invalid max positions");
        assert!(self.skew_ratio_cap >= dec!(0), "Invalid skew ratio cap");
        assert!(self.adl_offset >= dec!(0), "Invalid adl offset");
        assert!(self.adl_a >= dec!(0), "Invalid adl a");
        assert!(self.adl_b >= dec!(0), "Invalid adl b");
        assert!(self.fee_liquidity_add >= dec!(0), "Invalid liquidity fee");
        assert!(self.fee_liquidity_remove >= dec!(0), "Invalid liquidity fee");
        assert!(self.fee_share_protocol >= dec!(0) && self.fee_share_protocol <= dec!(0.3), "Invalid protocol fee");
        assert!(self.fee_share_treasury >= dec!(0) && self.fee_share_treasury <= dec!(0.15), "Invalid treasury fee");
        assert!(self.fee_share_referral >= dec!(0) && self.fee_share_referral <= dec!(1), "Invalid referral fee");
        assert!(self.fee_max >= dec!(0), "Invalid max fee");
        assert!(self.protocol_burn_amount >= dec!(1000), "Invalid protocol burn amount");
        assert!(self.reward_keeper >= dec!(0), "Invalid keeper reward");
    }

    pub fn compress(&self) -> ExchangeConfigCompressed {
        ExchangeConfigCompressed {
            max_price_age_seconds: self.max_price_age_seconds as u32,
            positions_max: self.positions_max,
            collaterals_max: self.collaterals_max,
            active_requests_max: self.active_requests_max,
            skew_ratio_cap: DFloat16::from(self.skew_ratio_cap),
            adl_offset: DFloat16::from(self.adl_offset),
            adl_a: DFloat16::from(self.adl_a),
            adl_b: DFloat16::from(self.adl_b),
            fee_liquidity_add: DFloat16::from(self.fee_liquidity_add),
            fee_liquidity_remove: DFloat16::from(self.fee_liquidity_remove),
            fee_share_protocol: DFloat16::from(self.fee_share_protocol),
            fee_share_treasury: DFloat16::from(self.fee_share_treasury),
            fee_share_referral: DFloat16::from(self.fee_share_referral),
            fee_max: DFloat16::from(self.fee_max),
            protocol_burn_amount: DFloat16::from(self.protocol_burn_amount),
            keeper_reward: DFloat16::from(self.reward_keeper),
        }
    }
}

impl ExchangeConfigCompressed {
    pub fn decompress(&self) -> ExchangeConfig {
        ExchangeConfig {
            max_price_age_seconds: self.max_price_age_seconds as i64,
            positions_max: self.positions_max,
            collaterals_max: self.collaterals_max,
            active_requests_max: self.active_requests_max,
            skew_ratio_cap: self.skew_ratio_cap.into(),
            adl_offset: self.adl_offset.into(),
            adl_a: self.adl_a.into(),
            adl_b: self.adl_b.into(),
            fee_liquidity_add: self.fee_liquidity_add.into(),
            fee_liquidity_remove: self.fee_liquidity_remove.into(),
            fee_share_protocol: self.fee_share_protocol.into(),
            fee_share_treasury: self.fee_share_treasury.into(),
            fee_share_referral: self.fee_share_referral.into(),
            fee_max: self.fee_max.into(),
            protocol_burn_amount: self.protocol_burn_amount.into(),
            reward_keeper: self.keeper_reward.into(),
        }
    }
}

#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct PairConfig {
    /// Price feed id
    pub pair_id: PairId,
    /// Maximum allowed combined oi for the pair
    pub oi_max: Decimal,
    /// Minimum trade size 
    pub trade_size_min: Decimal,
    /// Price delta ratio before updating a pair will be rewarded
    pub update_price_delta_ratio: Decimal,
    /// Time before updating a pair will be rewarded
    pub update_period_seconds: i64,
    /// Initial margin required
    pub margin_initial: Decimal,
    /// Maintenance margin required
    pub margin_maintenance: Decimal,
    /// Skew based funding 
    pub funding_1: Decimal,
    /// Integral of skew based funding
    pub funding_2: Decimal,
    /// Rate of change of funding 2 integral
    pub funding_2_delta: Decimal,
    /// Rate of decay for funding 2 if it is out of bounds
    pub funding_2_decay: Decimal,
    /// Constant pool funding
    pub funding_pool_0: Decimal,
    /// Skew based pool funding
    pub funding_pool_1: Decimal,
    /// Share of regular funding taken as pool funding
    pub funding_share: Decimal,
    /// Constant fee
    pub fee_0: Decimal,
    /// Price impact fee
    pub fee_1: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct PairConfigCompressed {
    /// Price feed id
    pub pair_id: PairId,
    /// Maximum allowed combined oi for the pair
    pub oi_max: DFloat16,
    /// Minimum trade size 
    pub trade_size_min: DFloat16,
    /// Price delta ratio before updating a pair will be rewarded
    pub update_price_delta_ratio: DFloat16,
    /// Time before updating a pair will be rewarded
    pub update_period_seconds: u16,
    /// Initial margin required
    pub margin_initial: DFloat16,
    /// Maintenance margin required
    pub margin_maintenance: DFloat16,
    /// Skew based funding 
    pub funding_1: DFloat16,
    /// Integral of skew based funding
    pub funding_2: DFloat16,
    /// Rate of change of funding 2 integral
    pub funding_2_delta: DFloat16,
    /// Rate of decay for funding 2 if it is out of bounds
    pub funding_2_decay: DFloat16,
    /// Constant pool funding
    pub funding_pool_0: DFloat16,
    /// Skew based pool funding
    pub funding_pool_1: DFloat16,
    /// Share of regular funding taken as pool funding
    pub funding_share: DFloat16,
    /// Constant fee
    pub fee_0: DFloat16,
    /// Price impact fee
    pub fee_1: DFloat16,
}

impl PairConfig {
    pub fn validate(&self) {
        assert!(self.oi_max >= dec!(0), "Invalid oi maximum");
        assert!(self.trade_size_min >= dec!(0), "Invalid minimum trade size");
        assert!(self.update_price_delta_ratio > dec!(0), "Invalid pair update price delta ratio");
        assert!(self.update_period_seconds > 0, "Invalid pair update period");
        assert!(self.margin_initial >= dec!(0) && self.margin_initial <= dec!(1), "Invalid initial margin");
        assert!(self.margin_maintenance >= dec!(0) && self.margin_maintenance <= dec!(1) && self.margin_maintenance <= self.margin_initial, "Invalid maintenance margin");
        assert!(self.funding_1 >= dec!(0) && self.funding_1 <= dec!(2), "Invalid funding 1");
        assert!(self.funding_2 >= dec!(0) && self.funding_2 <= dec!(4), "Invalid funding 2");
        assert!(self.funding_2_delta >= dec!(0) && self.funding_2_delta <= dec!(10000), "Invalid funding 2 delta");
        assert!(self.funding_2_decay >= dec!(0), "Invalid funding 2 decay");
        assert!(self.funding_pool_0 >= dec!(0) && self.funding_pool_0 <= dec!(1), "Invalid funding pool 0");
        assert!(self.funding_pool_1 >= dec!(0) && self.funding_pool_1 <= dec!(2), "Invalid funding pool 1");
        assert!(self.funding_share >= dec!(0) && self.funding_share <= dec!(0.1), "Invalid funding share");
        assert!(self.fee_0 >= dec!(0) && self.fee_0 <= dec!(0.015), "Invalid fee 0");
        assert!(self.fee_1 >= dec!(0) && self.fee_1 <= dec!(0.000001), "Invalid fee 1");
    }

    pub fn compress(&self) -> PairConfigCompressed {
        PairConfigCompressed {
            pair_id: self.pair_id.to_owned(),
            oi_max: DFloat16::from(self.oi_max),
            trade_size_min: DFloat16::from(self.trade_size_min),
            update_price_delta_ratio: DFloat16::from(self.update_price_delta_ratio),
            update_period_seconds: self.update_period_seconds as u16,
            margin_initial: DFloat16::from(self.margin_initial),
            margin_maintenance: DFloat16::from(self.margin_maintenance),
            funding_1: DFloat16::from(self.funding_1),
            funding_2: DFloat16::from(self.funding_2),
            funding_2_delta: DFloat16::from(self.funding_2_delta),
            funding_2_decay: DFloat16::from(self.funding_2_decay),
            funding_pool_0: DFloat16::from(self.funding_pool_0),
            funding_pool_1: DFloat16::from(self.funding_pool_1),
            funding_share: DFloat16::from(self.funding_share),
            fee_0: DFloat16::from(self.fee_0),
            fee_1: DFloat16::from(self.fee_1),
        }
    }
}

impl PairConfigCompressed {
    pub fn decompress(&self) -> PairConfig {
        PairConfig {
            pair_id: self.pair_id.to_owned(),
            oi_max: self.oi_max.into(),
            trade_size_min: self.trade_size_min.into(),
            update_price_delta_ratio: self.update_price_delta_ratio.into(),
            update_period_seconds: self.update_period_seconds as i64,
            margin_initial: self.margin_initial.into(),
            margin_maintenance: self.margin_maintenance.into(),
            funding_1: self.funding_1.into(),
            funding_2: self.funding_2.into(),
            funding_2_delta: self.funding_2_delta.into(),
            funding_2_decay: self.funding_2_decay.into(),
            funding_pool_0: self.funding_pool_0.into(),
            funding_pool_1: self.funding_pool_1.into(),
            funding_share: self.funding_share.into(),
            fee_0: self.fee_0.into(),
            fee_1: self.fee_1.into(),
        }
    }
}

#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct CollateralConfig {
    /// Price feed id
    pub pair_id: PairId,
    /// Discount applied to the collateral
    pub discount: Decimal,
    /// Margin required for the collateral
    pub margin: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct CollateralConfigCompressed {
    /// Price feed id
    pub pair_id: PairId,
    /// Discount applied to the collateral
    pub discount: DFloat16,
    /// Margin required for the collateral
    pub margin: DFloat16,
}

impl CollateralConfig {
    pub fn validate(&self) {
        assert!(self.discount >= dec!(0) && self.discount <= dec!(1), "Invalid discount");
        assert!(self.margin >= dec!(0) && self.margin <= dec!(0.1), "Invalid margin");
    }

    pub fn compress(&self) -> CollateralConfigCompressed {
        CollateralConfigCompressed {
            pair_id: self.pair_id.to_owned(),
            discount: DFloat16::from(self.discount),
            margin: DFloat16::from(self.margin),
        }
    }
}

impl CollateralConfigCompressed {
    pub fn decompress(&self) -> CollateralConfig {
        CollateralConfig {
            pair_id: self.pair_id.to_owned(),
            discount: self.discount.into(),
            margin: self.margin.into(),
        }
    }
}
