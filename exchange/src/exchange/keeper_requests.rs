use scrypto::prelude::*;

use crate::utils::Pair;

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
    pub fn new(minutes: i64) -> Self {
        let now = Clock::current_time_rounded_to_minutes();
        Self { 
            end: now.add_minutes(minutes).expect("Duration too long"),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Clock::current_time_rounded_to_minutes();
        now.compare(self.end, TimeComparisonOperator::Gt)
    }
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestMarginOrder {
    pub pair: Pair,
    pub amount: Decimal,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub enum Request {
    MarginOrder(RequestMarginOrder),
}

#[derive(ScryptoSbor, Clone)]
pub enum Status {
    Pending,
    Completed,
    Cancelled,
}

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    pub request: Request,
    pub duration: Duration,
    pub status: Status,
}

impl KeeperRequest {
    pub fn margin_order(
        pair: Pair,
        amount: Decimal,
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::MarginOrder(RequestMarginOrder {
                pair,
                amount,
                price_limit,
            }),
            duration,
            status: Status::Pending,
        }
    }
}
