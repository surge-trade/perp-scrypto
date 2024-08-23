use scrypto::prelude::*;
use account::Status;
use common::{PairId, ListIndex, ListIndexOffset};
use super::errors::*;

#[derive(ScryptoSbor, ManifestSbor, Clone, Copy, Debug, Eq, PartialEq)]
pub enum PriceLimit {
    None,
    Gte(Decimal),
    Lte(Decimal),
}

impl PriceLimit {
    pub fn compare(&self, value: Decimal) -> bool {
        match self {
            PriceLimit::None => true,
            PriceLimit::Gte(limit) => value >= *limit,
            PriceLimit::Lte(limit) => value <= *limit,
        }
    }

    pub fn price(&self) -> Decimal {
        match self {
            PriceLimit::None => Decimal::ZERO,
            PriceLimit::Gte(limit) => *limit,
            PriceLimit::Lte(limit) => *limit,
        }
    }

    pub fn op(&self) -> &'static str {
        match self {
            PriceLimit::None => "True",
            PriceLimit::Gte(_) => ">=",
            PriceLimit::Lte(_) => "<=",
        }
    }
}

#[derive(ScryptoSbor, ManifestSbor, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlippageLimit {
    None,
    Percent(Decimal),
    Absolute(Decimal),
}

impl SlippageLimit {
    pub fn compare(&self, slippage: Decimal, amount: Decimal) -> bool {
        match self {
            SlippageLimit::None => true,
            SlippageLimit::Percent(limit) => slippage <= amount * *limit / dec!(100),
            SlippageLimit::Absolute(limit) => slippage <= *limit,
        }
    }

    pub fn allowed_slippage(&self, amount: Decimal) -> Decimal {
        match self {
            SlippageLimit::None => Decimal::MAX,
            SlippageLimit::Percent(limit) => amount * *limit / dec!(100),
            SlippageLimit::Absolute(limit) => *limit,
        }
    }
}

#[derive(ScryptoSbor, ManifestSbor, Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestIndexRef {
    Index(ListIndex),
    RelativeIndex(ListIndexOffset),
}

impl RequestIndexRef {
    pub fn resolve(&self, request_index: ListIndex) -> ListIndex {
        match self {
            RequestIndexRef::Index(index) => *index,
            RequestIndexRef::RelativeIndex(offset) => {
                let offset = ListIndex::try_from(*offset).expect(ERROR_ARITHMETIC);
                request_index.checked_add(offset).expect(ERROR_ARITHMETIC)
            }
        }
    }
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct RequestRemoveCollateral {
    pub target_account: ComponentAddress, 
    pub claims: Vec<(ResourceAddress, Decimal)>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct RequestMarginOrder {
    pub pair_id: PairId,
    pub amount: Decimal,
    pub reduce_only: bool,
    pub price_limit: PriceLimit,
    pub slippage_limit: SlippageLimit,
    pub activate_requests: Vec<ListIndex>,
    pub cancel_requests: Vec<ListIndex>,
}

#[derive(ScryptoSbor, Clone, Debug)]
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
pub const STATUS_EXPIRED: Status = 4;
