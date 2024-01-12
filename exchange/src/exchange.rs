pub mod config;
pub mod keeper_request;
pub mod liquidity_pool;
pub mod margin_account;
pub mod oracle;

use scrypto::prelude::*;
use crate::utils::{List, Vaults};
use self::config::*;
use self::keeper_request::*;
use self::liquidity_pool::*;
use self::margin_account::*;
use self::oracle::oracle::Oracle;

#[derive(ScryptoSbor)]
pub struct PoolInfo {
    pub pool_value: Decimal, 
    pub lp_token_supply: Decimal, 
    pub lp_token_price: Decimal, 
    pub position_info_list: Vec<PoolPositionInfo>
}

#[derive(ScryptoSbor)]
pub struct PoolPositionInfo {
    pub resource: ResourceAddress,
    pub amount_borrowing: Decimal,
    pub amount_funding: Decimal,
    pub borrowing_long_rate: Decimal, 
    pub borrowing_short_rate: Decimal,
    pub funding_long_rate: Decimal,
    pub funding_short_rate: Decimal,
    pub borrowing_long_adjustment: Decimal,
    pub borrowing_short_adjustment: Decimal,
    pub funding_long_adjustment: Decimal,
    pub funding_short_adjustment: Decimal,
}

#[derive(ScryptoSbor)]
pub struct AccountInfo {
    pub positions_value: Decimal,
    pub collateral_value: Decimal,
    pub amount_interest: Decimal,
    pub position_info_list: Vec<AccountPositionInfo>,
}

#[derive(ScryptoSbor)]
pub struct AccountPositionInfo {
    pub resource: ResourceAddress,
    pub interest_adjustment: Decimal,
}

#[blueprint]
mod exchange {
    struct Exchange {
        config: ExchangeConfig,
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
                config: ExchangeConfig::default(),
                pool: LiquidityPool::new(resources.clone(), this, owner_role.clone()),
                accounts: MarginAccountManager::new(this, owner_role.clone()),
                oracle: Oracle::new(resources.clone()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role.clone())
            .with_address(address_reservation)
            .globalize()
        }

        pub fn get_resources(&self) -> Vec<ResourceAddress> {
            self.config.resource_configs.iter().map(|c| c.resource).collect()
        }

        pub fn get_pool_balances(&self) -> Vec<Decimal> {
            self.get_resources().iter().map(|r| self.pool.vaults.amount(r)).collect()
        }

        pub fn get_pool_info(&self, prices: HashMap<ResourceAddress, Decimal>) -> PoolInfo {
            let current_time = Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch;
            let last_update = self.pool.last_update.seconds_since_unix_epoch;
            let period_minutes = Decimal::from((current_time - last_update) / 60);

            let mut pool_value = dec!(0);
            let mut position_info_list = Vec::new();
            for resource_config in &self.config.resource_configs {
                let resource = &resource_config.resource;
                let price = *prices.get(&resource).expect("Missing price");
                let pool_position = self.pool.positions.get(&resource).expect("Missing pool position");
                
                let (borrowing_long_rate, borrowing_short_rate) = {
                    if pool_position.long_oi > pool_position.short_oi {
                        let long = self.config.borrowing_long_rate * self.config.borrowing_discount;
                        let short = self.config.borrowing_short_rate;
                        (long, short)
                    } else {
                        let long = self.config.borrowing_long_rate;
                        let short = self.config.borrowing_short_rate * self.config.borrowing_discount;
                        (long, short)
                    }
                };
                let borrowing_long_adjustment = borrowing_long_rate * period_minutes;
                let borrowing_short_adjustment = borrowing_short_rate * period_minutes;
                
                let amount_vault = self.pool.vaults.amount(&resource);
                let amount_oi_net = pool_position.long_oi - pool_position.short_oi;
                let amount_borrowing = borrowing_long_adjustment * pool_position.long_oi + borrowing_short_adjustment * pool_position.short_oi;

                let amount = amount_vault - amount_oi_net + amount_borrowing;
                let value = amount * price;
                pool_value += value;

                let (amount_funding, funding_long_rate, funding_short_rate) = if pool_position.long_oi.is_zero() || pool_position.short_oi.is_zero() {
                    (dec!(0), dec!(0), dec!(0))
                } else {
                    let amount_funding = self.config.funding_rate * amount_oi_net;
                    if pool_position.long_oi > pool_position.short_oi {
                        let long = amount_funding / pool_position.long_oi;
                        let short = -amount_funding / pool_position.short_oi;
                        (amount_funding, long, short)
                    } else {
                        let long = -amount_funding / pool_position.long_oi;
                        let short = amount_funding / pool_position.short_oi;
                        (amount_funding, long, short)
                    }
                };
                let funding_long_adjustment = funding_long_rate * period_minutes;
                let funding_short_adjustment = funding_short_rate * period_minutes;

                position_info_list.push(PoolPositionInfo {
                    resource: resource.clone(),
                    amount_borrowing,
                    amount_funding,
                    borrowing_long_rate,
                    borrowing_short_rate,
                    funding_long_rate,
                    funding_short_rate,
                    borrowing_long_adjustment,
                    borrowing_short_adjustment,
                    funding_long_adjustment,
                    funding_short_adjustment,
                });
            }
            pool_value += self.pool.unrealized_borrowing;

            let lp_token_supply = self.pool.lp_token_manager.total_supply().expect("Missing lp token supply");
            let lp_token_price = if lp_token_supply.is_zero() {
                dec!(0)
            } else {
                let lp_token_price = pool_value / lp_token_supply;

                for position_info in &mut position_info_list {
                    let price = *prices.get(&position_info.resource).expect("Missing price") / lp_token_price;
                    position_info.amount_borrowing *= price;
                    position_info.amount_funding *= price;
                    position_info.borrowing_long_adjustment *= price;
                    position_info.borrowing_short_adjustment *= price;
                    position_info.funding_long_adjustment *= price;
                    position_info.funding_short_adjustment *= price;
                }

                lp_token_price
            };

            PoolInfo {
                pool_value,
                lp_token_supply,
                lp_token_price,
                position_info_list,
            }
        }

