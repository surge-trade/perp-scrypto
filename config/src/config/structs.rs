use scrypto::prelude::*;
use common::{PairId, DFloat16};

pub struct ConfigInfo {
    pub exchange: ExchangeConfig,
    pub collaterals: HashMap<ResourceAddress, CollateralConfig>,
}

#[derive(ScryptoSbor)]
pub struct ConfigInfoCompressed {
    pub exchange: ExchangeConfigCompressed,
    pub collaterals: HashMap<ResourceAddress, CollateralConfigCompressed>,
}


impl ConfigInfoCompressed {
    pub fn decompress(&self) -> ConfigInfo {
        ConfigInfo {
            exchange: self.exchange.decompress(),
            collaterals: self.collaterals.iter().map(|(resource, config)| (*resource, config.decompress())).collect(),
        }
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct ExchangeConfig {
    /// Maximum allowed age of the price in seconds
    pub max_price_age_seconds: i64,
    /// Price delta ratio before updating a pair will be rewarded
    pub pair_update_price_delta_ratio: Decimal,
    /// Time be before updating a pair will be rewarded
    pub pair_update_period_seconds: i64,
    /// Flat fee to cover the keeper's expenses
    pub keeper_fee: Decimal,
    /// Maximum allowed number of positions per account
    pub positions_max: u16,
    /// Maximum skew ratio allowed before skew increasing orders can not be made
    pub skew_ratio_cap: Decimal,
    /// ADL offset calculation parameter
    pub adl_offset: Decimal,
    /// ADL A calculation parameter
    pub adl_a: Decimal,
    /// ADL B calculation parameter
    pub adl_b: Decimal,
    /// Fee for adding and removing liquidity
    pub fee_liquidity: Decimal,
    /// Share of fees that go to the referrer
    pub fee_share_referral: Decimal,
    /// Maximum fee that can be charged
    pub fee_max: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct ExchangeConfigCompressed {
    /// Maximum allowed age of the price in seconds
    pub max_price_age_seconds: u32,
    /// Price delta ratio before updating a pair will be rewarded
    pub pair_update_price_delta_ratio: DFloat16,
    /// Time be before updating a pair will be rewarded
    pub pair_update_period_seconds: u16,
    /// Flat fee to cover the keeper's expenses
    pub keeper_fee: DFloat16,
    /// Maximum allowed number of positions per account
    pub positions_max: u16,
    /// Maximum skew ratio allowed before skew increasing orders can not be made
    pub skew_ratio_cap: DFloat16,
    /// ADL offset calculation parameter
    pub adl_offset: DFloat16,
    /// ADL A calculation parameter
    pub adl_a: DFloat16,
    /// ADL B calculation parameter
    pub adl_b: DFloat16,
    /// Fee for adding and removing liquidity
    pub fee_liquidity: DFloat16,
    /// Share of fees that go to the referrer
    pub fee_share_referral: DFloat16,
    /// Maximum fee that can be charged
    pub fee_max: DFloat16,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            max_price_age_seconds: 60,
            pair_update_price_delta_ratio: dec!(0.001),
            pair_update_period_seconds: 120,
            keeper_fee: dec!(0.01),
            positions_max: 10,
            skew_ratio_cap: dec!(0.1),
            adl_offset: dec!(0.1),
            adl_a: dec!(0.1),
            adl_b: dec!(0.1),
            fee_liquidity: dec!(0.01),
            fee_share_referral: dec!(0.1),
            fee_max: dec!(0.1),
        }
    }
}

impl ExchangeConfig {
    pub fn validate(&self) {
        assert!(self.max_price_age_seconds > 0, "Invalid max price age");
        assert!(self.pair_update_price_delta_ratio >= dec!(0), "Invalid pair update price delta ratio");
        assert!(self.pair_update_period_seconds >= 0, "Invalid pair update period");
        assert!(self.keeper_fee >= dec!(0), "Invalid keeper fee");
        assert!(self.positions_max > 0, "Invalid max positions");
        assert!(self.skew_ratio_cap >= dec!(0), "Invalid skew ratio cap");
        assert!(self.adl_offset >= dec!(0), "Invalid adl offset");
        assert!(self.adl_a >= dec!(0), "Invalid adl a");
        assert!(self.adl_b >= dec!(0), "Invalid adl b");
        assert!(self.fee_liquidity >= dec!(0), "Invalid liquidity fee");
        assert!(self.fee_max >= dec!(0), "Invalid max fee");
    }

