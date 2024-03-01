use scrypto::prelude::*;
use super::errors::*;
use super::exchange::Oracle;

pub struct VirtualOracle {
    oracle: Global<Oracle>,
    prices: HashMap<u64, Decimal>,
    resource_feeds: HashMap<ResourceAddress, u64>,
}

impl VirtualOracle {
    pub fn new(oracle: ComponentAddress, resource_feeds: HashMap<ResourceAddress, u64>) -> Self {
        let oracle = Global::<Oracle>::try_from(oracle).expect(ERROR_INVALID_ORACLE);
        let prices = oracle.prices();

        Self {
            oracle,
            prices,
            resource_feeds,
        }
    }

    pub fn price(&self, pair_id: u64) -> Decimal {
        *self.prices.get(&pair_id).expect(ERROR_MISSING_PRICE)
    }

    pub fn price_resource(&self, resource: ResourceAddress) -> Decimal {
        let pair_id = *self.resource_feeds.get(&resource).expect(ERROR_MISSING_RESOURCE_FEED);
        self.price(pair_id)
    }
}
