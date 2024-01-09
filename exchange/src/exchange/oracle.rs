use scrypto::prelude::*;

#[blueprint]
mod oracle {
    struct Oracle {
        resources: Vec<ResourceAddress>,
    }

    impl Oracle {
        pub fn new(resources: Vec<ResourceAddress>) -> Global<Oracle> {
            Self {
                resources,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn get_prices(&self) -> HashMap<ResourceAddress, Decimal> {
            // for testing purposes, just return a constant price
            let price = dec!(1);

            let mut prices = HashMap::new();
            for resource in &self.resources {
                prices.insert(resource.clone(), price);
            }

            prices
        }
    }
}
