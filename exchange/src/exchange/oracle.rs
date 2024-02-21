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

        pub fn get_price(&self, _pair_id: u64) -> Decimal {
            // for testing purposes, just return a constant price
            dec!(1)
        }

        pub fn get_price_resource(&self, _: ResourceAddress) -> Decimal {
            dec!(1)
        }
    }
}
