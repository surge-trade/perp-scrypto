// TODO: remove dead code
#![allow(dead_code)]

pub mod config;
pub mod consts;
pub mod keeper_requests;
pub mod liquidity_pool;
pub mod margin_account;
pub mod oracle;

use scrypto::prelude::*;
use crate::utils::*;
use self::config::*;
use self::consts::*;
use self::keeper_requests::*;
use self::liquidity_pool::*;
use self::margin_account::*;
use self::oracle::oracle::Oracle;

#[blueprint]
mod exchange {
    struct Exchange {
        config: ExchangeConfig,
        pool: LiquidityPool,
        accounts: MarginAccountManager,
        oracle: Global<Oracle>,
    }
    impl Exchange {
        pub fn new(base_resource: ResourceAddress) -> Global<Exchange> {
            // for testing purposes
            let owner_role = OwnerRole::None;
            let resources = vec![];

            let (address_reservation, this) = Runtime::allocate_component_address(Exchange::blueprint_id());
            Self {
                config: ExchangeConfig::default(),
                pool: LiquidityPool::new(this, owner_role.clone()),
                accounts: MarginAccountManager::new(this, owner_role.clone()),
                oracle: Oracle::new(resources.clone()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role.clone())
            .with_address(address_reservation)
            .globalize()
        }

        // update_pair
        
        // margin_order

        // add_collateral

        // remove_collateral

        // swap_debt

        // liquidate



        
    }
}
