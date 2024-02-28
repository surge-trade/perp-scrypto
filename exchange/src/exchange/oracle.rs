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

        pub fn get_prices(&self) -> HashMap<u64, Decimal> {
            HashMap::new()
        }

        pub fn get_price_resource(&self, _: ResourceAddress) -> Option<Decimal> {
            Some(dec!(1))
        }
    }
}
