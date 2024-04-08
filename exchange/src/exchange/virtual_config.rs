use scrypto::prelude::*;
use config::*;
use common::PairId;
use super::errors::*;
use super::exchange::Config;

pub struct VirtualConfig {
    config: Global<Config>,
    config_info: ConfigInfo,
    pair_configs: HashMap<PairId, PairConfig>,
}

impl VirtualConfig {
    pub fn new(config: Global<Config>) -> Self {
        let config_info = config.get_info();

        Self {
            config,
            config_info: config_info.decompress(),
            pair_configs: HashMap::new(),
        }
    }

    pub fn load_pair_configs(&mut self, pair_ids: HashSet<PairId>) {
        self.pair_configs = self.config.get_pair_configs(pair_ids).into_iter()
            .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG).decompress())).collect();
    }

    pub fn exchange_config(&self) -> &ExchangeConfig {
        &self.config_info.exchange
    }

    pub fn pair_config(&self, pair_id: PairId) -> &PairConfig {
        match self.pair_configs.get(&pair_id) {
            Some(config) => config,
            _ => panic!("{}", ERROR_MISSING_PAIR_CONFIG),
        }
    }
    
    pub fn collateral_config(&self, resource: ResourceAddress) -> &CollateralConfig {
        self.config_info.collaterals.get(&resource)
            .expect(ERROR_COLLATERAL_INVALID)
    }

    pub fn collateral_configs(&self) -> &HashMap<ResourceAddress, CollateralConfig> {
        &self.config_info.collaterals
    }

    pub fn collaterals(&self) -> Vec<ResourceAddress> {
        self.config_info.collaterals.keys().cloned().collect()
    }

    pub fn collateral_feeds(&self) -> HashMap<ResourceAddress, PairId> {
        self.config_info.collaterals.iter().map(|(resource, config)| (*resource, config.pair_id)).collect()
    }
}
