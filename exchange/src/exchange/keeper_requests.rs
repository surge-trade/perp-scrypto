use scrypto::prelude::*;
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
pub struct Duration {
    end: Instant,
}

impl Duration {
    pub fn new(minutes: u64) -> Self {
        let now = Clock::current_time_rounded_to_minutes();
        Self { 
            end: now.add_minutes(minutes as i64).expect("Duration too long"),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Clock::current_time_rounded_to_minutes();
        now.compare(self.end, TimeComparisonOperator::Gt)
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestMarginOrder {
    pub pair_id: u64,
    pub amount: Decimal,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRemoveCollateral {
    resource: ResourceAddress, 
    amount: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub enum Request {
    MarginOrder(RequestMarginOrder),
    RemoveCollateral(RequestRemoveCollateral),
}

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
    fn decode(data: &[u8]) -> Self;
}

impl Encodable for Request {
    fn encode(&self) -> Vec<u8> {
        // TODO: verify this is deterministic and will not change with different versions
        scrypto_encode(self).expect(ERROR_REQUEST_ENCODING)
    }

    fn decode(data: &[u8]) -> Self {
        scrypto_decode(data).expect(ERROR_REQUEST_DECODING)
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    data: Vec<u8>,
    duration: Duration,
    processed: bool,
}

impl KeeperRequest {
    pub fn is_active(&self) -> bool {
        !self.processed && !self.duration.is_expired()
    }

    pub fn process(&mut self) {
        self.processed = true;
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn margin_order(
        pair_id: u64, 
        amount: Decimal, 
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        let request = Request::MarginOrder(RequestMarginOrder {
            pair_id,
            amount,
            price_limit,
        });
        let data = request.encode();

        KeeperRequest {
            data,
            duration,
            processed: false,
        }
    }

    pub fn remove_collateral(
        resource: ResourceAddress, 
        amount: Decimal, 
        duration: Duration,
    ) -> Self {
        let request = Request::RemoveCollateral(RequestRemoveCollateral {
            resource,
            amount,
        });
        let data = request.encode();

        KeeperRequest {
            data,
            duration,
            processed: false,
        }
    }
}
