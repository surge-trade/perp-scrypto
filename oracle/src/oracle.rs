use scrypto::prelude::*;
use common::PairId;

// THIS IS A MOCK IMPLEMENTATION
#[blueprint]
mod oracle_mod {
    struct Oracle {
        prices: HashMap<PairId, Decimal>,
    }

    impl Oracle {
        pub fn new(owner_role: OwnerRole) -> Global<Oracle> {    
            let mut prices = HashMap::new();
                prices.insert(0, dec!(71107.81));
                prices.insert(1, dec!(3687.39));
                prices.insert(2, dec!(177.45));

            Self {
                prices
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn push_price(&mut self, pair_id: PairId, price: Decimal) {
            self.prices.insert(pair_id, price);
        }

        pub fn prices(&self, _max_age: Instant) -> HashMap<PairId, Decimal> {
            self.prices.clone()
        }
    }
}