// TODO: remove
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]


mod config;
mod errors;
mod events;
mod requests;
mod virtual_margin_pool;
mod virtual_margin_account;
mod virtual_oracle;

use scrypto::prelude::*;
use utils::{PairId, ListIndex, HashList, _BASE_RESOURCE, _KEEPER_REWARD_RESOURCE, TO_ZERO, TO_INFINITY};
use account::*;
use pool::*;
use self::config::*;
use self::errors::*;
use self::requests::*;
use self::virtual_margin_pool::*;
use self::virtual_margin_account::*;
use self::virtual_oracle::*;

#[blueprint]
mod exchange {
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;
    const KEEPER_REWARD_RESOURCE: ResourceAddress = _KEEPER_REWARD_RESOURCE;

    extern_blueprint! {
        "package_tdx_2_1ph0tfat6m4srl7lqgx4jntut6x8zdgdqxj4mg993ltslzz2zcnp382",
        MarginAccount {
            // Constructor
            fn new(initial_rule: AccessRule) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self, collateral_resources: Vec<ResourceAddress>) -> MarginAccountInfo;
            fn get_request(&self, index: ListIndex) -> Option<KeeperRequest>;
            fn get_requests(&self, start: ListIndex, end: ListIndex) -> Vec<KeeperRequest>;
            fn get_requests_tail(&self, num: ListIndex) -> Vec<KeeperRequest>;
            fn get_requests_by_indexes(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, Option<KeeperRequest>>;
            fn get_requests_len(&self) -> ListIndex;

            // Authority protected methods
            fn update(&self, update: MarginAccountUpdates);
            fn deposit_collateral_batch(&self, tokens: Vec<Bucket>);
            fn withdraw_collateral_batch(&self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>;
        }
    }
    extern_blueprint! {
        "package_tdx_2_1p5hc4926ylqzs5ctvfsn2wdgjqjrm6jd44zud45rxd8tsscym9af4k",
        MarginPool {
            // Getter methods
            fn get_info(&self) -> MarginPoolInfo;
            fn get_position(&self, pair_id: PairId) -> Option<PoolPosition>;            

            // Authority protected methods
            fn update(&self, update: MarginPoolUpdates);
            fn deposit(&self, token: Bucket);
            fn withdraw(&self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;            
            fn mint_lp(&self, amount: Decimal) -> Bucket;
            fn burn_lp(&self, token: Bucket);
        }
    }
    extern_blueprint! {
        "package_tdx_2_1p46enjlnmuun4pt5rl2k06jjnz7mal5tf6mcg8ufcf5gl4dkra77ae",
        Oracle {
            // Getter methods
            fn prices(&self, max_age: Instant) -> HashMap<PairId, Decimal>;
        }
    }
    extern_blueprint! {
        "package_tdx_2_1phwprch0565rev6d9msn4pw56p4xhza5mja8rlf44hdy442y6qep2w",
        Referrals {
            // Getter methods
            fn get_referrer(&self, account: ComponentAddress) -> Option<ComponentAddress>;

            // Authority protected methods
            fn update_rebate(&self, rebate: Decimal);
            fn update_trickle_up(&self, trickle_up: Decimal);
            fn set_referrer(&self, account: ComponentAddress, referrer: Option<ComponentAddress>);
            fn reward(&self, referred_account: ComponentAddress, token: Bucket);
            fn collect(&self, account: ComponentAddress) -> Bucket;
        }
    }

    enable_method_auth! { 
        roles {
            admin => updatable_by: [OWNER];
            keeper => updatable_by: [OWNER];
            user => updatable_by: [OWNER, admin];
        },
        methods { 
            // Owner methods
            deposit_authority => restrict_to: [OWNER];
            withdraw_authority => restrict_to: [OWNER];
            update_exchange_config => restrict_to: [OWNER];
            update_pair_configs => restrict_to: [OWNER];
            update_collateral_configs => restrict_to: [OWNER];
            remove_collateral_config => restrict_to: [OWNER];
            update_referral_rebate => restrict_to: [OWNER];
            update_referral_trickle_up => restrict_to: [OWNER];

            // Get methods
            get_exchange_config => PUBLIC;
            get_pairs_len => PUBLIC;
            get_pair_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_collateral_config => PUBLIC;
            get_collateral_configs => PUBLIC;
            get_collaterals => PUBLIC;
            get_pool_value => PUBLIC;
            get_skew_ratio => PUBLIC;
            get_referrer => PUBLIC;

            // User methods
            create_account => restrict_to: [user];
            set_level_1_auth => restrict_to: [user];
            set_level_2_auth => restrict_to: [user];
            set_level_3_auth => restrict_to: [user];
            set_referrer => restrict_to: [user];
            collect_referral_rewards => restrict_to: [user];
            add_liquidity => restrict_to: [user];
            remove_liquidity => restrict_to: [user];
            add_collateral => restrict_to: [user];
            remove_collateral_request => restrict_to: [user];
            margin_order_request => restrict_to: [user];
            cancel_request => restrict_to: [user];

            // Keeper methods
            process_request => restrict_to: [keeper];
            swap_debt => restrict_to: [keeper];
            liquidate => restrict_to: [keeper];
            auto_deleverage => restrict_to: [keeper];
            update_pair => restrict_to: [keeper];
        }
    }

    macro_rules! authorize {
        ($self:expr, $func:expr) => {{
            $self.authority_token.create_proof_of_amount(dec!(0.000000000000000001)).authorize(|| {
                $func
            })
        }};
    }

    struct Exchange {
        authority_token: FungibleVault,
        config: Config,
        pool: Global<MarginPool>,
        oracle: Global<Oracle>,
        referrals: Global<Referrals>,
    }
    
    impl Exchange {
        pub fn new(
            owner_role: OwnerRole,
            authority_token: Bucket,
            pool: ComponentAddress,
            oracle: ComponentAddress,
            referrals: ComponentAddress,
        ) -> Global<Exchange> {
            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                config: Config {
                    exchange: ExchangeConfig::default(),
                    pairs: HashList::new(),
                    collaterals: HashMap::new(),
                },
                pool: pool.into(),
                oracle: oracle.into(),
                referrals: referrals.into(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                admin => OWNER;
                keeper => rule!(allow_all);
                user => rule!(allow_all);
            })
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
            configs: Vec<PairConfig>,
        ) {
            for config in configs.iter() {
                config.validate();
                self.config.pairs.insert(config.pair_id, config.clone());
            }
        }

        pub fn update_collateral_configs(
            &mut self, 
                configs: Vec<(ResourceAddress, CollateralConfig)>,
        ) {
            for (resource, config) in configs.iter() {
                config.validate();
                self.config.collaterals.insert(*resource, config.clone());
            }
        }

        pub fn remove_collateral_config(
            &mut self, 
            resource: ResourceAddress,
        ) {
            self.config.collaterals.remove(&resource);
        }

        pub fn update_referral_rebate(
            &self, 
            rebate: Decimal,
        ) {
            authorize!(self, {
                self.referrals.update_rebate(rebate);
            })
        }

        pub fn update_referral_trickle_up(
            &self, 
            trickle_up: Decimal,
        ) {
            authorize!(self, {
                self.referrals.update_trickle_up(trickle_up);
            })
        }

        // --- GET METHODS ---

        pub fn get_exchange_config(
            &self
        ) -> ExchangeConfig {
            self.config.exchange.clone()
        }

        pub fn get_pairs_len(
            &self,
        ) -> ListIndex {
            self.config.pairs.len()
        }

        pub fn get_pair_config(
            &self, 
            pair_id: PairId,
        ) -> PairConfig {
            self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG).clone()
        }

        pub fn get_pair_configs(&self, 
            start: ListIndex, 
            end: ListIndex,
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

        pub fn get_referrer(
            &self, 
            account: ComponentAddress,
        ) -> Option<ComponentAddress> {
            self.referrals.get_referrer(account)
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

        pub fn set_referrer(
            &self, 
            account: ComponentAddress, 
            referrer: Option<ComponentAddress>,
        ) {
            authorize!(self, {
                let account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();
                if let Some(referrer) = referrer {
                    Global::<MarginAccount>::try_from(referrer).expect(ERROR_INVALID_ACCOUNT); // TODO: check
                }
                self.referrals.set_referrer(account.address(), referrer);
                account.realize();
            })
        }

        pub fn collect_referral_rewards(
            &self, 
            account: ComponentAddress, 
        ) -> Bucket {
            authorize!(self, {
                let account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();
                let token = self.referrals.collect(account.address());
                account.realize();
                token
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
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);
                self._add_collateral(&mut pool, &mut account, tokens);
                pool.realize();
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
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_1_auth();

                let request = Request::RemoveCollateral(RequestRemoveCollateral {
                    target_account,
                    claims,
                });
                account.push_request(request, expiry_seconds, STATUS_ACTIVE);

                account.realize();
            })
        }

        pub fn margin_order_request(
            &self,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: PairId,
            amount: Decimal,
            price_limit: Limit,
            active_requests: Vec<ListIndex>,
            cancel_requests: Vec<ListIndex>,
            status: Status,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_2_auth();

                let request = Request::MarginOrder(RequestMarginOrder {
                    pair_id,
                    amount,
                    price_limit,
                    active_requests,
                    cancel_requests,
                });
                account.push_request(request, expiry_seconds, status);

                account.realize();
            })
        }

