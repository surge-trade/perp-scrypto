use scrypto::prelude::*;
use common::{PairId, ListIndex};

#[derive(ScryptoSbor, Clone, Debug, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub cost: Decimal,
    pub funding_index: Decimal,
}

impl AccountPosition {
    pub fn remove(&mut self) {
        self.amount = dec!(0);
        self.cost = dec!(0);
        self.funding_index = dec!(0);
    }
}

#[derive(ScryptoSbor)]
pub struct MarginAccountInfo {
    pub positions: HashMap<PairId, AccountPosition>,
    pub collateral_balances: HashMap<ResourceAddress, Decimal>,
    pub virtual_balance: Decimal,
    pub requests_len: ListIndex,
    pub active_requests_len: usize,
    pub valid_requests_start: ListIndex,
    pub referral_id: Option<NonFungibleLocalId>,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountUpdates {
    pub position_updates: HashMap<PairId, AccountPosition>,
    pub virtual_balance: Decimal,
    pub valid_requests_start: ListIndex,
    pub request_additions: Vec<KeeperRequest>,
    pub request_updates: HashMap<ListIndex, KeeperRequest>,
    pub active_request_additions: Vec<ListIndex>,
    pub active_request_removals: Vec<ListIndex>,
}

pub type Status = u8;

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    pub request: Vec<u8>,
    pub submission: Instant,
    pub expiry: Instant,
    pub status: Status,
    pub effected_components: Vec<ComponentAddress>,
}
