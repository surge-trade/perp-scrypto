use scrypto::prelude::*;
use common::{PairId, ListIndex};

#[derive(ScryptoSbor, Clone, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub cost: Decimal,
    pub funding_index: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountInfo {
    pub positions: HashMap<PairId, AccountPosition>,
    pub collateral_balances: HashMap<ResourceAddress, Decimal>,
    pub virtual_balance: Decimal,
    pub requests_len: ListIndex,
    pub valid_requests_start: ListIndex,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountUpdates {
    pub position_updates: HashMap<PairId, AccountPosition>,
    pub virtual_balance: Decimal,
    pub valid_requests_start: ListIndex,
    pub requests_new: Vec<KeeperRequest>,
    pub request_updates: HashMap<ListIndex, KeeperRequest>,
}

pub type Status = u8;

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    pub request: Vec<u8>,
    pub submission: Instant,
    pub expiry: Instant,
    pub status: Status,
}