        pub fn cancel_request(
            &self, 
            account: ComponentAddress, 
            index: ListIndex,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                account.verify_level_2_auth();
                account.cancel_request(index);
                account.realize();
            })
        }

        // --- KEEPER METHODS ---

        pub fn process_request(
            &self, 
            account: ComponentAddress, 
            index: ListIndex,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                let (request, submission) = account.process_request(index);

                let mut pool = VirtualLiquidityPool::new(self.pool);
                let current_time = Clock::current_time_rounded_to_seconds();
                let max_age = current_time.add_seconds(-(self.config.exchange.max_price_age_seconds)).expect(ERROR_ARITHMETIC);
                let max_age = if max_age.compare(submission, TimeComparisonOperator::Gt) {
                    max_age
                } else {
                    submission
                };
                
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds(), max_age);

                match request {
                    Request::RemoveCollateral(request) => {
                        self._remove_collateral(&mut pool, &mut account, &oracle, request);
                    },
                    Request::MarginOrder(request) => {
                        self._margin_order(&mut pool, &mut account, &oracle, request);
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
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let current_time = Clock::current_time_rounded_to_seconds();
                let max_age = current_time.add_seconds(-(self.config.exchange.max_price_age_seconds)).expect(ERROR_ARITHMETIC);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds(), max_age);

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
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let current_time = Clock::current_time_rounded_to_seconds();
                let max_age = current_time.add_seconds(-(self.config.exchange.max_price_age_seconds)).expect(ERROR_ARITHMETIC);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds(), max_age);

                let tokens = self._liquidate(&mut pool, &mut account, &oracle, payment_tokens);

                account.realize();
                pool.realize();

                tokens
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: PairId, 
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account, self._collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let current_time = Clock::current_time_rounded_to_seconds();
                let max_age = current_time.add_seconds(-(self.config.exchange.max_price_age_seconds)).expect(ERROR_ARITHMETIC);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds(), max_age);

                self._auto_deleverage(&mut pool, &mut account, &oracle, pair_id);

                account.realize();
                pool.realize();
            })
        }

        pub fn update_pair(
            &self, 
            pair_id: PairId,
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool);

                let current_time = Clock::current_time_rounded_to_seconds();
                let max_age = current_time.add_seconds(-(self.config.exchange.max_price_age_seconds)).expect(ERROR_ARITHMETIC);
                let oracle = VirtualOracle::new(self.oracle, self._collateral_feeds(), max_age);

                let rewarded = self._update_pair(&mut pool, &oracle, pair_id);

                pool.realize();

                if rewarded {
                    ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(1)
                } else {
                    Bucket::new(KEEPER_REWARD_RESOURCE)
                }
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
        ) -> HashMap<ResourceAddress, PairId> {
            self.config.collaterals.iter().map(|(resource, config)| (*resource, config.pair_id)).collect()
        }

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
            skew_delta: Decimal,
        ) {
            if self._skew_ratio(pool) >= self.config.exchange.skew_ratio_cap && skew_delta <= dec!(0) {
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
            let collateral_value = self._value_collateral(account, oracle);
            let account_value = pnl + collateral_value + account.virtual_balance();

            if account_value < margin {
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
            
            self._assert_pool_integrity(pool, dec!(0));

            payment
        }

        fn _add_collateral(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            mut tokens: Vec<Bucket>,
        ) {
            if let Some(index) = tokens.iter().position(|token| token.resource_address() == BASE_RESOURCE) {
                let base_token = tokens.remove(index);
                let base_amount = base_token.amount();
                pool.deposit(base_token);
                pool.update_virtual_balance(pool.virtual_balance() - base_amount);
                account.update_virtual_balance(account.virtual_balance() + base_amount);
            }
            for token in tokens.iter() {
                self._assert_valid_collateral(token.resource_address());
            }

            account.deposit_collateral_batch(tokens);
        }
        
        fn _remove_collateral(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            request: RequestRemoveCollateral,
        ) {
            let target_account = request.target_account;
            let mut claims = request.claims;

            let mut tokens = Vec::new();
            claims.retain(|(resource, amount)| {
                if *resource == BASE_RESOURCE {
                    assert!(
                        *amount <= pool.base_tokens_amount(),
                        "{}", ERROR_REMOVE_COLLATERAL_INSUFFICIENT_POOL_TOKENS
                    );

                    let base_token = pool.withdraw(*amount, TO_ZERO);
                    let base_amount = base_token.amount();
                    pool.update_virtual_balance(pool.virtual_balance() + base_amount);
                    account.update_virtual_balance(account.virtual_balance() - base_amount);
                    tokens.push(base_token);
                    false
                } else {
                    true
                }
            });

            let mut target_account = Global::<Account>::try_from(target_account).expect(ERROR_INVALID_ACCOUNT);
            
            tokens.append(&mut account.withdraw_collateral_batch(claims, TO_ZERO));
            target_account.try_deposit_batch_or_abort(tokens, None); // TODO: create authorization badge
            
            self._assert_account_integrity(pool, account, oracle);
        }

        fn _margin_order(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            request: RequestMarginOrder,
        ) {
            let pair_id = request.pair_id;
            let amount = request.amount;
            let price_limit = request.price_limit;

            let current_price = oracle.price(pair_id);
            assert!(
                price_limit.compare(current_price),
                "{}", ERROR_MARGIN_ORDER_PRICE_LIMIT
            );

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

            let skew0 = pool.skew_abs_snap();

            if !amount_close.is_zero() {
                self._close_position(pool, account, oracle, pair_id, amount_close);
            }
            if !amount_open.is_zero() {
                self._open_position(pool, account, oracle, pair_id, amount_open);
            }

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            let skew1 = pool.skew_abs_snap();

            let status_updates = request.active_requests.into_iter().map(|index| (index, STATUS_ACTIVE))
                .chain(request.cancel_requests.into_iter().map(|index| (index, STATUS_CANCELLED)))
                .collect();
            account.try_set_keeper_request_statuses(status_updates);

            self._assert_account_integrity(pool, account, oracle);
            self._assert_pool_integrity(pool, skew1 - skew0);
        }

        fn _swap_debt(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            resource: &ResourceAddress, 
            payment_token: Bucket, 
        ) -> Bucket {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let value = payment_token.amount();
            let virtual_balance = account.virtual_balance();
            
            assert!(
                value <= -virtual_balance,
                "{}", ERROR_SWAP_NOT_ENOUGH_DEBT
            );
            let price_resource = oracle.price_resource(*resource);
            let amount = value / price_resource;
            // TODO: check amount first? take less of two?

            pool.deposit(payment_token);
            pool.update_virtual_balance(pool.virtual_balance() - value);
            account.update_virtual_balance(virtual_balance + value);
            let collateral = account.withdraw_collateral_batch(vec![(*resource, amount)], TO_ZERO).pop().unwrap();

            collateral
        }

        fn _liquidate(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            mut payment_token: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let (pnl, margin, fee_referral) = self._liquidate_positions(pool, account, oracle);
            let (collateral_value, mut collateral_tokens) = self._liquidate_collateral(account, oracle);
            let account_value = pnl + collateral_value + account.virtual_balance();

            assert!(
                account_value < margin,
                "{}", ERROR_LIQUIDATION_SUFFICIENT_MARGIN
            );
            
            let deposit_token = payment_token.take_advanced(collateral_value, TO_INFINITY);
            let value = payment_token.amount();

            pool.deposit(deposit_token);
            pool.update_virtual_balance(pool.virtual_balance() - value);
            account.update_virtual_balance(account.virtual_balance() + value);
            
            self._settle_with_pool(pool, account, pnl);
            self._settle_with_referrals(pool, account, fee_referral);
            // TODO: insurance fund for outstanding_base
            // TODO: unsettled balance vs collateral value movement leaving debt

            account.update_last_liquidation_index();

            collateral_tokens.push(payment_token);
            collateral_tokens
        }

        fn _auto_deleverage(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: PairId, 
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

            if amount.is_zero() {
                panic!("{}", ERROR_ADL_NO_POSITION);
            }

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
            self._close_position(pool, account, oracle, pair_id, amount_close);

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
            pair_id: PairId, 
            amount: Decimal, 
        ) {
            let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = oracle.price(pair_id);

            let value = amount * price_token;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);
            
            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short + amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_delta = skew_abs_1 - skew_abs_0;
            let fee = value_abs * (config.fee_0 + skew_abs_delta / pool_value * config.fee_1).clamp(dec!(0), self.config.exchange.fee_max);
            let fee_referral = fee * self.config.exchange.fee_share_referral;
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

            self._settle_with_referrals(pool, account, fee_referral);

            self._assert_position_limit(account);
        }

        fn _close_position(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: PairId, 
            amount: Decimal, 
        ) {
            let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
            let price_token = oracle.price(pair_id);
            
            let value = amount * price_token;
            let value_abs = value.checked_abs().unwrap();
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);

            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_delta = skew_abs_1 - skew_abs_0;
            let fee = value_abs * (config.fee_0 + skew_abs_delta / pool_value * config.fee_1).clamp(dec!(0), self.config.exchange.fee_max);
            let fee_referral = fee * self.config.exchange.fee_share_referral;
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

            self._settle_with_pool(pool, account, pnl);
            self._settle_with_referrals(pool, account, fee_referral);
        }

        fn _update_pair(
            &self, 
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: PairId,
        ) -> bool {
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
            
            let current_time = Clock::current_time_rounded_to_seconds();
            let period_seconds = current_time.seconds_since_unix_epoch - pool_position.last_update.seconds_since_unix_epoch;
            let period = Decimal::from(period_seconds);
            
            if !period.is_zero() {
                let funding_2_rate_delta = skew * config.funding_2_delta * period;
                pool_position.funding_2_rate += funding_2_rate_delta;

                if !oi_long.is_zero() && !oi_short.is_zero() {
                    let funding_1_rate = skew * config.funding_1;
                    let funding_2_rate = pool_position.funding_2_rate * config.funding_2;
                    let funding_rate = funding_1_rate + funding_2_rate;

                    let (funding_long_index, funding_short_index, funding_share) = if funding_rate.is_positive() {
                        let funding_long = funding_rate * period;
                        let funding_long_index = funding_long / oi_long;
        
                        let funding_share = funding_long * config.funding_share;
                        let funding_short_index = -(funding_long - funding_share) / oi_short;
        
                        (funding_long_index, funding_short_index, funding_share)
                    } else {
                        let funding_short = -funding_rate * period * price_token;
                        let funding_short_index = funding_short / oi_short;
        
                        let funding_share = funding_short * config.funding_share;
                        let funding_long_index = -(funding_short - funding_share) / oi_long;
        
                        (funding_long_index, funding_short_index, funding_share)
                    };

                    let funding_pool_0_rate = (oi_long + oi_short) * price_token * config.funding_pool_0;
                    let funding_pool_1_rate = skew_abs * config.funding_pool_1;
                    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;

                    let funding_pool = funding_pool_rate * period;
                    let funding_pool_index = funding_pool / (oi_long + oi_short);
                    pool.update_unrealized_pool_funding(pool.unrealized_pool_funding() + funding_pool + funding_share);

                    pool_position.funding_long_index += funding_long_index + funding_pool_index;
                    pool_position.funding_short_index += funding_short_index + funding_pool_index;
                }
            }

            let price_delta_ratio = (price_token - pool_position.last_price).checked_abs().expect(ERROR_ARITHMETIC) / pool_position.last_price;
            pool_position.last_price = price_token;
            pool_position.last_update = current_time;

            pool.update_position(pair_id, pool_position);

            if period_seconds >= self.config.exchange.pair_update_period_seconds || 
            price_delta_ratio >= self.config.exchange.pair_update_price_delta_ratio {
                true
            } else {
                false
            }
        }

        fn _update_pair_snaps(
            &self, 
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: PairId,
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

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_delta = skew_abs_1 - skew_abs_0;
                let fee = value_abs * (config.fee_0 + skew_abs_delta / pool_value * config.fee_1).clamp(dec!(0), self.config.exchange.fee_max);
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
        ) -> (Decimal, Decimal, Decimal) {
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            let mut total_fee_referral = dec!(0);
            for (&pair_id, position) in account.positions().clone().iter() {
                let config = self.config.pairs.get(&pair_id).expect(ERROR_MISSING_PAIR_CONFIG);
                let price_token = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let mut pool_position = pool.position(pair_id);

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_delta = skew_abs_1 - skew_abs_0;
                let fee = value_abs * (config.fee_0 + skew_abs_delta / pool_value * config.fee_1).clamp(dec!(0), self.config.exchange.fee_max);
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
                total_fee_referral += fee * self.config.exchange.fee_share_referral;

                pool.update_position(pair_id, pool_position);
                account.remove_position(pair_id);
            }

            (total_pnl, total_margin, total_fee_referral)
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
            pool.update_virtual_balance(pool.virtual_balance() - amount);
            account.update_virtual_balance(account.virtual_balance() + amount);
        }

        fn _settle_funding(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: PairId,
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

        fn _settle_with_referrals(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            fee_referral: Decimal,
        ) {
            let fee_referral = fee_referral.min(pool.base_tokens_amount());
            let fee_referral_tokens = pool.withdraw(fee_referral, TO_ZERO);
            self.referrals.reward(account.address(), fee_referral_tokens);
        }
        
        fn _save_funding_index(
            &self,
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: PairId,
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
