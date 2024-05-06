use scrypto::prelude::*;
use config::*;
use common::PairId;
use super::errors::*;
use super::exchange_mod::Config;

pub struct VirtualConfig {
    config: Global<Config>,
    config_info: ConfigInfo,
    pair_configs: Option<HashMap<PairId, PairConfig>>,
}

impl VirtualConfig {
    pub fn new(config: Global<Config>) -> Self {
        let config_info = config.get_info().decompress();

        Self {
            config,
            config_info,
            pair_configs: None,
        }
    }

    pub fn load_pair_configs(&mut self, pair_ids: HashSet<PairId>) {
        let pair_configs = self.config.get_pair_configs(pair_ids).into_iter()
            .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG).decompress())).collect();
        self.pair_configs = Some(pair_configs);
    }

    pub fn exchange_config(&self) -> &ExchangeConfig {
        &self.config_info.exchange
    }

    pub fn pair_config(&self, pair_id: &PairId) -> &PairConfig {
        match self.pair_configs.as_ref().expect(ERROR_PAIR_CONFIGS_NOT_LOADED).get(pair_id) {
            Some(ref config) => config,
            None => panic!("{}", ERROR_MISSING_PAIR_CONFIG),
        }
    }

    pub fn collateral_configs(&self) -> &HashMap<ResourceAddress, CollateralConfig> {
        &self.config_info.collaterals
    }

    pub fn collaterals(&self) -> Vec<ResourceAddress> {
        self.config_info.collaterals.keys().cloned().collect()
    }

    pub fn collateral_feeds(&self) -> HashMap<ResourceAddress, PairId> {
        self.config_info.collaterals.iter().map(|(resource, config)| (*resource, config.pair_id.clone())).collect()
    }
}
