pub mod config;
pub mod keeper_request;
pub mod liquidity_pool;
pub mod margin_account;
pub mod oracle;

use scrypto::prelude::*;
use crate::utils::{List, Vaults};
use self::config::Config;
use self::keeper_request::*;
use self::liquidity_pool::LiquidityPool;
use self::margin_account::{MarginAccount, MarginAccountManager};
use self::oracle::oracle::Oracle;

#[blueprint]
mod exchange {
    struct Exchange {
        config: Config,
        pool: LiquidityPool,
        accounts: MarginAccountManager,
        oracle: Global<Oracle>,
    }
    impl Exchange {
        pub fn new() -> Global<Exchange> {
            // for testing purposes
            let owner_role = OwnerRole::None;
            let resources = vec![];

            let (address_reservation, this) = Runtime::allocate_component_address(Exchange::blueprint_id());
            Self {
                config: Config::default(),
                pool: LiquidityPool::new(resources.clone(), this, owner_role.clone()),
                accounts: MarginAccountManager::new(this, owner_role.clone()),
                oracle: Oracle::new(resources.clone()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role.clone())
            .with_address(address_reservation)
            .globalize()
        }

        
    }
}
