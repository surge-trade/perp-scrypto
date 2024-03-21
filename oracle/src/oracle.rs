use scrypto::prelude::*;
use utils::PairId;

// THIS IS A MOCK IMPLEMENTATION
#[blueprint]
mod oracle {
    struct Oracle {
        prices: HashMap<PairId, Decimal>,
    }

    impl Oracle {
        pub fn new(owner_role: OwnerRole) -> Global<Oracle> {    
            let mut prices = HashMap::new();
                prices.insert(1, dec!(100));
                prices.insert(2, dec!(200));
                prices.insert(3, dec!(300));

            Self {
                prices
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn prices(&self, _max_age: Instant) -> HashMap<PairId, Decimal> {
            self.prices.clone()
        }
    }
}