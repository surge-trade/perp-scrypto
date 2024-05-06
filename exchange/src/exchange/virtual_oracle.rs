use scrypto::prelude::*;
use common::PairId;
use super::errors::*;
use super::exchange::Oracle;

pub struct VirtualOracle {
    prices: HashMap<PairId, Decimal>,
    resource_feeds: HashMap<ResourceAddress, PairId>,
}

impl VirtualOracle {
    pub fn new(oracle: Global<Oracle>, resource_feeds: HashMap<ResourceAddress, PairId>, max_age: Instant) -> Self {
        let prices = oracle.prices(max_age);

        Self {
            prices,
            resource_feeds,
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
