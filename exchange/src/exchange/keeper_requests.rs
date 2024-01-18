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
pub struct RequestAddLiquidity {
    pub resource_in: ResourceAddress,
    pub amount_in: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestAddLiquidityAsCollateral {
    pub resource_in: ResourceAddress,
    pub amount_in: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRemoveLiquidity {
    pub amount_lp: Decimal,
    pub resource_out: ResourceAddress,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRemoveCollateral {
    pub amount_lp: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRemoveCollateralAsToken {
    pub amount_lp: Decimal,
    pub resource_out: ResourceAddress,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestSwapOrder {
    pub resource_in: ResourceAddress,
    pub amount_in: Decimal,
    pub resource_out: ResourceAddress,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestMarginOrder {
    pub resource_0: ResourceAddress,
    pub amount_0: Decimal,
    pub resource_1: ResourceAddress,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestClosePosition {
    pub resource_0: ResourceAddress,
    pub resource_1: ResourceAddress,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub struct RequestRestrike {
    pub resource_restrike: ResourceAddress,
    pub amount_rebase: Decimal,
    pub resource_opposing: ResourceAddress,
    pub price_limit: Limit,
}

#[derive(ScryptoSbor, Clone)]
pub enum Request {
    AddLiquidity(RequestAddLiquidity),
    AddLiquidityAsCollateral(RequestAddLiquidityAsCollateral),
    RemoveLiquidity(RequestRemoveLiquidity),
    RemoveCollateral(RequestRemoveCollateral),
    RemoveCollateralAsToken(RequestRemoveCollateralAsToken),
    SwapOrder(RequestSwapOrder),
    MarginOrder(RequestMarginOrder),
    ClosePosition(RequestClosePosition),
    Restrike(RequestRestrike),
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
    pub fn add_liquidity(
        resource_in: ResourceAddress, 
        amount_in: Decimal,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::AddLiquidity(RequestAddLiquidity {
                resource_in,
                amount_in,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn add_liquidity_as_collateral(
        resource_in: ResourceAddress, 
        amount_in: Decimal,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::AddLiquidityAsCollateral(RequestAddLiquidityAsCollateral {
                resource_in,
                amount_in,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn remove_liquidity(
        amount_lp: Decimal, 
        resource_out: ResourceAddress,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::RemoveLiquidity(RequestRemoveLiquidity {
                amount_lp,
                resource_out,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn remove_collateral(
        amount_lp: Decimal,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::RemoveCollateral(RequestRemoveCollateral {
                amount_lp,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn remove_collateral_as_token(
        amount_lp: Decimal, 
        resource_out: ResourceAddress,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::RemoveCollateralAsToken(RequestRemoveCollateralAsToken {
                amount_lp,
                resource_out,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn swap_order(
        resource_in: ResourceAddress,
        amount_in: Decimal,
        resource_out: ResourceAddress,
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::SwapOrder(RequestSwapOrder {
                resource_in,
                amount_in,
                resource_out,
                price_limit,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn margin_order(
        resource_0: ResourceAddress,
        amount_0: Decimal,
        resource_1: ResourceAddress,
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::MarginOrder(RequestMarginOrder {
                resource_0,
                amount_0,
                resource_1,
                price_limit,
            }),
            duration,
            status: Status::Pending,
        }
    }

    pub fn close_position(
        resource_0: ResourceAddress,
        resource_1: ResourceAddress,
        price_limit: Limit,
        duration: Duration,
    ) -> Self {
        KeeperRequest {
            request: Request::ClosePosition(RequestClosePosition {
                resource_0,
                resource_1,
                price_limit,
            }),
            duration,
            status: Status::Pending,
        }
    }
}
