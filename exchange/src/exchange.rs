// TODO: remove dead code
#![allow(dead_code)]

pub mod config;
pub mod consts;
pub mod errors;
pub mod keeper_requests;
pub mod liquidity_pool;
pub mod margin_account;
pub mod oracle;

use scrypto::prelude::*;
use crate::utils::*;
use self::config::*;
use self::consts::*;
use self::errors::*;
use self::keeper_requests::*;
use self::liquidity_pool::*;
use self::margin_account::*;
use self::oracle::oracle::Oracle;

#[blueprint]
mod exchange {
    struct Exchange {
        config: ExchangeConfig,
        pool: LiquidityPool,
        oracle: Global<Oracle>,
    }
    impl Exchange {
        pub fn new() -> Global<Exchange> {
            // TODO: for testing purposes
            let owner_role = OwnerRole::None;
            let resources = vec![];

            let (address_reservation, this) = Runtime::allocate_component_address(Exchange::blueprint_id());
            Self {
                config: ExchangeConfig {
                    max_price_age_seconds: 0,
                    keeper_fee: dec!(0),
                    positions_max: 0,
                    skew_ratio_cap: dec!(0),
                    skew_ratio_adl: dec!(0),
                    fee_max: dec!(0),
                    pairs: List::new(),
                    collaterals: HashMap::new(),
                },
                pool: LiquidityPool::new(this, owner_role.clone()),
                oracle: Oracle::new(resources.clone()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role.clone())
            .with_address(address_reservation)
            .globalize()
        }

        // get_pool_value
        fn get_pool_value(&self) -> Decimal {
            self.pool.base_tokens.amount() + 
            self.pool.virtual_balance + 
            self.pool.unrealized_pool_funding +
            self.pool.pnl_snap
        }

        // assert_skewness
        fn assert_pool_integrity(&mut self) {
            if self.pool.skew_abs_snap / self.get_pool_value() >= self.config.skew_ratio_cap {
                panic!("{}", ERROR_SKEW_TOO_HIGH);
            }
        }

        fn assert_account_integrity(&mut self, account: &VirtualMarginAccount) {
            let (pnl, margin) = self.value_positions(account);
            let collateral = self.value_collateral(account);

            if pnl + collateral < margin {
                panic!("{}", ERROR_INSUFFICIENT_MARGIN);
            }
        }

        fn assert_valid_collateral(
            &self, 
            resource: ResourceAddress,
        ) {
            if !self.config.collaterals.contains_key(&resource) {
                panic!("{}", ERROR_COLLATERAL_INVALID);
            }
            if self.config.collaterals.get(&resource).unwrap().disabled {
                panic!("{}", ERROR_COLLATERAL_DISABLED);
            }
        }

        // TODO: assert not too many positions

        // update_pair_snaps
        fn update_pair_snaps(
            &mut self, 
            pair_id: u64,
        ) {
            let price_token = self.oracle.get_price(pair_id);

            let mut position = self.pool.positions.get_mut(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
            let oi_long = position.oi_long;
            let oi_short = position.oi_short;

            let skew = (oi_long - oi_short) * price_token;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - position.skew_abs_snap;
            position.skew_abs_snap = skew_abs;
            self.pool.skew_abs_snap += skew_abs_snap_delta;

            let pnl = skew - position.cost;
            let pnl_snap_delta = pnl - position.pnl_snap;
            position.pnl_snap = pnl;
            self.pool.pnl_snap += pnl_snap_delta;
        }

        // update_pair
        fn update_pair(
            &mut self, 
            pair_id: u64,
        ) {
            let config = self.config.pairs.get(pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = self.oracle.get_price(pair_id);

            let mut pool_position = self.pool.positions.get_mut(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price_token;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - pool_position.skew_abs_snap;
            pool_position.skew_abs_snap = skew_abs;
            self.pool.skew_abs_snap += skew_abs_snap_delta;

            let pnl = skew - pool_position.cost;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;
            self.pool.pnl_snap += pnl_snap_delta;
            
            let current_time = Clock::current_time_rounded_to_minutes();
            let period_minutes = Decimal::from((current_time.seconds_since_unix_epoch - pool_position.last_update.seconds_since_unix_epoch) / 60);
            
            if period_minutes.is_zero() {
                return ;
            }
            
            pool_position.last_update = current_time;

            let funding_2_rate_delta = skew * config.funding_2_delta * period_minutes;
            pool_position.funding_2_rate += funding_2_rate_delta;

            if !oi_long.is_zero() && !oi_short.is_zero() {
                let funding_1_rate = skew * config.funding_1;
                let funding_2_rate = pool_position.funding_2_rate * config.funding_2;
                let funding_rate = funding_1_rate + funding_2_rate;

                let (funding_long_index, funding_short_index, funding_share) = if funding_rate.is_positive() {
                    let funding_long = funding_rate * period_minutes;
                    let funding_long_index = funding_long / oi_long;
    
                    let funding_share = funding_long * config.funding_share;
                    let funding_short_index = -(funding_long - funding_share) / oi_short;
    
                    (funding_long_index, funding_short_index, funding_share)
                } else {
                    let funding_short = -funding_rate * period_minutes * price_token;
                    let funding_short_index = funding_short / oi_short;
    
                    let funding_share = funding_short * config.funding_share;
                    let funding_long_index = -(funding_short - funding_share) / oi_long;
    
                    (funding_long_index, funding_short_index, funding_share)
                };

                let funding_pool_0_rate = (oi_long + oi_short) * price_token * config.funding_pool_0;
                let funding_pool_1_rate = skew_abs * config.funding_pool_1;
                let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;

                let funding_pool = funding_pool_rate * period_minutes;
                let funding_pool_index = funding_pool / (oi_long + oi_short);
                self.pool.unrealized_pool_funding += funding_pool + funding_share;

                pool_position.funding_long_index += funding_long_index + funding_pool_index;
                pool_position.funding_short_index += funding_short_index + funding_pool_index;
            }
        }
        
        // margin_order
        fn margin_order(
            &mut self, 
            account: &mut VirtualMarginAccount, 
            pair_id: u64, 
            amount: Decimal, 
            vault_transfers: Vec<(ResourceAddress, Decimal)>, 
        ) {
            for (resource, _amount) in vault_transfers.iter() {
                self.assert_valid_collateral(*resource);
            }
            account.transfer_from_vaults_to_collateral(vault_transfers, TO_INFINITY);

            self.update_pair(pair_id); // TODO: Do we need to do this
            self.settle_funding(account, pair_id);
                
            let (amount_close, amount_open) = {
                let position_amount = account.positions().get(&pair_id).map_or(dec!(0), |p| p.amount);
                if position_amount.is_positive() && amount.is_negative() {
                    let amount_close = amount.max(-position_amount);
                    let amount_open = amount - amount_close;
                    (amount_close, amount_open)
                } else if position_amount.is_negative() && amount.is_positive() {
                    let amount_close = amount.min(-position_amount);
                    let amount_open = amount - amount_close;
                    (amount_close, amount_open)
                } else {
                    (dec!(0), amount)
                }
            };
            if !amount_close.is_zero() {
                self.close_position(account, pair_id, amount_close);
            }
            if !amount_open.is_zero() {
                self.open_position(account, pair_id, amount_open);
            }

            self.save_funding_index(account, pair_id);
            self.update_pair_snaps(pair_id);

            self.assert_account_integrity(account);
            self.assert_pool_integrity();
        }

        fn settle_with_pool(
            &mut self,
            account: &mut VirtualMarginAccount,
            amount: Decimal,
        ) {
            let outstanding_base = if amount.is_positive() {
                let available_base = self.pool.base_tokens.amount();
                let amount_base = amount.min(available_base);
                let tokens_base = self.pool.base_tokens.take_advanced(amount_base, TO_ZERO);
                account.deposit_collateral(tokens_base);

                amount - amount_base
            } else {
                let available_base = account.collateral_amount(&BASE_RESOURCE);
                let amount_base = (-amount).min(available_base);
                let tokens_base = account.withdraw_collateral(&BASE_RESOURCE, amount_base, TO_INFINITY);
                self.pool.base_tokens.put(tokens_base);
                
                amount + amount_base
            };
            account.update_virtual_balance(account.virtual_balance() + outstanding_base);
            self.pool.virtual_balance -= outstanding_base;
        }

        // settle_funding
        fn settle_funding(
            &mut self,
            account: &mut VirtualMarginAccount,
            pair_id: u64,
        ) {
            let funding = {
                let pool_position = self.pool.positions.get(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
    
                let funding = if let Some(position) = account.positions().get(&pair_id) {
                    if position.amount.is_positive() {
                        position.amount * (position.funding_index - pool_position.funding_long_index)
                    } else {
                        position.amount * (position.funding_index - pool_position.funding_short_index)            
                    }
                } else {
                    dec!(0)
                };
                self.pool.unrealized_pool_funding += funding;

                funding
            };
            self.settle_with_pool(account, funding);
        }
        
        fn save_funding_index(
            &mut self,
            account: &mut VirtualMarginAccount,
            pair_id: u64,
        ) {
            let pool_position = self.pool.positions.get(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
            let mut position = account.position(&pair_id);

            let funding_index = if position.amount.is_positive() {
                pool_position.funding_long_index
            } else {
                pool_position.funding_short_index
            };
            position.funding_index = funding_index;

            account.update_position(&pair_id, position)
        }

        // open_position
        fn open_position(
            &mut self, 
            account: &mut VirtualMarginAccount, 
            pair_id: u64, 
            amount: Decimal, 
        ) {
            let config = self.config.pairs.get(pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = self.oracle.get_price(pair_id);

            let value = amount * price_token;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self.get_pool_value();

            let mut pool_position = self.pool.positions.get_mut(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
            let mut position = account.position(&pair_id);
            
            let skew_abs = ((pool_position.oi_long - pool_position.oi_short + amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.fee_max);
            let cost = value + fee;

            if amount.is_positive() {
                pool_position.oi_long += amount;
            } else {
                pool_position.oi_short -= amount;
            }
            pool_position.cost += cost;
            
            position.amount += amount;
            position.cost += cost;

            account.update_position(&pair_id, position);
        }

        // close_position
        fn close_position(
            &mut self, 
            account: &mut VirtualMarginAccount, 
            pair_id: u64, 
            amount: Decimal, 
        ) {
            let pnl = {
                let config = self.config.pairs.get(pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = self.oracle.get_price(pair_id);
                
                let value = amount * price_token;
                let value_abs = value.checked_abs().unwrap();
                let pool_value = self.get_pool_value();

                let mut pool_position = self.pool.positions.get_mut(&pair_id).expect(ERROR_MISSING_POOL_POSITION);
                let mut position = account.position(&pair_id);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.fee_max);
                let cost = -amount / position.amount * position.cost;
                let pnl = value - cost - fee;
            
                if position.amount.is_positive() {
                    pool_position.oi_long -= amount;
                } else {
                    pool_position.oi_short += amount;
                }
                pool_position.cost -= cost;

                position.amount += amount;
                position.cost -= cost;

                account.update_position(&pair_id, position);

                pnl
            };

            self.settle_with_pool(account, pnl);
        }

        // TODO: add_collateral?

        // // add_collateral
        // fn add_collateral(
        //     &mut self, 
        //     account: &mut VirtualMarginAccount, 
        //     transfers: Vec<(ResourceAddress, Decimal)>,
        // ) {
        //     self.assert_valid_collateral(*resource);

        //     let collateral = account.vaults.take_advanced(resource, amount, INCOMING);
        //     account.collateral.put(collateral);
        // }

        // liquidate
        fn liquidate(
            &mut self,
            account: &mut VirtualMarginAccount,
            mut payment_tokens: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                payment_tokens.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let (pnl, margin) = self.liquidate_positions(account);
            let (collateral_value, mut collateral_tokens) = self.liquidate_collateral(account);

            assert!(
                pnl + collateral_value < margin,
                "{}", ERROR_LIQUIDATION_SUFFICIENT_MARGIN
            );
            
            account.deposit_collateral(payment_tokens.take_advanced(collateral_value, TO_INFINITY));

            self.settle_with_pool(account, pnl);
            // TODO: insurance fund for outstanding_base

            collateral_tokens.push(payment_tokens);
            
            collateral_tokens
        }

        // calculate_positions_value
        fn liquidate_positions(
            &mut self, 
            account: &mut VirtualMarginAccount, 
        ) -> (Decimal, Decimal) {
            let pool_value = self.get_pool_value();
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (pair_id, position) in account.positions().clone().iter() {
                let config = self.config.pairs.get(*pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = self.oracle.get_price(*pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let mut pool_position = self.pool.positions.get_mut(&pair_id).expect(ERROR_MISSING_POOL_POSITION);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.fee_max);
                let cost = position.cost;

                if position.amount.is_positive() {
                    pool_position.oi_long -= amount;
                } else {
                    pool_position.oi_short += amount;
                }
                pool_position.cost -= cost;

                let pnl = value - cost - fee;
                let margin = value_abs * config.margin_maintenance;
                total_pnl += pnl;
                total_margin += margin;

                account.update_position(&pair_id, AccountPosition::default());
            }

            (total_pnl, total_margin)
        }

        // calculate_account_value
        fn liquidate_collateral(
            &mut self, 
            account: &mut VirtualMarginAccount, 
        ) -> (Decimal, Vec<Bucket>) {            
            let mut total_value = dec!(0);
            let mut withdraw_collateral = vec![];
            for (resource, config) in self.config.collaterals.iter() {
                let price_resource = self.oracle.get_price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource * config.discount;
                withdraw_collateral.push((*resource, amount));
                total_value += value;
            }
            let collateral_tokens = account.withdraw_collateral_batch(withdraw_collateral, TO_ZERO);

            (total_value, collateral_tokens)
        }

        fn value_positions(
            &self, 
            account: &VirtualMarginAccount, 
        ) -> (Decimal, Decimal) {
            let pool_value = self.get_pool_value();
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (pair_id, position) in account.positions().iter() {
                let config = self.config.pairs.get(*pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = self.oracle.get_price(*pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = self.pool.positions.get(&pair_id).expect(ERROR_MISSING_POOL_POSITION);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.fee_max);
                let cost = position.cost;

                let pnl = value - cost - fee;
                let margin = value_abs * config.margin_initial;
                total_pnl += pnl;
                total_margin += margin;
            }

            (total_pnl, total_margin)
        }

        fn value_collateral(
            &self, 
            account: &VirtualMarginAccount, 
        ) -> Decimal {
            let mut total_value = dec!(0);
            for (resource, config) in self.config.collaterals.iter() {
                let price_resource = self.oracle.get_price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource * config.discount;
                total_value += value;
            }
            total_value
        }
        
        // remove_collateral
        fn remove_collateral(
            &mut self, 
            account: &mut VirtualMarginAccount, 
            collateral_transfers: Vec<(ResourceAddress, Decimal)>,
        ) {
            account.transfer_from_collateral_to_vaults(collateral_transfers, TO_ZERO);
            self.assert_account_integrity(account);
        }

        // swap_debt
        fn swap_debt(
            &mut self, 
            account: &mut VirtualMarginAccount, 
            resource: &ResourceAddress, 
            payment: Bucket, 
        ) -> Bucket {
            assert!(
                payment.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let value = payment.amount();
            let virtual_balance = account.virtual_balance();
            
            assert!(
                value <= -virtual_balance,
                "{}", ERROR_SWAP_NOT_ENOUGH_DEBT
            );
            let price_resource = self.oracle.get_price_resource(*resource);
            let amount = value / price_resource;
            // TODO: check amount first?

            self.pool.base_tokens.put(payment);
            self.pool.virtual_balance -= value;
            account.update_virtual_balance(virtual_balance + value);
            let collateral = account.withdraw_collateral(resource, amount, TO_ZERO);

            collateral
        }


        // auto_deleverage
        
    }
}