    pub fn compress(&self) -> ExchangeConfigCompressed {
        ExchangeConfigCompressed {
            max_price_age_seconds: self.max_price_age_seconds as u32,
            pair_update_price_delta_ratio: DFloat16::from(self.pair_update_price_delta_ratio),
            pair_update_period_seconds: self.pair_update_period_seconds as u16,
            keeper_fee: DFloat16::from(self.keeper_fee),
            positions_max: self.positions_max,
            skew_ratio_cap: DFloat16::from(self.skew_ratio_cap),
            adl_offset: DFloat16::from(self.adl_offset),
            adl_a: DFloat16::from(self.adl_a),
            adl_b: DFloat16::from(self.adl_b),
            fee_liquidity: DFloat16::from(self.fee_liquidity),
            fee_share_referral: DFloat16::from(self.fee_share_referral),
            fee_max: DFloat16::from(self.fee_max),
        }
    }
}

impl ExchangeConfigCompressed {
    pub fn decompress(&self) -> ExchangeConfig {
        ExchangeConfig {
            max_price_age_seconds: self.max_price_age_seconds as i64,
            pair_update_price_delta_ratio: self.pair_update_price_delta_ratio.into(),
            pair_update_period_seconds: self.pair_update_period_seconds as i64,
            keeper_fee: self.keeper_fee.into(),
            positions_max: self.positions_max,
            skew_ratio_cap: self.skew_ratio_cap.into(),
            adl_offset: self.adl_offset.into(),
            adl_a: self.adl_a.into(),
            adl_b: self.adl_b.into(),
            fee_liquidity: self.fee_liquidity.into(),
            fee_share_referral: self.fee_share_referral.into(),
            fee_max: self.fee_max.into(),
        }
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct PairConfig {
    /// Price feed id
    pub pair_id: PairId,
    /// If the pair is disabled
    pub disabled: bool,
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
    /// Constant pool funding
    pub funding_pool_0: Decimal,
    /// Skew based pool funding
    pub funding_pool_1: Decimal,
    /// Share of regular funding taken as pool funding
    pub funding_share: Decimal,
    /// Constant fee
    pub fee_0: Decimal,
    /// Skew based fee
    pub fee_1: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct PairConfigCompressed {
    /// Price feed id
    pub pair_id: PairId,
    /// If the pair is disabled
    pub disabled: bool,
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
    /// Constant pool funding
    pub funding_pool_0: DFloat16,
    /// Skew based pool funding
    pub funding_pool_1: DFloat16,
    /// Share of regular funding taken as pool funding
    pub funding_share: DFloat16,
    /// Constant fee
    pub fee_0: DFloat16,
    /// Skew based fee
    pub fee_1: DFloat16,
}

impl PairConfig {
    pub fn validate(&self) {
        assert!(self.margin_initial >= dec!(0), "Invalid initial margin");
        assert!(self.margin_maintenance >= dec!(0), "Invalid maintenance margin");
        assert!(self.funding_1 >= dec!(0), "Invalid funding 1");
        assert!(self.funding_2 >= dec!(0), "Invalid funding 2");
        assert!(self.funding_2_delta >= dec!(0), "Invalid funding 2 delta");
        assert!(self.funding_pool_0 >= dec!(0), "Invalid funding pool 0");
        assert!(self.funding_pool_1  >= dec!(0), "Invalid funding pool 1");
        assert!(self.funding_share >= dec!(0), "Invalid funding share");
        assert!(self.fee_0  >= dec!(0), "Invalid fee 0");
        assert!(self.fee_1 >= dec!(0), "Invalid fee 1");
    }

    pub fn compress(&self) -> PairConfigCompressed {
        PairConfigCompressed {
            pair_id: self.pair_id,
            disabled: self.disabled,
            margin_initial: DFloat16::from(self.margin_initial),
            margin_maintenance: DFloat16::from(self.margin_maintenance),
            funding_1: DFloat16::from(self.funding_1),
            funding_2: DFloat16::from(self.funding_2),
            funding_2_delta: DFloat16::from(self.funding_2_delta),
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
            pair_id: self.pair_id,
            disabled: self.disabled,
            margin_initial: self.margin_initial.into(),
            margin_maintenance: self.margin_maintenance.into(),
            funding_1: self.funding_1.into(),
            funding_2: self.funding_2.into(),
            funding_2_delta: self.funding_2_delta.into(),
            funding_pool_0: self.funding_pool_0.into(),
            funding_pool_1: self.funding_pool_1.into(),
            funding_share: self.funding_share.into(),
            fee_0: self.fee_0.into(),
            fee_1: self.fee_1.into(),
        }
    }
}

#[derive(ScryptoSbor, Clone)]
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
        assert!(self.discount >= dec!(0), "Invalid discount");
        assert!(self.margin >= dec!(0), "Invalid margin");
    }

    pub fn compress(&self) -> CollateralConfigCompressed {
        CollateralConfigCompressed {
            pair_id: self.pair_id,
            discount: DFloat16::from(self.discount),
            margin: DFloat16::from(self.margin),
        }
    }
}

impl CollateralConfigCompressed {
    pub fn decompress(&self) -> CollateralConfig {
        CollateralConfig {
            pair_id: self.pair_id,
            discount: self.discount.into(),
            margin: self.margin.into(),
        }
    }
}
