// TODO: remove
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]


mod config;
mod consts;
mod errors;
mod requests;
mod virtual_liquidity_pool;
mod virtual_margin_account;
mod virtual_oracle;

use scrypto::prelude::*;
use utils::*;
use account::*;
use pool::*;
use self::config::*;
use self::consts::*;
use self::errors::*;
use self::requests::*;
use self::virtual_liquidity_pool::*;
use self::virtual_margin_account::*;
use self::virtual_oracle::*;

#[blueprint]
mod exchange {
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        MarginAccount {
            // Constructor
            fn new(initial_rule: AccessRule) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self, collateral_resources: Vec<ResourceAddress>) -> MarginAccountInfo;
            fn get_request(&self, index: u64) -> Option<KeeperRequest>;
            fn get_requests(&self, start: u64, end: u64) -> Vec<KeeperRequest>;

            // Authority protected methods
            fn update(&self, update: MarginAccountUpdates);
            fn push_request(&self, request: KeeperRequest);
            fn set_request_status(&self, index: u64, status: u8);
            fn process_request(&self, index: u64) -> Option<KeeperRequest>;
            fn deposit_collateral(&self, token: Bucket);
            fn deposit_collateral_batch(&self, tokens: Vec<Bucket>);
            fn withdraw_collateral(&self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;
            fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>;
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        MarginPool {
            // Getter methods
            fn get_info(&self) -> MarginPoolInfo;
            fn get_position(&self, position_id: u64) -> Option<PoolPosition>;            

            // Authority protected methods
            fn update(&self, update: MarginPoolUpdates);
            fn deposit(&mut self, token: Bucket);
            fn withdraw(&mut self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;            
            fn mint_lp(&mut self, amount: Decimal) -> Bucket;
            fn burn_lp(&mut self, token: Bucket);
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Oracle {
            // Getter methods
            fn prices(&self) -> HashMap<u64, Decimal>;
        }
    }

    enable_method_auth! { 
        roles {
            admin => updatable_by: [OWNER];
            keeper => updatable_by: [OWNER, admin];
            user => updatable_by: [OWNER, admin];
        },
        methods { 
            deposit_authority => restrict_to: [OWNER];
            withdraw_authority => restrict_to: [OWNER];
            update_exchange_config => restrict_to: [OWNER];
            update_pair_configs => restrict_to: [OWNER];
            update_collateral_configs => restrict_to: [OWNER];
            remove_collateral_config => restrict_to: [OWNER];

            get_exchange_config => PUBLIC;
            get_num_pairs => PUBLIC;
            get_pair_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_collateral_config => PUBLIC;
            get_collateral_configs => PUBLIC;
            get_collaterals => PUBLIC;
            get_pool_value => PUBLIC;
            get_skew_ratio => PUBLIC;

            create_account => restrict_to: [user];
            set_level_1_auth => restrict_to: [user];
            set_level_2_auth => restrict_to: [user];
            set_level_3_auth => restrict_to: [user];
            add_liquidity => restrict_to: [user];
            remove_liquidity => restrict_to: [user];
            add_collateral => restrict_to: [user];
            remove_collateral_request => restrict_to: [user];
            margin_order_request => restrict_to: [user];
            cancel_request => restrict_to: [user];

            process_request => restrict_to: [keeper];
            swap_debt => restrict_to: [keeper];
            liquidate => restrict_to: [keeper];
            auto_deleverage => restrict_to: [keeper];
            update_pair => restrict_to: [keeper];
        }
    }

    macro_rules! authorize {
        ($self:expr, $func:expr) => {{
            $self.authority_token.create_proof_of_amount(dec!(1)).authorize(|| {
                $func
            })
        }};
    }
    struct Exchange {
        authority_token: FungibleVault,
        config: Config,
        pool: ComponentAddress,
        oracle: ComponentAddress,
    }
    
    impl Exchange {
        pub fn new(
            owner_rule: AccessRule,
            authority_token: Bucket,
            pool: ComponentAddress,
            oracle: ComponentAddress,
        ) -> Global<Exchange> {
            // TODO: for testing purposes
            let (component_reservation, _this) = Runtime::allocate_component_address(Exchange::blueprint_id());
            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                config: Config {
                    exchange: ExchangeConfig {
                        max_price_age_seconds: 0,
                        keeper_fee: dec!(0),
                        positions_max: 0,
                        skew_ratio_cap: dec!(0),
                        adl_offset: dec!(0),
                        adl_a: dec!(0),
                        adl_b: dec!(0),
                        fee_liquidity: dec!(0),
                        fee_max: dec!(0),
                    },
                    pairs: HashList::new(),
                    collaterals: HashMap::new(),
                },
                pool,
                oracle,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(owner_rule))
            .roles(roles! {
                admin => OWNER;
                keeper => rule!(allow_all);
                user => rule!(allow_all);
            })
            .with_address(component_reservation)
            .globalize()
        }

        // --- OWNER METHODS ---

        pub fn deposit_authority(
            &mut self, 
            token: Bucket,
        ) {
            self.authority_token.put(token.as_fungible());
        }

        pub fn withdraw_authority(
            &mut self
        ) -> Bucket {
            self.authority_token.take_all().into()
        }

        // --- ADMIN METHODS ---

        pub fn update_exchange_config(
            &mut self, 
            config: ExchangeConfig,
        ) {
            config.validate();
            self.config.exchange = config;
        }

        pub fn update_pair_configs(
            &mut self, 
            pair_id: u64, 
            configs: Vec<PairConfig>,
        ) {
            for config in configs.iter() {
                config.validate();
                self.config.pairs.insert(pair_id, config.clone());
            }
        }

        pub fn update_collateral_configs(
            &mut self, 
            resource: ResourceAddress, 
            configs: Vec<CollateralConfig>,
        ) {
            for config in configs.iter() {
                config.validate();
                self.config.collaterals.insert(resource, config.clone());
            }
        }

        pub fn remove_collateral_config(
            &mut self, 
            resource: ResourceAddress,
        ) {
            self.config.collaterals.remove(&resource);
        }

        // --- GET METHODS ---

        pub fn get_exchange_config(
            &self
        ) -> ExchangeConfig {
            self.config.exchange.clone()
        }

        pub fn get_num_pairs(
            &self,
        ) -> u64 {
            self.config.pairs.len()
        }

        pub fn get_pair_config(
            &self, 
            pair_id: u64,
        ) -> PairConfig {
            self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG).clone()
        }

        pub fn get_pair_configs(&self, 
            start: u64, 
            end: u64,
        ) -> Vec<PairConfig> {
            self.config.pairs.range(start, end)
        }

        pub fn get_collateral_config(
            &self, 
            resource: ResourceAddress,
        ) -> CollateralConfig {
            self.config.collaterals.get(&resource).expect(ERROR_COLLATERAL_INVALID).clone()
        }

        pub fn get_collateral_configs(
            &self, 
            resource: ResourceAddress,
        ) -> HashMap<ResourceAddress, CollateralConfig> {
            self.config.collaterals.clone()
        }

        pub fn get_collaterals(
            &self,
        ) -> Vec<ResourceAddress> {
            self._collaterals()
        }

        pub fn get_pool_value(
            &self,
        ) -> Decimal {
            let pool = VirtualLiquidityPool::new(self.pool);
            self._pool_value(&pool)
        }

        pub fn get_skew_ratio(
            &self,
        ) -> Decimal {
            let pool = VirtualLiquidityPool::new(self.pool);
            self._skew_ratio(&pool)
        }

        // --- USER METHODS ---

        pub fn create_account(
            &self, 
            initial_rule: AccessRule,
        ) -> ComponentAddress {
            // TODO: globalize within exchange?
            authorize!(self, {
                Blueprint::<MarginAccount>::new(initial_rule).address()
            })
        }

        pub fn set_level_1_auth(
            &self, 
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();
                account.set_level_1_auth(rule);
                account.realize();
            })
        }

        pub fn set_level_2_auth(
            &self, 
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();
                account.set_level_2_auth(rule);
                account.realize();
            })
        }

        pub fn set_level_3_auth(
            &self, 
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();
                account.set_level_3_auth(rule);
                account.realize();
            })
        }

        pub fn add_liquidity(
            &self,
            payment: Bucket,
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool);
                let lp_token = self._add_liquidity(&mut pool, payment);
                pool.realize();
                lp_token
            })
        }

        pub fn remove_liquidity(
            &self,
            lp_token: Bucket,
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool);
                let payment = self._remove_liquidity(&mut pool, lp_token);
                pool.realize();
                payment
            })
        }

        pub fn add_collateral(
            &self, 
            account: ComponentAddress, 
            tokens: Vec<Bucket>,
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                self._add_collateral(&mut account, tokens);
                account.realize();
            })
        }

        pub fn remove_collateral_request(
            &self,
            expiry_seconds: u64, 
            account: ComponentAddress, 
            target_account: ComponentAddress,
            claims: Vec<(ResourceAddress, Decimal)>,
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();

                let request = Request::RemoveCollateral(RequestRemoveCollateral {
                    target_account,
                    claims,
                });
                account.push_request(request, expiry_seconds);

                account.realize();
            })
        }

        pub fn margin_order_request(
            &self,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: u64,
            amount: Decimal,
            price_limit: Limit,
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_2_auth();

                let request = Request::MarginOrder(RequestMarginOrder {
                    pair_id,
                    amount,
                    price_limit,
                });
                account.push_request(request, expiry_seconds);

                account.realize();
            })
        }

        pub fn cancel_request(
            &self, 
            account: ComponentAddress, 
            index: u64,
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_2_auth();

                account.set_request_status(index, 2);
                account.realize();
            })
        }

        // --- KEEPER METHODS ---

        pub fn process_request(
            &self, 
            account: ComponentAddress, 
            index: u64,
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                let request = account.process_request(index);

                let mut pool = VirtualLiquidityPool::new(self.pool);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds());

                match request {
                    Request::RemoveCollateral(request) => {
                        let target_account = request.target_account;
                        let claims = request.claims;
                        self._remove_collateral(&mut pool, &mut account, &oracle, target_account, claims);
                    },
                    Request::MarginOrder(request) => {
                        let pair_id = request.pair_id;
                        let amount = request.amount;
                        let price_limit = request.price_limit;
                        self._margin_order(&mut pool, &mut account, &oracle, pair_id, amount);
                    },
                }

                account.realize();
                pool.realize();
            })
        }

        pub fn swap_debt(
            &self, 
            account: ComponentAddress, 
            resource: ResourceAddress, 
            payment: Bucket, 
        ) -> Bucket {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);
    
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds());
                let token = self._swap_debt(&mut pool, &mut account, &oracle, &resource, payment);
    
                account.realize();
                pool.realize();
    
                token
            })
        }

        pub fn liquidate(
            &self,
            account: ComponentAddress,
            payment_tokens: Bucket,
        ) -> Vec<Bucket> {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds());
                let tokens = self._liquidate(&mut pool, &mut account, &oracle, payment_tokens);

                account.realize();
                pool.realize();

                tokens
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: u64, 
        ) {
            authorize!(self, {
                let mut account =  VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds());
                self._auto_deleverage(&mut pool, &mut account, &oracle, pair_id);

                account.realize();
                pool.realize();
            })
        }

        pub fn update_pair(
            &self, 
            pair_id: u64,
        ) {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds());
                self._update_pair(&mut pool, &oracle, pair_id);
    
                pool.realize();
            })
        }

        // --- INTERNAL METHODS ---

        fn _collaterals(
            &self,
        ) -> Vec<ResourceAddress> {
            self.config.collaterals.keys().cloned().collect()
        }

        fn _collateral_feeds(
            &self,
        ) -> HashMap<ResourceAddress, u64> {
            self.config.collaterals.iter().map(|(resource, config)| (*resource, config.pair_id)).collect()
        }

        // get_pool_value
        fn _pool_value(
            &self, 
            pool: &VirtualLiquidityPool,
        ) -> Decimal {
            pool.base_tokens_amount() + 
            pool.virtual_balance() + 
            pool.unrealized_pool_funding() +
            pool.pnl_snap()
        }

        fn _skew_ratio(
            &self,
            pool: &VirtualLiquidityPool,
        ) -> Decimal {
            pool.skew_abs_snap() / self._pool_value(pool)
        }

        fn _assert_pool_integrity(
            &self,
            pool: &VirtualLiquidityPool,
        ) {
            if self._skew_ratio(pool) >= self.config.exchange.skew_ratio_cap {
                panic!("{}", ERROR_SKEW_TOO_HIGH);
            }
        }

        fn _assert_account_integrity(
            &self, 
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount,
            oracle: &VirtualOracle,
        ) {
            let (pnl, margin) = self._value_positions(pool, account, oracle);
            let collateral = self._value_collateral(account, oracle);

            if pnl + collateral < margin {
                panic!("{}", ERROR_INSUFFICIENT_MARGIN);
            }
        }

        fn _assert_valid_collateral(
            &self, 
            resource: ResourceAddress,
        ) {
            if !self.config.collaterals.contains_key(&resource) {
                panic!("{}", ERROR_COLLATERAL_INVALID);
            }
        }

        fn _assert_position_limit(
            &self, 
            account: &VirtualMarginAccount,
        ) {
            if account.positions().len() > self.config.exchange.positions_max as usize {
                panic!("{}", ERROR_POSITIONS_TOO_MANY);
            }
        }

        fn _add_liquidity(
            &self,
            pool: &mut VirtualLiquidityPool,
            payment: Bucket,
        ) -> Bucket {
            assert!(
                payment.resource_address() == BASE_RESOURCE,
                "{}", ERROR_INVALID_PAYMENT
            );

            let pool_value = self._pool_value(pool);
            let value = payment.amount();
            let fee = value * self.config.exchange.fee_liquidity;
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();

            let lp_amount = (value - fee) / pool_value * lp_supply;

            pool.deposit(payment);
            let lp_token = pool.mint_lp(lp_amount);

            lp_token
        }

        fn _remove_liquidity(
            &self,
            pool: &mut VirtualLiquidityPool,
            lp_token: Bucket,
        ) -> Bucket {
            assert!(
                lp_token.resource_address() == pool.lp_token_manager().address(),
                "{}", ERROR_INVALID_LP_TOKEN
            );

            let pool_value = self._pool_value(pool);
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();
            let lp_amount = lp_token.amount();

            let value = lp_amount / lp_supply * pool_value;
            let fee = value * self.config.exchange.fee_liquidity;

            pool.burn_lp(lp_token);
            let payment = pool.withdraw(value - fee, TO_ZERO);

            payment
        }

        fn _add_collateral(
            &self, 
            account: &mut VirtualMarginAccount, 
            tokens: Vec<Bucket>,
        ) {
            for tokens in tokens.iter() {
                self._assert_valid_collateral(tokens.resource_address());
            }

            account.deposit_collateral_batch(tokens);
        }
        
        fn _remove_collateral(
            &self, 
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            target_account: ComponentAddress,
            claims: Vec<(ResourceAddress, Decimal)>,
        ) {
            let mut target_account = Global::<Account>::try_from(target_account).expect(ERROR_INVALID_ACCOUNT);
            
            let tokens = account.withdraw_collateral_batch(claims, TO_ZERO);
            target_account.try_deposit_batch_or_abort(tokens, None); // TODO: create authorization badge
            
            self._assert_account_integrity(pool, account, oracle);
        }

        fn _margin_order(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            pair_id: u64, 
            amount: Decimal, 
        ) {
            self._update_pair(pool, oracle, pair_id); // TODO: Do we need to do this
            self._settle_funding(pool, account, pair_id);
                
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
                self._close_position(pool, account, oracle, pair_id, amount_close);
            }
            if !amount_open.is_zero() {
                self._open_position(pool, account, oracle, pair_id, amount_open);
            }

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            self._assert_account_integrity(pool, account, oracle);
            self._assert_pool_integrity(pool);
        }

        fn _swap_debt(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
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
            let price_resource = oracle.price_resource(*resource);
            let amount = value / price_resource;
            // TODO: check amount first? take less of two?

            pool.deposit(payment);
            pool.update_virtual_balance(pool.virtual_balance() - value);
            account.update_virtual_balance(virtual_balance + value);
            let collateral = account.withdraw_collateral(resource, amount, TO_ZERO);

            collateral
        }

        fn _liquidate(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            mut payment_tokens: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                payment_tokens.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let (pnl, margin) = self._liquidate_positions(pool, account, oracle);
            let (collateral_value, mut collateral_tokens) = self._liquidate_collateral(account, oracle);

            assert!(
                pnl + collateral_value < margin,
                "{}", ERROR_LIQUIDATION_SUFFICIENT_MARGIN
            );
            
            account.deposit_collateral(payment_tokens.take_advanced(collateral_value, TO_INFINITY));

            self._settle_with_pool(pool, account, pnl);
            // TODO: insurance fund for outstanding_base

            collateral_tokens.push(payment_tokens);
            
            collateral_tokens
        }

        fn _auto_deleverage(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: u64, 
        ) {
            self._update_pair(pool, oracle, pair_id); // TODO: Do we need to do this
            self._settle_funding(pool, account, pair_id);

            let skew_ratio = self._skew_ratio(pool);
            assert!(
                skew_ratio > self.config.exchange.skew_ratio_cap,
                "{}", ERROR_ADL_SKEW_TOO_LOW
            );
                
            let position = account.position(pair_id);
            let price_token = oracle.price(pair_id);
            let amount = position.amount;
            let value = position.amount * price_token;

            let cost = position.cost;

            let pnl_percent = (value - cost) / cost;

            let u = skew_ratio / self.config.exchange.adl_a - self.config.exchange.adl_offset / self.config.exchange.adl_a;
            let threshold = -(u * u * u) - self.config.exchange.adl_b * u;
            assert!(
                pnl_percent > threshold,
                "{}", ERROR_ADL_PNL_BELOW_THRESHOLD
            );
            
            let amount_close = -amount;
            if !amount_close.is_zero() { // TODO: panic?
                self._close_position(pool, account, oracle, pair_id, amount_close);
            }

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            self._assert_account_integrity(pool, account, oracle); // TODO: not needed?

            let skew_ratio_f = self._skew_ratio(pool);
            assert!(
                skew_ratio_f < skew_ratio_f,
                "{}", ERROR_ADL_SKEW_NOT_REDUCED
            );
        }

        fn _open_position(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: u64, 
            amount: Decimal, 
        ) {
            let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = oracle.price(pair_id);

            let value = amount * price_token;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);
            
            let skew_abs = ((pool_position.oi_long - pool_position.oi_short + amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.exchange.fee_max);
            let cost = value + fee;

            if amount.is_positive() {
                pool_position.oi_long += amount;
            } else {
                pool_position.oi_short -= amount;
            }
            pool_position.cost += cost;
            
            position.amount += amount;
            position.cost += cost;

            pool.update_position(pair_id, pool_position);
            account.update_position(pair_id, position);

            self._assert_position_limit(account);
        }

        fn _close_position(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: u64, 
            amount: Decimal, 
        ) {
            let pnl = {
                let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = oracle.price(pair_id);
                
                let value = amount * price_token;
                let value_abs = value.checked_abs().unwrap();
                let pool_value = self._pool_value(pool);

                let mut pool_position = pool.position(pair_id);
                let mut position = account.position(pair_id);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.exchange.fee_max);
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

                pool.update_position(pair_id, pool_position);
                account.update_position(pair_id, position);

                pnl
            };

            self._settle_with_pool(pool, account, pnl);
        }

        fn _update_pair(
            &self, 
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: u64,
        ) {
            let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = oracle.price(pair_id);

            let mut pool_position = pool.position(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price_token;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - pool_position.skew_abs_snap;
            pool_position.skew_abs_snap = skew_abs;
            pool.update_skew_abs_snap(pool.skew_abs_snap() + skew_abs_snap_delta);

            let pnl = skew - pool_position.cost;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;
            pool.update_pnl_snap(pool.pnl_snap() + pnl_snap_delta);
            
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
                pool.update_unrealized_pool_funding(pool.unrealized_pool_funding() + funding_pool + funding_share);

                pool_position.funding_long_index += funding_long_index + funding_pool_index;
                pool_position.funding_short_index += funding_short_index + funding_pool_index;
            }

            pool.update_position(pair_id, pool_position);
        }

        fn _update_pair_snaps(
            &self, 
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: u64,
        ) {
            let price_token = oracle.price(pair_id);

            let mut pool_position = pool.position(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price_token;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - pool_position.skew_abs_snap;
            pool_position.skew_abs_snap = skew_abs;
            pool.update_skew_abs_snap(pool.skew_abs_snap() + skew_abs_snap_delta);

            let pnl = skew - pool_position.cost;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;
            pool.update_pnl_snap(pool.pnl_snap() + pnl_snap_delta);

            pool.update_position(pair_id, pool_position);
        }

        fn _value_positions(
            &self,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Decimal) {
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (&pair_id, position) in account.positions().iter() {
                let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position(pair_id);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.exchange.fee_max);
                let cost = position.cost;

                let pnl = value - cost - fee;
                let margin = value_abs * config.margin_initial;
                total_pnl += pnl;
                total_margin += margin;
            }

            (total_pnl, total_margin)
        }

        fn _liquidate_positions(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Decimal) {
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (&pair_id, position) in account.positions().clone().iter() {
                let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let mut pool_position = pool.position(pair_id);

                let skew_abs = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let fee = value_abs * (config.fee_0 + skew_abs / pool_value * config.fee_1).min(self.config.exchange.fee_max);
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

                pool.update_position(pair_id, pool_position);
                account.remove_position(pair_id);
            }

            (total_pnl, total_margin)
        }

        fn _value_collateral(
            &self, 
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> Decimal {
            let mut total_value = dec!(0);
            for (resource, config) in self.config.collaterals.iter() {
                let price_resource = oracle.price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource * config.discount;
                total_value += value;
            }
            total_value
        }

        fn _liquidate_collateral(
            &self, 
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Vec<Bucket>) {            
            let mut total_value = dec!(0);
            let mut withdraw_collateral = vec![];
            for (resource, config) in self.config.collaterals.iter() {
                let price_resource = oracle.price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource * config.discount;
                withdraw_collateral.push((*resource, amount));
                total_value += value;
            }
            let collateral_tokens = account.withdraw_collateral_batch(withdraw_collateral, TO_ZERO);

            (total_value, collateral_tokens)
        }

        fn _settle_with_pool(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            amount: Decimal,
        ) {
            let outstanding_base = if amount.is_positive() {
                let available_base = pool.base_tokens_amount();
                let amount_base = amount.min(available_base);
                let tokens_base = pool.withdraw(amount_base, TO_ZERO);
                account.deposit_collateral(tokens_base);

                amount - amount_base
            } else {
                let available_base = account.collateral_amount(&BASE_RESOURCE);
                let amount_base = (-amount).min(available_base);
                let tokens_base = account.withdraw_collateral(&BASE_RESOURCE, amount_base, TO_INFINITY);
                pool.deposit(tokens_base);
                
                amount + amount_base
            };
            pool.update_virtual_balance(pool.virtual_balance() - outstanding_base);
            account.update_virtual_balance(account.virtual_balance() + outstanding_base);
        }

        fn _settle_funding(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: u64,
        ) {
            let funding = {
                let pool_position = pool.position(pair_id);
    
                let funding = if let Some(position) = account.positions().get(&pair_id) {
                    if position.amount.is_positive() {
                        position.amount * (position.funding_index - pool_position.funding_long_index)
                    } else {
                        position.amount * (position.funding_index - pool_position.funding_short_index)            
                    }
                } else {
                    dec!(0)
                };
                pool.update_unrealized_pool_funding(pool.unrealized_pool_funding() + funding);

                funding
            };
            self._settle_with_pool(pool, account, funding);
        }
        
        fn _save_funding_index(
            &self,
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: u64,
        ) {
            let pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);

            let funding_index = if position.amount.is_positive() {
                pool_position.funding_long_index
            } else {
                pool_position.funding_short_index
            };
            position.funding_index = funding_index;

            account.update_position(pair_id, position)
        }
    }
}
