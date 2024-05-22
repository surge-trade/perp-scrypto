mod structs;

use scrypto::prelude::*;
use common::{PairId, ListIndex, HashList, _AUTHORITY_RESOURCE};
pub use self::structs::*;

#[blueprint]
#[types(
    PairId,
    ListIndex,
    PairConfigCompressed,
)]
mod config_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth! { 
        roles {
            authority => updatable_by: [];
        },
        methods { 
            // Get methods
            get_info => PUBLIC;
            get_pair_configs => PUBLIC;
            get_pair_configs_by_ids => PUBLIC;
            get_pair_configs_len => PUBLIC;

            // Authority protected methods
            update_exchange_config => restrict_to: [authority];
            update_pair_configs => restrict_to: [authority];
            update_collateral_configs => restrict_to: [authority];
            remove_collateral_config => restrict_to: [authority];
        }
    }

    struct Config {
        pub exchange: ExchangeConfigCompressed,
        pub pairs: HashList<PairId, PairConfigCompressed>,
        pub collaterals: HashMap<ResourceAddress, CollateralConfigCompressed>,
    }

    impl Config {
        pub fn new(owner_role: OwnerRole) -> Global<Config> {
            let exchange = ExchangeConfig::default().compress();
            let pairs = HashList::new(ConfigKeyValueStore::new_with_registered_type, ConfigKeyValueStore::new_with_registered_type);
            let collaterals = HashMap::new();

            Self {
                exchange,
                pairs,
                collaterals,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_info(&self) -> ConfigInfoCompressed {
            let exchange = self.exchange.clone();
            let collaterals = self.collaterals.iter().map(|(k, v)| (*k, v.clone())).collect();

            ConfigInfoCompressed {
                exchange,
                collaterals,
            }
        }

        pub fn get_pair_configs(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<PairConfigCompressed> {
            let start = start.unwrap_or(0);
            let end = (start + n).min(self.pairs.len());
            self.pairs.range(start, end)
        }

        pub fn get_pair_configs_by_ids(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PairConfigCompressed>> {
            pair_ids.into_iter().map(|k| {
                let value = self.pairs.get(&k).map(|v| v.clone());
                (k, value)
            }).collect()
        }

        pub fn get_pair_configs_len(&self) -> ListIndex {
            self.pairs.len()
        }

        pub fn update_exchange_config(&mut self, config: ExchangeConfig) {
            config.validate();
            self.exchange = config.compress();
        }

        pub fn update_pair_configs(&mut self, configs: Vec<PairConfig>) {
            for config in configs.into_iter() {
                config.validate();
                self.pairs.insert(config.pair_id.to_owned(), config.compress());
            }
        }

        pub fn update_collateral_configs(&mut self, configs: Vec<(ResourceAddress, CollateralConfig)>) {
            for (_, config) in configs.iter() {
                config.validate();
            }
            self.collaterals.extend(configs.into_iter().map(|(k, v)| (k, v.compress())));
        }

        pub fn remove_collateral_config(&mut self, resource: ResourceAddress) {
            self.collaterals.remove(&resource);
        }
    }

}

