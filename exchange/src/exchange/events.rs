use scrypto::prelude::*;
use account::KeeperRequest;
use utils::{PairId, ListIndex};
use pool::PoolPosition;
use super::config::*;

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

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventExchangeConfigUpdate {
    pub config: ExchangeConfig,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventPairConfigUpdates {
    pub configs: Vec<PairConfig>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventCollateralConfigUpdates {
    pub configs: Vec<(ResourceAddress, CollateralConfig)>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct EventCollateralConfigRemoval {
    pub resource: ResourceAddress,
}
