use scrypto::prelude::*;
use utils::HashList;

#[derive(ScryptoSbor)]
pub struct Config {
    pub exchange: ExchangeConfig,
    pub pairs: HashList<u64, PairConfig>,
    pub collaterals: HashMap<ResourceAddress, CollateralConfig>, // TODO: HashList?
}

impl Config {
    pub fn new() -> Self {
        Self {
            exchange: ExchangeConfig::default(),
            pairs: HashList::new(),
            collaterals: HashMap::new(),
        }
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct ExchangeConfig {
    /// Maximum allowed age of the price in seconds
    pub max_price_age_seconds: u64,
    /// Flat fee to cover the keeper's expenses
    pub keeper_fee: Decimal,
    /// Maximum allowed number of positions per account
    pub positions_max: u64,
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
    /// Maximum fee that can be charged
    pub fee_max: Decimal,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            max_price_age_seconds: 60,
            keeper_fee: Decimal::from_str("0.01").unwrap(),
            positions_max: 100,
            skew_ratio_cap: Decimal::from_str("0.1").unwrap(),
            adl_offset: Decimal::from_str("0.1").unwrap(),
            adl_a: Decimal::from_str("0.1").unwrap(),
            adl_b: Decimal::from_str("0.1").unwrap(),
            fee_liquidity: Decimal::from_str("0.01").unwrap(),
            fee_max: Decimal::from_str("0.01").unwrap(),
        }
    }
}

impl ExchangeConfig {
    pub fn validate(&self) {
        assert!(self.max_price_age_seconds > 0, "Invalid max price age");
        assert!(self.keeper_fee >= dec!(0), "Invalid keeper fee");
        assert!(self.positions_max > 0, "Invalid max positions");
        assert!(self.skew_ratio_cap >= dec!(0), "Invalid skew ratio cap");
        assert!(self.adl_offset >= dec!(0), "Invalid adl offset");
        assert!(self.adl_a >= dec!(0), "Invalid adl a");
        assert!(self.adl_b >= dec!(0), "Invalid adl b");
        assert!(self.fee_liquidity >= dec!(0), "Invalid liquidity fee");
        assert!(self.fee_max >= dec!(0), "Invalid max fee");
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct PairConfig {
    /// Price feed id
    pub pair_id: u64,
    /// If the pair is disabled  // TODO: implement
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
}

#[derive(ScryptoSbor, Clone)]
pub struct CollateralConfig {
    /// Price feed id
    pub pair_id: u64,
    /// Discount applied to the collateral
    pub discount: Decimal,
}

impl CollateralConfig {
    pub fn validate(&self) {
        assert!(self.discount >= dec!(0), "Invalid discount");
    }
}
