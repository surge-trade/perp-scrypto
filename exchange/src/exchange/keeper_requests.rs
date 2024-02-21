use scrypto::prelude::*;

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
    pub pair_id: u64,
    pub amount: Decimal,
    pub margin: Decimal,
    pub collateral_resource: ResourceAddress,
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
        pair_id: u64, 
        amount: Decimal, 
        margin: Decimal, 
        collateral_resource: ResourceAddress,
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::MarginOrder(RequestMarginOrder {
                pair_id,
                amount,
                margin,
                collateral_resource,
                price_limit,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn remove_collateral(
        resource: ResourceAddress, 
        amount: Decimal, 
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::RemoveCollateral(RequestRemoveCollateral {
                resource,
                amount,
            }),
            duration,
            status: Status::Pending,
        }
    }
}
