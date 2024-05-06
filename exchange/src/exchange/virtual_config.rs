use scrypto::prelude::*;
use config::*;
use common::PairId;
use super::errors::*;
use super::exchange::Config;

pub struct VirtualConfig {
    config: Global<Config>,
    config_info: ConfigInfoCompressed,
    exchange_config: Option<ExchangeConfig>,
    collateral_configs: Option<HashMap<ResourceAddress, CollateralConfig>>,
    pair_configs: Option<HashMap<PairId, PairConfig>>,
}

impl VirtualConfig {
    pub fn new(config: Global<Config>) -> Self {
        let config_info = config.get_info();

        Self {
            config,
            config_info,
            exchange_config: None,
            collateral_configs: None,
            pair_configs: None,
        }
    }

    pub fn load_exchange_config(&mut self) {
        self.exchange_config = Some(self.config_info.exchange.decompress());
    }

    pub fn load_collateral_configs(&mut self) {
        let collateral_configs = self.config_info.collaterals.iter()
            .map(|(k, v)| (*k, v.decompress())).collect();
        self.collateral_configs = Some(collateral_configs);
    }

    pub fn load_pair_configs(&mut self, pair_ids: HashSet<PairId>) {
        let pair_configs = self.config.get_pair_configs(pair_ids).into_iter()
            .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG).decompress())).collect();
        self.pair_configs = Some(pair_configs);
    }

    pub fn exchange_config(&self) -> &ExchangeConfig {
        self.exchange_config.as_ref().expect(ERROR_EXCHANGE_CONFIG_NOT_LOADED)
    }

    pub fn pair_config(&self, pair_id: &PairId) -> &PairConfig {
        match self.pair_configs.as_ref().expect(ERROR_PAIR_CONFIGS_NOT_LOADED).get(pair_id) {
            Some(ref config) => config,
            None => panic!("{}", ERROR_MISSING_PAIR_CONFIG),
        }
    }

    pub fn collateral_configs(&self) -> &HashMap<ResourceAddress, CollateralConfig> {
        self.collateral_configs.as_ref().expect(ERROR_COLLATERAL_CONFIGS_NOT_LOADED)
    }

    pub fn collaterals(&self) -> Vec<ResourceAddress> {
        self.config_info.collaterals.keys().cloned().collect()
    }

    pub fn collateral_feeds(&self) -> HashMap<ResourceAddress, PairId> {
        self.config_info.collaterals.iter().map(|(resource, config)| (*resource, config.pair_id.clone())).collect()
    }
}
