use scrypto::prelude::*;
use common::{PairId, ListIndex};
use super::errors::*;
use super::exchange_mod::Oracle;

pub struct VirtualOracle {
    prices: HashMap<PairId, Decimal>,
    resource_map: HashMap<ResourceAddress, PairId>,
}

impl VirtualOracle {
    pub fn new(oracle: Global<Oracle>, resource_feeds: HashMap<ResourceAddress, (PairId, i64)>, mut pair_feeds: HashMap<PairId, i64>, updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>) -> Self {
        for (_, (pair_id, resource_max_age)) in resource_feeds.iter() {
            pair_feeds.entry(pair_id.clone()).and_modify(|max_age| {
                if *resource_max_age < *max_age {
                    *max_age = *resource_max_age;
                }
            }).or_insert(*resource_max_age);
        }
        
        let pair_ids: HashSet<PairId> = pair_feeds.keys().cloned().collect();
        let prices_with_timestamp = if let Some((update_data, update_signature, key_id)) = updates {
            oracle.push_and_get_prices_with_auth(pair_ids, update_data, update_signature, key_id)
        } else {
            oracle.get_prices_with_auth(pair_ids)
        };

        let current_time = Clock::current_time_rounded_to_seconds();
        let prices = prices_with_timestamp.into_iter().map(|(pair_id, (price, timestamp))| {
            let max_age = pair_feeds.get(&pair_id).expect(ERROR_INVALID_PRICE);
            let min_timestamp = current_time.add_seconds(-max_age).expect(ERROR_ARITHMETIC);

            assert!(
                timestamp.compare(min_timestamp, TimeComparisonOperator::Gt),
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_PRICE_TOO_OLD, timestamp.seconds_since_unix_epoch, min_timestamp.seconds_since_unix_epoch
            );
            assert!(
                price.is_positive(), 
                "{}, VALUE:{}, REQUIRED:0, OP:>", ERROR_INVALID_PRICE, price
            );
            (pair_id, price)
        }).collect();

        let resource_map = resource_feeds.into_iter().map(|(resource, (pair_id, _))| (resource, pair_id)).collect();
        Self {
            prices,
            resource_map,
        }
    }

    pub fn price(&self, pair_id: &PairId) -> Decimal {
        *self.prices.get(pair_id).expect(ERROR_MISSING_PRICE)
    }

    pub fn price_resource(&self, resource: ResourceAddress) -> Decimal {
        let pair_id = self.resource_map.get(&resource).expect(ERROR_MISSING_RESOURCE_FEED);
        self.price(pair_id)
    }
}
