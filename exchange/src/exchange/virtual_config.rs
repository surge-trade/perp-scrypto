use scrypto::prelude::*;
use config::*;
use common::PairId;
use super::errors::*;
use super::exchange_mod::Config;

pub struct VirtualConfig {
    exchange_config: ExchangeConfig,
    pair_configs: HashMap<PairId, PairConfig>,
    collateral_configs: HashMap<ResourceAddress, CollateralConfig>,
}

impl VirtualConfig {
    pub fn new(config: Global<Config>, pair_ids: HashSet<PairId>) -> Self {
        let config_info = config.get_info(pair_ids).decompress();
        let exchange_config = config_info.exchange;
        let pair_configs = config_info.pair_configs.into_iter()
            .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG))).collect();
        let collateral_configs = config_info.collaterals;

        Self {
            exchange_config,
            pair_configs,
            collateral_configs,
        }
    }

    pub fn exchange_config(&self) -> &ExchangeConfig {
        &self.exchange_config
    }

    pub fn pair_config(&self, pair_id: &PairId) -> &PairConfig {
        match self.pair_configs.get(pair_id) {
            Some(ref config) => config,
            None => panic!("{}", ERROR_MISSING_PAIR_CONFIG),
        }
    }

    pub fn collateral_configs(&self) -> &HashMap<ResourceAddress, CollateralConfig> {
        &self.collateral_configs
    }

    pub fn collateral_feeds(&self) -> HashMap<ResourceAddress, PairId> {
        self.collateral_configs.iter().map(|(resource, config)| (*resource, config.pair_id.clone())).collect()
    }
}
