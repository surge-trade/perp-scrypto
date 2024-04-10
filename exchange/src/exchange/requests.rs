use scrypto::prelude::*;
use account::Status;
use common::{PairId, ListIndex};
use super::errors::*;

#[derive(ScryptoSbor, Clone)]
pub enum Limit {
    Gte(Decimal),
    Lte(Decimal),
}

impl Limit {
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
    pub pair_id: PairId,
    pub amount: Decimal,
    pub price_limit: Limit,
    pub activate_requests: Vec<ListIndex>,
    pub cancel_requests: Vec<ListIndex>,
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

pub const STATUS_DORMANT: Status = 0;
pub const STATUS_ACTIVE: Status = 1;
pub const STATUS_EXECUTED: Status = 2;
pub const STATUS_CANCELLED: Status = 3;
