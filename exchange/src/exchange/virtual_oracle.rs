use scrypto::prelude::*;
use common::PairId;
use super::errors::*;
use super::exchange_mod::Oracle;

pub struct VirtualOracle {
    prices: HashMap<PairId, Decimal>,
    resource_feeds: HashMap<ResourceAddress, PairId>,
}

impl VirtualOracle {
    pub fn new(oracle: Global<Oracle>, resource_feeds: HashMap<ResourceAddress, PairId>, mut pair_ids: HashSet<PairId>, max_age: Instant, updates: Option<(Vec<u8>, Bls12381G2Signature)>) -> Self {
        if let Some((update_data, update_signature)) = updates {
            pair_ids.extend(resource_feeds.values().cloned());
            let prices = oracle.push_and_get_prices(pair_ids, max_age, update_data, update_signature);

            Self {
                prices,
                resource_feeds,
            }
        } else {
            let prices = oracle.get_prices(pair_ids, max_age);

            Self {
                prices,
                resource_feeds,
            }
        }
    }

    pub fn price(&self, pair_id: &PairId) -> Decimal {
        *self.prices.get(pair_id).expect(ERROR_MISSING_PRICE)
    }

    pub fn price_resource(&self, resource: ResourceAddress) -> Decimal {
        let pair_id = self.resource_feeds.get(&resource).expect(ERROR_MISSING_RESOURCE_FEED);
        self.price(pair_id)
    }
}
