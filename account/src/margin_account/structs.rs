use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub cost: Decimal,
    pub funding_index: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountInfo {
    pub positions: HashMap<u64, AccountPosition>,
    pub collateral_balances: HashMap<ResourceAddress, Decimal>,
    pub virtual_balance: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountUpdates {
    pub position_updates: HashMap<u64, AccountPosition>,
    pub virtual_balance: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    pub request: Vec<u8>,
    pub expiry: Instant,
    pub processed: bool,
}

impl KeeperRequest {
    pub fn new(request: Vec<u8>, expiry: Instant) -> Self {
        Self {
            request,
            expiry,
            processed: false,
        }
    }
}