        pub fn get_account_info(&self, account_id: NonFungibleLocalId, lp_token_price: Decimal, prices: HashMap<ResourceAddress, Decimal>) -> AccountInfo {
            let account = self.accounts.get(&account_id).expect("Missing account");

            let mut positions_value = dec!(0);
            let mut amount_interest = dec!(0);
            let mut position_info_list = Vec::new();
            for (resource, account_position) in &account.positions {
                let price = *prices.get(resource).expect("Missing price");
                let pool_position = self.pool.positions.get(resource).expect("Missing pool position");

                let interest_adjustment = if account_position.amount.is_positive() {
                    pool_position.interest_long_checkpoint - account_position.interest_checkpoint
                } else if account_position.amount.is_negative() {
                    pool_position.interest_short_checkpoint - account_position.interest_checkpoint
                } else {
                    dec!(0)
                };
                amount_interest += interest_adjustment * account_position.amount;

                let value = account_position.amount * price;
                positions_value += value;

                position_info_list.push(AccountPositionInfo {
                    resource: resource.clone(),
                    interest_adjustment,
                });
            }

            let collateral_value = account.collateral.amount() * lp_token_price;

            AccountInfo {
                positions_value,
                collateral_value,
                amount_interest,
                position_info_list,
            }
        }

        fn update_pool_interest(&mut self, pool_info: &PoolInfo) {
            let mut unrealized_borrowing = dec!(0);
            for position_info in &pool_info.position_info_list {
                let resource = &position_info.resource;
                let pool_position = self.pool.positions.get_mut(resource).expect("Missing pool position");
                pool_position.interest_long_checkpoint += position_info.borrowing_long_adjustment + position_info.funding_long_adjustment;
                pool_position.interest_short_checkpoint += position_info.borrowing_short_adjustment + position_info.funding_short_adjustment;
                unrealized_borrowing += position_info.amount_borrowing;
            }
            self.pool.unrealized_borrowing = unrealized_borrowing;
            self.pool.last_update = Clock::current_time_rounded_to_minutes();
        }

        fn update_account_interest(&mut self, account_id: NonFungibleLocalId, account_info: &AccountInfo) {
            let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

            for position_info in &account_info.position_info_list {
                let resource = &position_info.resource;
                let account_position = account.positions.get_mut(resource).expect("Missing account position");
                account_position.interest_checkpoint += position_info.interest_adjustment;
            }

            if account_info.amount_interest.is_negative() {
                let lp_tokens = self.pool.lp_token_manager.mint(-account_info.amount_interest);
                account.collateral.put(lp_tokens);
            } else {
                account.collateral.take(account_info.amount_interest).burn();
            }
        }

        fn add_liquidity(&mut self, account_id: NonFungibleLocalId, resource: ResourceAddress, amount: Decimal, prices: HashMap<ResourceAddress, Decimal>) {
            let pool_info = self.get_pool_info(prices.clone());
            self.update_pool_interest(&pool_info);

            let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

            let amount_available = account.vaults.amount(&resource);
            if amount > amount_available {
                panic!("Insufficient amount");
            }

            // TODO: calculate fee + price impact
            // let fee_impact = 
            let fee = self.config.swap_fee * amount;

            let price = *prices.get(&resource).expect("Missing price");
            let value = amount * price;
            let amount_mint = if pool_info.pool_value.is_zero() {
                value * pool_info.lp_token_supply / pool_info.pool_value * fee
            } else {
                value
            };

            let lp_tokens = self.pool.lp_token_manager.mint(amount_mint);
            let tokens = account.vaults.take(resource, amount);
            account.vaults.put(lp_tokens);
            self.pool.vaults.put(tokens);
        }


    }
}
