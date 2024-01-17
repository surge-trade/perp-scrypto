// TODO: remove dead code
#![allow(dead_code)]

pub mod config;
pub mod keeper_requests;
pub mod liquidity_pool;
pub mod margin_account;
pub mod oracle;

use scrypto::prelude::*;
use crate::utils::*;
use self::config::*;
use self::keeper_requests::*;
use self::liquidity_pool::*;
use self::margin_account::*;
use self::oracle::oracle::Oracle;

#[derive(ScryptoSbor)]
pub struct PoolInfo {
    pub pool_value: Decimal, 
    pub lp_token_supply: Decimal, 
    pub lp_token_price: Decimal, 
    pub position_info_map: HashMap<ResourceAddress, PoolPositionInfo>
}

#[derive(ScryptoSbor, Clone)]
pub struct PoolPositionInfo {
    pub target_weight: Decimal,
    pub value: Decimal,
    pub amount_borrowing: Decimal,
    pub amount_funding: Decimal,
    pub borrowing_long_rate: Decimal, 
    pub borrowing_short_rate: Decimal,
    pub funding_long_rate: Decimal,
    pub funding_short_rate: Decimal,
    pub interest_long_checkpoint: Decimal,
    pub interest_short_checkpoint: Decimal,
}

#[derive(ScryptoSbor)]
pub struct AccountInfo {
    pub positions_value: Decimal,
    pub collateral_value: Decimal,
    pub liquidation_fee: Decimal,
    pub amount_interest: Decimal,
    pub position_info_map: HashMap<ResourceAddress, AccountPositionInfo>
}

#[derive(ScryptoSbor, Clone)]
pub struct AccountPositionInfo {
    pub value: Decimal,
    pub interest_checkpoint: Decimal,
}

