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
            get_pair_config_len => PUBLIC;
            get_pair_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_pair_config_range => PUBLIC;

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

        pub fn get_pair_config_len(&self) -> ListIndex {
            self.pairs.len()
        }

        pub fn get_pair_config(&self, pair_id: PairId) -> Option<PairConfigCompressed> {
            self.pairs.get(&pair_id).map(|v| v.clone())
        }

        pub fn get_pair_configs(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PairConfigCompressed>> {
            pair_ids.into_iter().map(|k| (k.to_owned(), self.pairs.get(&k).map(|v| v.clone()))).collect()
        }

        pub fn get_pair_config_range(&self, start: ListIndex, end: ListIndex) -> Vec<PairConfigCompressed> {
            self.pairs.range(start, end)
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

