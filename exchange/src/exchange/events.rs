use scrypto::prelude::*;
use account::KeeperRequest;
use utils::{PairId, ListIndex};
use pool::PoolPosition;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventSignalUpgrade {
    pub new_exchange: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventRequests {
    pub account: ComponentAddress,
    pub requests: Vec<(ListIndex, KeeperRequest)>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventPairUpdates {
    pub time: Instant,
    pub updates: Vec<(PairId, PoolPosition)>,
}