fn calculate_imbalances(position_changes: HashMap<ResourceAddress, Decimal>, pool_info: &PoolInfo) -> (Decimal, Decimal) {
    let mut pool_imbalance_0 = dec!(0);
    let mut pool_imbalance_f = dec!(0);
    let pool_change = position_changes.values().fold(dec!(0), |a, b| a + *b);
    for (resource, pool_position_info) in pool_info.position_info_map.iter() {
        let position_change = *position_changes.get(resource).unwrap_or(&dec!(0));
        let position_imbalance_0 = pool_position_info.value - pool_info.pool_value * pool_position_info.target_weight;
        let position_imbalance_f = (pool_position_info.value + position_change) - (pool_info.pool_value + pool_change) * pool_position_info.target_weight;
        pool_imbalance_0 += position_imbalance_0.checked_abs().unwrap();
        pool_imbalance_f += position_imbalance_f.checked_abs().unwrap();
    }
    (pool_imbalance_0, pool_imbalance_f)
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

            let prices: Vec<Decimal> = self.config.resource_configs.iter().map(|c| *prices.get(&c.resource).expect("Missing price")).collect();

            let mut pool_value = dec!(0);
            let mut position_info_list = Vec::new();
            for (resource_config, &price) in self.config.resource_configs.iter().zip(prices.iter()) {
                let resource = &resource_config.resource;
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

                position_info_list.push((
                    resource.clone(),
                    PoolPositionInfo {
                        target_weight: resource_config.weight,
                        value: value,
                        amount_borrowing,
                        amount_funding,
                        borrowing_long_rate,
                        borrowing_short_rate,
                        funding_long_rate,
                        funding_short_rate,
                        interest_long_checkpoint: pool_position.interest_long_checkpoint,
                        interest_short_checkpoint: pool_position.interest_short_checkpoint,
                    },
                    (
                        borrowing_long_adjustment + funding_long_adjustment,
                        borrowing_short_adjustment + funding_short_adjustment,
                    )
                ));
            }
            pool_value += self.pool.unrealized_borrowing;

            let lp_token_supply = self.pool.lp_token_manager.total_supply().expect("Missing lp token supply");
            let lp_token_price = if lp_token_supply.is_zero() {
                dec!(0)
            } else {
                let lp_token_price = pool_value / lp_token_supply;

                for ((_, position_info, adjustments), &price) in position_info_list.iter_mut().zip(prices.iter()) {
                    let price =  price / lp_token_price;
                    position_info.amount_borrowing *= price;
                    position_info.amount_funding *= price;
                    adjustments.0 *= price;
                    adjustments.1 *= price;
                }

                lp_token_price
            };

            for (_, position_info, adjustments) in position_info_list.iter_mut() {
                position_info.interest_long_checkpoint += adjustments.0;
                position_info.interest_short_checkpoint += adjustments.1;
            }

            PoolInfo {
                pool_value,
                lp_token_supply,
                lp_token_price,
                position_info_map: position_info_list.into_iter().map(|(r, p, _)| (r , p)).collect(),
            }
        }

        pub fn get_info(&self, account_id: NonFungibleLocalId, prices: HashMap<ResourceAddress, Decimal>) -> (AccountInfo, PoolInfo) {
            let pool_info = self.get_pool_info(prices.clone());
            let account = self.accounts.get(&account_id).expect("Missing account");

            let mut positions_value = dec!(0);
            let mut amount_interest = dec!(0);
            let mut position_info_list = Vec::new();
            for (resource, account_position) in account.positions.iter() {
                let price = *prices.get(resource).expect("Missing price");
                let pool_position_info = pool_info.position_info_map.get(resource).expect("Missing pool position info");

                let interest_checkpoint = if account_position.amount.is_positive() {
                    pool_position_info.interest_long_checkpoint
                } else {
                    pool_position_info.interest_short_checkpoint
                };
                let interest_adjustment = interest_checkpoint - account_position.interest_checkpoint;
                amount_interest += interest_adjustment * account_position.amount.checked_abs().unwrap();

                let value = account_position.amount * price;
                positions_value += value;

                position_info_list.push((
                    resource.clone(),
                    AccountPositionInfo {
                        value,
                        interest_checkpoint,
                    }
                ));
            }

            let liquidation_changes: HashMap<ResourceAddress, Decimal> = position_info_list.iter().map(|(r, p)| (r.clone(), p.value)).collect();
            let (imbalance_0, imbalance_f) = calculate_imbalances(liquidation_changes.clone(), &pool_info);
            let fee_base = liquidation_changes.values().fold(dec!(0), |a, b| a + b.checked_abs().unwrap()) * self.config.margin_base_fee;
            let fee_impact = (imbalance_f.pow(self.config.margin_impact_exp) - imbalance_0.pow(self.config.margin_impact_exp)) * self.config.margin_impact_fee;
            let liquidation_fee = (fee_base + fee_impact).max(dec!(0));

            let account_info = AccountInfo {
                positions_value,
                collateral_value: account.collateral.amount() * pool_info.lp_token_price,
                liquidation_fee,
                amount_interest,
                position_info_map: position_info_list.into_iter().collect(),
            };

            (account_info, pool_info)
        }

        pub fn assert_pool_integrity(&self) {
            for resource_config in self.config.resource_configs.iter() {
                let pool_position = self.pool.positions.get(&resource_config.resource).expect("Missing pool position");
                let amount_tokens = self.pool.vaults.amount(&resource_config.resource);

                assert!(
                    pool_position.long_oi < amount_tokens * resource_config.max_oi_long_factor, 
                    "Pool long OI too high for {}",
                    Runtime::bech32_encode_address(resource_config.resource)
                );
                assert!(
                    pool_position.long_oi + pool_position.short_oi < amount_tokens * resource_config.max_oi_net_factor, 
                    "Pool net OI too high for {}",
                    Runtime::bech32_encode_address(resource_config.resource)
                );
            }
        }

        fn update_pool_interest(&mut self, pool_info: &PoolInfo) {
            let mut unrealized_borrowing = dec!(0);
            for (resource, position_info) in pool_info.position_info_map.iter() {
                let pool_position = self.pool.positions.get_mut(resource).expect("Missing pool position");
                pool_position.interest_long_checkpoint = position_info.interest_long_checkpoint;
                pool_position.interest_short_checkpoint = position_info.interest_short_checkpoint;
                unrealized_borrowing += position_info.amount_borrowing;
            }
            self.pool.unrealized_borrowing = unrealized_borrowing;
            self.pool.last_update = Clock::current_time_rounded_to_minutes();
        }

        fn update_account_interest(&mut self, account_id: NonFungibleLocalId, account_info: &AccountInfo) {
            let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

            for (resource, position_info) in account_info.position_info_map.iter() {
                let account_position = account.positions.get_mut(resource).expect("Missing account position");
                account_position.interest_checkpoint = position_info.interest_checkpoint;
            }

            if account_info.amount_interest.is_negative() {
                let lp_tokens = self.pool.lp_token_manager.mint(-account_info.amount_interest);
                account.collateral.put(lp_tokens);
            } else {
                account.collateral.take(account_info.amount_interest).burn();
            }
        }

        fn liquidate_account(&mut self, account_id: NonFungibleLocalId, prices: HashMap<ResourceAddress, Decimal>) {
            let (account_info, pool_info) = self.get_info(account_id.clone(), prices.clone());
            self.update_pool_interest(&pool_info);
            self.update_account_interest(account_id.clone(), &account_info);
            {
                let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

                let positions_liquidation_value = account_info.positions_value - account_info.liquidation_fee;
                let account_value = positions_liquidation_value + account_info.collateral_value;

                assert!(
                    account_value / account_info.collateral_value < self.config.min_collateral_ratio,
                    "Sufficient collateral ratio"
                );

                for (resource, account_position) in account.positions.iter_mut() {
                    let pool_position = self.pool.positions.get_mut(resource).expect("Missing pool position");
                    if account_position.amount.is_positive() {
                        pool_position.long_oi -= account_position.amount;
                    } else {
                        pool_position.short_oi += account_position.amount;
                    }
                    account_position.amount = dec!(0);
                }

                let liquidation_amount = -positions_liquidation_value / pool_info.lp_token_price;
                if account_value.is_negative() {
                    // TODO: bad debt
                    let bad_debt = -account_value;
                    info!("Bad debt: {}", bad_debt);
                    account.collateral.take(liquidation_amount).burn();
                } else {
                    account.collateral.take(liquidation_amount).burn();
                }
            }
        }


        fn add_liquidity(&mut self, account_id: NonFungibleLocalId, resource: ResourceAddress, amount: Decimal, prices: HashMap<ResourceAddress, Decimal>) -> Decimal {
            let pool_info = self.get_pool_info(prices.clone());
            self.update_pool_interest(&pool_info);

            let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

            if amount > account.vaults.amount(&resource) {
                panic!("Insufficient balance in account");
            }

            let price = *prices.get(&resource).expect("Missing price");
            let value = amount * price;
            let (imbalance_0, imbalance_f) = calculate_imbalances(hashmap!{resource.clone() => value}, &pool_info);
            let fee_base = value * self.config.swap_base_fee;
            let fee_impact = (imbalance_f.pow(self.config.swap_impact_exp) - imbalance_0.pow(self.config.swap_impact_exp)) * self.config.swap_impact_fee;
            let fee = (fee_base + fee_impact).max(dec!(0));

            let amount_mint = if pool_info.lp_token_supply.is_zero() {
                (value - fee) / pool_info.lp_token_price
            } else {
                value
            };

            let lp_tokens = self.pool.lp_token_manager.mint(amount_mint);
            let tokens = account.vaults.take(resource, amount);
            account.vaults.put(lp_tokens);
            self.pool.vaults.put(tokens);

            amount_mint
        }

        fn add_liquidity_as_collateral(&mut self, account_id: NonFungibleLocalId, resource: ResourceAddress, amount: Decimal, prices: HashMap<ResourceAddress, Decimal>) -> Decimal {
            let amount_lp = self.add_liquidity(account_id.clone(), resource, amount, prices);
            
            let mut account = self.accounts.get_mut(&account_id).expect("Missing account");
            let lp_tokens = account.vaults.take(resource, amount_lp);
            account.collateral.put(lp_tokens);

            amount_lp
        }

        fn remove_liquidity(&mut self, account_id: NonFungibleLocalId, amount_lp: Decimal, resource: ResourceAddress, prices: HashMap<ResourceAddress, Decimal>) {
            let pool_info = self.get_pool_info(prices.clone());
            self.update_pool_interest(&pool_info);
            {
                let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

                if amount_lp > account.vaults.amount(&self.pool.lp_token_manager.address()) {
                    panic!("Insufficient balance in account");
                }

                let value = amount_lp * pool_info.lp_token_price;
                let (imbalance_0, imbalance_f) = calculate_imbalances(hashmap!{resource.clone() => -value}, &pool_info);
                let fee_base = value * self.config.swap_base_fee;
                let fee_impact = (imbalance_f.pow(self.config.swap_impact_exp) - imbalance_0.pow(self.config.swap_impact_exp)) * self.config.swap_impact_fee;
                let fee = (fee_base + fee_impact).max(dec!(0));

                let price = *prices.get(&resource).expect("Missing price");
                let amount_withdraw = if pool_info.lp_token_supply == amount_lp {
                    value / price
                } else {
                    (value - fee) / price
                };
                
                if amount_withdraw > self.pool.vaults.amount(&resource) {
                    panic!("Insufficient balance in pool");
                }

                account.vaults.take(self.pool.lp_token_manager.address(), amount_lp).burn();
                let tokens = self.pool.vaults.take(resource, amount_withdraw);
                account.vaults.put(tokens);
            }
            self.assert_pool_integrity();
        }

        fn remove_collateral(&mut self, account_id: NonFungibleLocalId, amount_lp: Decimal, prices: HashMap<ResourceAddress, Decimal>) {
            let (account_info, pool_info) = self.get_info(account_id.clone(), prices.clone());
            self.update_pool_interest(&pool_info);
            self.update_account_interest(account_id.clone(), &account_info);
            {
                let mut account = self.accounts.get_mut(&account_id).expect("Missing account");

                if amount_lp > account.collateral.amount() {
                    panic!("Insufficient collateral balance in account");
                }

                let value = amount_lp * pool_info.lp_token_price;
                let positions_liquidation_value = account_info.positions_value - account_info.liquidation_fee;
                let account_value = positions_liquidation_value + account_info.collateral_value;
                assert!(
                    (account_value - value) / (account_info.collateral_value - value) >= self.config.min_collateral_ratio,
                    "Insufficient collateral ratio"
                );

                let lp_tokens = account.collateral.take(amount_lp);
                account.vaults.put(lp_tokens);
            }
        }

        // fn swap_order(&mut self, account_id: NonFungibleLocalId, resource_in: ResourceAddress, amount_in: Decimal, resource_out: ResourceAddress, price_limit: Limit, prices: HashMap<ResourceAddress, Decimal>) -> Decimal {
        // fn margin_order(&mut self, account_id: NonFungibleLocalId, resource_in: ResourceAddress, amount_in: Decimal, resource_out: ResourceAddress, price_limit: Limit, prices: HashMap<ResourceAddress, Decimal>) -> Decimal {
        
        
        // fn remove_collateral_as_token(&mut self, account_id: NonFungibleLocalId, amount_lp: Decimal, resource: ResourceAddress, prices: HashMap<ResourceAddress, Decimal>) {
        // fn restrike_order(&mut self, account_id: NonFungibleLocalId, order_id: NonFungibleLocalId, price_limit: Limit, prices: HashMap<ResourceAddress, Decimal>) -> Decimal {
    }
}
