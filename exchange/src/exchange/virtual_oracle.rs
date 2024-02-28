use scrypto::prelude::*;
use super::errors::*;
use super::oracle::oracle::Oracle;

pub struct VirtualOracle {
    oracle: Global<Oracle>,
    prices: HashMap<u64, Decimal>,
}

impl VirtualOracle {
    pub fn new(oracle: ComponentAddress) -> Self {
        let oracle = Global::<Oracle>::try_from(oracle).expect(ERROR_INVALID_ORACLE);
        let prices = oracle.get_prices();

        Self {
            oracle,
            prices,
        }
    }

    pub fn price(&self, pair_id: u64) -> Decimal {
        *self.prices.get(&pair_id).expect(ERROR_MISSING_PRICE)
    }

    pub fn price_resource(&self, resource: ResourceAddress) -> Decimal {
        dec!(0) // TODO: implement
    }
}
