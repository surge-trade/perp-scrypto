use scrypto::prelude::*;
use account::KeeperRequest;
use super::errors::*;

#[derive(ScryptoSbor, Clone)]
pub enum Limit {
    Gte(Decimal),
    Lte(Decimal),
}

impl Limit {
    pub fn gte(value: Decimal) -> Self {
        Limit::Gte(value)
    }

    pub fn lte(value: Decimal) -> Self {
        Limit::Lte(value)
    }

    pub fn compare(&self, value: Decimal) -> bool {
        match self {
            Limit::Gte(limit) => value >= *limit,
            Limit::Lte(limit) => value <= *limit,
        }
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRemoveCollateral {
    pub target_account: ComponentAddress, 
    pub claims: Vec<(ResourceAddress, Decimal)>,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestMarginOrder {
    pub pair_id: u64,
    pub amount: Decimal,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub enum Request {
    RemoveCollateral(RequestRemoveCollateral),
    MarginOrder(RequestMarginOrder),
}

impl Request {
    pub fn encode(&self) -> Vec<u8> {
        // TODO: verify this is deterministic and will not change with different versions
        scrypto_encode(self).expect(ERROR_REQUEST_ENCODING)
    }

    pub fn decode(data: &[u8]) -> Self {
        scrypto_decode(data).expect(ERROR_REQUEST_DECODING)
    }
}
