use scrypto::prelude::*;
use common::{PairId, ListIndex};
use super::errors::*;
use super::exchange_mod::Oracle;

pub struct VirtualOracle {
    prices: HashMap<PairId, Decimal>,
    resource_feeds: HashMap<ResourceAddress, PairId>,
}

impl VirtualOracle {
    pub fn new(oracle: Global<Oracle>, resource_feeds: HashMap<ResourceAddress, PairId>, mut pair_ids: HashSet<PairId>, max_age: Instant, updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>) -> Self {
        pair_ids.extend(resource_feeds.values().cloned());
        let prices_with_timestamp = if let Some((update_data, update_signature, key_id)) = updates {
            oracle.push_and_get_prices_with_auth(pair_ids, update_data, update_signature, key_id)
        } else {
            oracle.get_prices_with_auth(pair_ids)
        };

        let prices = prices_with_timestamp.into_iter().map(|(pair_id, (price, timestamp))| {
            assert!(
                timestamp.compare(max_age, TimeComparisonOperator::Gt),
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_PRICE_TOO_OLD, timestamp.seconds_since_unix_epoch, max_age.seconds_since_unix_epoch
            );
            assert!(
                price.is_positive(), 
                "{}, VALUE:{}, REQUIRED:0, OP:>", ERROR_INVALID_PRICE, price
            );
            (pair_id, price)
        }).collect();

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
