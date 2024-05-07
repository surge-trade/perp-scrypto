mod errors;
mod events;
mod requests;
mod results;
mod virtual_config;
mod virtual_margin_pool;
mod virtual_margin_account;
mod virtual_oracle;

use scrypto::prelude::*;
use common::{PairId, ListIndex, _AUTHORITY_RESOURCE, _BASE_RESOURCE, _KEEPER_REWARD_RESOURCE, TO_ZERO, TO_INFINITY};
use account::*;
use pool::*;
use config::*;
use self::errors::*;
use self::events::*;
use self::requests::*;
use self::results::*;
use self::virtual_config::*;
use self::virtual_margin_pool::*;
use self::virtual_margin_account::*;
use self::virtual_oracle::*;

#[blueprint]
#[events(
    EventSignalUpgrade,
    EventExchangeConfigUpdate,
    EventPairConfigUpdates,
    EventCollateralConfigUpdates,
    EventCollateralConfigRemoval,
    EventPairUpdates,
    EventAccountCreation,
    EventRequests,
    EventValidRequestsStart,
    EventAddLiquidity,
    EventRemoveLiquidity,
    EventAddCollateral,
    EventRemoveCollateral,
    EventMarginOrder,
    EventSwapDebt,
    EventLiquidate,
    EventAutoDeleverage,
)]
mod exchange_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;
    const KEEPER_REWARD_RESOURCE: ResourceAddress = _KEEPER_REWARD_RESOURCE;

    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Config {
            // Constructor
            fn new(initial_rule: AccessRule) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self) -> ConfigInfoCompressed;
            fn get_pair_config_len(&self) -> ListIndex;
            fn get_pair_config(&self, pair_id: PairId) -> Option<PairConfigCompressed>;
            fn get_pair_configs(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PairConfigCompressed>>;
            fn get_pair_config_range(&self, start: ListIndex, end: ListIndex) -> Vec<PairConfigCompressed>;

            // Authority protected methods
            fn update_exchange_config(&mut self, config: ExchangeConfig);
            fn update_pair_configs(&mut self, configs: Vec<PairConfig>);
            fn update_collateral_configs(&mut self, configs: Vec<(ResourceAddress, CollateralConfig)>);
            fn remove_collateral_config(&mut self, resource: ResourceAddress);
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        MarginAccount {
            // Constructor
            fn new(initial_rule: AccessRule, reservation: Option<GlobalAddressReservation>) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self, collateral_resources: Vec<ResourceAddress>) -> MarginAccountInfo;
            fn get_request(&self, index: ListIndex) -> Option<KeeperRequest>;
            fn get_requests(&self, start: ListIndex, end: ListIndex) -> Vec<KeeperRequest>;
            fn get_requests_tail(&self, num: ListIndex) -> Vec<KeeperRequest>;
            fn get_requests_by_indexes(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, Option<KeeperRequest>>;
            fn get_requests_len(&self) -> ListIndex;
            fn get_active_requests(&self) -> HashMap<ListIndex, KeeperRequest>;

            // Authority protected methods
            fn update(&self, update: MarginAccountUpdates);
            fn deposit_collateral_batch(&self, tokens: Vec<Bucket>);
            fn withdraw_collateral_batch(&self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>;
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        MarginPool {
            // Getter methods
            fn get_info(&self, pair_ids: HashSet<PairId>) -> MarginPoolInfo;
            fn get_position(&self, pair_id: PairId) -> Option<PoolPosition>;
            fn get_positions(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PoolPosition>>;     

            // Authority protected methods
            fn update(&self, update: MarginPoolUpdates);
            fn deposit(&self, token: Bucket);
            fn withdraw(&self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;            
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Oracle {
            // Public methods
            fn push_and_get_prices(&self, pair_ids: HashSet<PairId>, max_age: Instant, data: Vec<u8>, signature: Bls12381G2Signature) -> HashMap<PairId, Decimal>;
            fn get_prices(&self, pair_ids: HashSet<PairId>, max_age: Instant) -> HashMap<PairId, Decimal>;
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        FeeDistributor {
            // Getter methods
            fn get_referrer(&self, account: ComponentAddress) -> Option<ComponentAddress>;
            fn get_rebate(&self) -> Decimal;
            fn get_trickle_up(&self) -> Decimal;
            fn get_protocol_virtual_balance(&self) -> Decimal;
            fn get_treasury_virtual_balance(&self) -> Decimal;

            // Authority protected methods
            fn update_rebate(&self, rebate: Decimal);
            fn update_trickle_up(&self, trickle_up: Decimal);
            fn update_protocol_virtual_balance(&self, protocol_virtual_balance: Decimal);
            fn update_treasury_virtual_balance(&self, treasury_virtual_balance: Decimal);
            fn set_referrer(&self, account: ComponentAddress, referrer: Option<ComponentAddress>);
            fn reward(&self, amount_protocol: Decimal, amount_treasury: Decimal, amount_referral: Decimal, referred_account: ComponentAddress);
            fn collect(&self, account: ComponentAddress) -> Decimal;
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        FeeDelegator {
            // Getter methods
            fn get_fee_oath_resource(&self) -> ResourceAddress;
            fn get_max_lock(&self) -> Decimal;
            fn get_is_contingent(&self) -> bool;
            fn get_vault_amount(&self) -> Decimal;
            fn get_virtual_balance(&self) -> Decimal;

            // Authority protected methods
            fn update_max_lock(&self, max_lock: Decimal);
            fn update_is_contingent(&self, is_contingent: bool);
            fn update_virtual_balance(&self, virtual_balance: Decimal);
            fn deposit(&self, token: Bucket);
            fn withdraw(&self, amount: Decimal) -> Bucket;

            // User methods
            fn lock_fee(&self, amount: Decimal) -> Bucket;
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
            signal_upgrade => restrict_to: [OWNER];
            update_exchange_config => restrict_to: [OWNER];
            update_pair_configs => restrict_to: [OWNER];
            update_collateral_configs => restrict_to: [OWNER];
            remove_collateral_config => restrict_to: [OWNER];
            update_referral_rebate => restrict_to: [OWNER];
            update_referral_trickle_up => restrict_to: [OWNER];
            update_max_fee_delegator_lock => restrict_to: [OWNER];
            update_fee_delegator_is_contingent => restrict_to: [OWNER];
            collect_fee_distributor_balance => restrict_to: [OWNER];
            collect_fee_delegator_balance => restrict_to: [OWNER];
            deposit_fee_delegator => restrict_to: [OWNER];
            withdraw_fee_delegator => restrict_to: [OWNER];

            // Get methods
            get_exchange_config => PUBLIC;
            get_pairs_len => PUBLIC;
            get_pair_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_pair_config_range => PUBLIC;
            get_collateral_config => PUBLIC;
            get_collateral_configs => PUBLIC;
            get_collaterals => PUBLIC;
            get_pool_value => PUBLIC;
            get_skew_ratio => PUBLIC;
            get_referrer => PUBLIC;
            get_protocol_balance => PUBLIC;

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
            cancel_all_requests => restrict_to: [user];

            // Keeper methods
            process_request => restrict_to: [keeper];
            swap_debt => restrict_to: [keeper];
            liquidate => restrict_to: [keeper];
            auto_deleverage => restrict_to: [keeper];
            update_pair => restrict_to: [keeper];
            swap_protocol_fee => restrict_to: [keeper];
        }
    }

    macro_rules! authorize {
        ($self:expr, $func:expr) => {{
            $self.authority_token.authorize_with_amount(dec!(0.000000000000000001),|| {
                $func
            })
        }};
    }

    struct Exchange {
        authority_token: FungibleVault,
        config: Global<Config>,
        pool: Global<MarginPool>,
        oracle: Global<Oracle>,
        fee_distributor: Global<FeeDistributor>,
        fee_delegator: Global<FeeDelegator>,
        fee_oath_resource: ResourceAddress,
    }
    
    impl Exchange {
        pub fn new(
            owner_role: OwnerRole,
            authority_token: Bucket,
            config: ComponentAddress,
            pool: ComponentAddress,
            oracle: ComponentAddress,
            fee_distributor: ComponentAddress,
            fee_delegator: ComponentAddress,
        ) -> Global<Exchange> {
            assert!(
                authority_token.resource_address() == AUTHORITY_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_AUTHORITY, Runtime::bech32_encode_address(authority_token.resource_address()), Runtime::bech32_encode_address(AUTHORITY_RESOURCE)
            );

            let config: Global<Config> = config.into();
            let pool: Global<MarginPool> = pool.into();
            let oracle: Global<Oracle> = oracle.into();
            let fee_distributor: Global<FeeDistributor> = fee_distributor.into();
            let fee_delegator: Global<FeeDelegator> = fee_delegator.into();
            let fee_oath_resource = fee_delegator.get_fee_oath_resource();

            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                config,
                pool,
                oracle,
                fee_distributor,
                fee_delegator,
                fee_oath_resource

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

        pub fn signal_upgrade(
            &self, 
            new_exchange: ComponentAddress,
        ) {
            Runtime::emit_event(EventSignalUpgrade {
                new_exchange,
            });
        }

        // --- ADMIN METHODS ---

        pub fn update_exchange_config(
            &mut self, 
            config: ExchangeConfig,
        ) {
            authorize!(self, {
                self.config.update_exchange_config(config.clone());
            });
                
            Runtime::emit_event(EventExchangeConfigUpdate {
                config,
            });
        }

        pub fn update_pair_configs(
            &mut self, 
            configs: Vec<PairConfig>,
        ) {
            authorize!(self, {
                self.config.update_pair_configs(configs.clone());
            });

            Runtime::emit_event(EventPairConfigUpdates {
                configs,
            });
        }

        pub fn update_collateral_configs(
            &mut self, 
            configs: Vec<(ResourceAddress, CollateralConfig)>,
        ) {
            authorize!(self, {
                self.config.update_collateral_configs(configs.clone());
            });

            Runtime::emit_event(EventCollateralConfigUpdates {
                configs
            });
        }

        pub fn remove_collateral_config(
            &mut self, 
            resource: ResourceAddress,
        ) {
            authorize!(self, {
                self.config.remove_collateral_config(resource);
            });

            Runtime::emit_event(EventCollateralConfigRemoval {
                resource
            });
        }

        pub fn update_referral_rebate(
            &self, 
            rebate: Decimal,
        ) {
            authorize!(self, {
                self.fee_distributor.update_rebate(rebate);
            })
        }

        pub fn update_referral_trickle_up(
            &self, 
            trickle_up: Decimal,
        ) {
            authorize!(self, {
                self.fee_distributor.update_trickle_up(trickle_up);
            })
        }

        pub fn update_max_fee_delegator_lock(
            &self, 
            max_lock: Decimal,
        ) {
            authorize!(self, {
                self.fee_delegator.update_max_lock(max_lock);
            })
        }

        pub fn update_fee_delegator_is_contingent(
            &self, 
            is_contingent: bool,
        ) {
            authorize!(self, {
                self.fee_delegator.update_is_contingent(is_contingent);
            })
        }

        pub fn collect_fee_distributor_balance(
            &self, 
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = self.fee_distributor.get_treasury_virtual_balance();

                self.fee_distributor.update_treasury_virtual_balance(dec!(0));
                pool.update_virtual_balance(pool.virtual_balance() + amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn collect_fee_delegator_balance(
            &self, 
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = self.fee_delegator.get_virtual_balance();

                self.fee_delegator.update_virtual_balance(dec!(0));
                pool.update_virtual_balance(pool.virtual_balance() + amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn deposit_fee_delegator(
            &self, 
            token: Bucket,
        ) {
            authorize!(self, {
                self.fee_delegator.deposit(token);
            })
        }

        pub fn withdraw_fee_delegator(
            &self, 
            amount: Decimal,
        ) -> Bucket {
            authorize!(self, {
                self.fee_delegator.withdraw(amount)
            })
        }

        // --- GET METHODS ---

        pub fn get_exchange_config(
            &self
        ) -> ExchangeConfig {
            self.config.get_info().exchange.decompress()
        }

        pub fn get_pairs_len(
            &self,
        ) -> ListIndex {
            self.config.get_pair_config_len()
        }

        pub fn get_pair_config(
            &self, 
            pair_id: PairId,
        ) -> PairConfig {
            self.config.get_pair_config(pair_id).expect(ERROR_MISSING_PAIR_CONFIG).decompress()
        }

        pub fn get_pair_configs(
            &self, 
            pair_ids: HashSet<PairId>,
        ) -> HashMap<PairId, PairConfig> {
            self.config.get_pair_configs(pair_ids).into_iter()
                .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG).decompress())).collect()
        }

        pub fn get_pair_config_range(
            &self, 
            start: ListIndex, 
            end: ListIndex,
        ) -> Vec<PairConfig> {
            self.config.get_pair_config_range(start, end).into_iter().map(|v| v.decompress()).collect()
        }

        pub fn get_collateral_config(
            &self, 
            resource: ResourceAddress,
        ) -> CollateralConfig {
            self.config.get_info().collaterals.into_iter()
                .find(|(k, _)| *k == resource).expect(ERROR_COLLATERAL_INVALID).1.decompress()
        }

        pub fn get_collateral_configs(
            &self, 
        ) -> HashMap<ResourceAddress, CollateralConfig> {
            self.config.get_info().collaterals.into_iter()
                .map(|(k, v)| (k, v.decompress())).collect()
        }

        pub fn get_collaterals(
            &self,
        ) -> Vec<ResourceAddress> {
            self.config.get_info().collaterals.keys().cloned().collect()
        }

        pub fn get_pool_value(
            &self,
        ) -> Decimal {
            let pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
            self._pool_value(&pool)
        }

        pub fn get_skew_ratio(
            &self,
        ) -> Decimal {
            let pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
            self._skew_ratio(&pool)
        }

        pub fn get_referrer(
            &self, 
            account: ComponentAddress,
        ) -> Option<ComponentAddress> {
            self.fee_distributor.get_referrer(account)
        }

        pub fn get_protocol_balance(
            &self,
        ) -> Decimal {
            self.fee_distributor.get_protocol_virtual_balance()
        }

        // --- USER METHODS ---

        pub fn create_account(
            &self, 
            initial_rule: AccessRule,
            reservation: Option<GlobalAddressReservation>,
        ) -> Global<MarginAccount> {
            let account = authorize!(self, {
                Blueprint::<MarginAccount>::new(initial_rule, reservation)
            });

            Runtime::emit_event(EventAccountCreation {
                account: account.address(),
            });

            account
        }

        pub fn set_level_1_auth(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);
                account.verify_level_1_auth();
                account.set_level_1_auth(rule);
                account.realize();
            })
        }

        pub fn set_level_2_auth(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);
                account.verify_level_1_auth();
                account.set_level_2_auth(rule);
                account.realize();
            })
        }

        pub fn set_level_3_auth(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);
                account.verify_level_1_auth();
                account.set_level_3_auth(rule);
                account.realize();
            })
        }

        // TODO: Don't allow setting referrer if already set?
        pub fn set_referrer(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            referrer: ComponentAddress,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let account = self._handle_fee_oath(account, &config, fee_oath);
                account.verify_level_2_auth();

                let current_referrer = self.fee_distributor.get_referrer(account.address());
                assert!(
                    current_referrer.is_none(),
                    "{}, VALUE:{:?}, REQUIRED:None, OP:== |", ERROR_REFERRAL_ALREADY_SET, current_referrer,
                );

                Global::<MarginAccount>::try_from(referrer).expect(ERROR_INVALID_ACCOUNT);

                self.fee_distributor.set_referrer(account.address(), Some(referrer));
                account.realize();
            })
        }

        pub fn collect_referral_rewards(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
        ) -> Bucket {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let account = self._handle_fee_oath(account, &config, fee_oath);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                account.verify_level_2_auth();

                let amount = self.fee_distributor.collect(account.address());
                let payment = pool.withdraw(amount, TO_ZERO);

                account.realize();
                pool.realize();
                payment
            })
        }

        pub fn add_liquidity(
            &self,
            payment: Bucket,
        ) -> Bucket {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let lp_token = self._add_liquidity(&config, &mut pool, payment);
                pool.realize();

                lp_token
            })
        }

        pub fn remove_liquidity(
            &self,
            lp_token: Bucket,
        ) -> Bucket {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let token = self._remove_liquidity(&config, &mut pool, lp_token);
                pool.realize();

                token
            })
        }

        pub fn add_collateral(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            tokens: Vec<Bucket>,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());

                self._add_collateral(&config, &mut pool, &mut account, tokens);

                pool.realize();
                account.realize();
            })
        }

        pub fn remove_collateral_request(
            &self,
            fee_oath: Option<Bucket>,
            expiry_seconds: u64, 
            account: ComponentAddress, 
            target_account: ComponentAddress,
            claims: Vec<(ResourceAddress, Decimal)>,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);

                account.verify_level_2_auth();

                let request = Request::RemoveCollateral(RequestRemoveCollateral {
                    target_account,
                    claims,
                });
                account.push_request(request, expiry_seconds, STATUS_ACTIVE);
                self._assert_active_requests_limit(&config, &account);

                account.realize();
            })
        }

        pub fn margin_order_request(
            &self,
            fee_oath: Option<Bucket>,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: PairId,
            amount: Decimal,
            price_limit: Limit,
            activate_requests: Vec<ListIndex>,
            cancel_requests: Vec<ListIndex>,
            status: Status,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);

                account.verify_level_3_auth();

                let request = Request::MarginOrder(RequestMarginOrder {
                    pair_id,
                    amount,
                    price_limit,
                    activate_requests,
                    cancel_requests,
                });
                account.push_request(request, expiry_seconds, status);
                self._assert_active_requests_limit(&config, &account);

                account.realize();
            })
        }

        pub fn cancel_request(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            index: ListIndex,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);

                account.verify_level_3_auth();
                account.cancel_request(index);
                account.realize();
            })
        }

        pub fn cancel_all_requests(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);

                account.verify_level_3_auth();
                account.update_valid_requests_start();

                account.realize();
            })
        }

        // --- KEEPER METHODS ---

        pub fn process_request(
            &self, 
            account: ComponentAddress, 
            index: ListIndex,
            update_data: String,
            update_signature: String,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let mut account = VirtualMarginAccount::new(account, config.collaterals());
                let (request, submission) = account.process_request(index);
                
                let mut pair_ids = account.position_ids();
                if let Request::MarginOrder(request) = &request {
                    pair_ids.insert(request.pair_id.clone());
                }
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let max_age = if max_age.compare(submission, TimeComparisonOperator::Gt) {
                    max_age
                } else {
                    submission
                };
                
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, Some((update_data, update_signature)));

                match request {
                    Request::RemoveCollateral(request) => {
                        self._remove_collateral(&config, &mut pool, &mut account, &oracle, request);
                    },
                    Request::MarginOrder(request) => {
                        self._margin_order(&config, &mut pool, &mut account, &oracle, request);
                    },
                };

                account.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(100); // TODO: Set reward amount
                reward
            })
        }

        pub fn swap_debt(
            &self, 
            account: ComponentAddress, 
            resource: ResourceAddress, 
            payment: Bucket, 
        ) -> (Bucket, Bucket) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                self._assert_valid_collateral(&config, resource);
                let collaterals = vec![resource];
                let mut account = VirtualMarginAccount::new(account, collaterals);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                
                let max_age = self._max_age(&config);
                let collateral_feeds = HashMap::from([(resource, config.collateral_feeds().get(&resource).unwrap().clone())]);
                let oracle = VirtualOracle::new(self.oracle, collateral_feeds, HashSet::new(), max_age, None);

                let (token, remainder) = self._swap_debt(&mut pool, &mut account, &oracle, &resource, payment);
    
                account.realize();
                pool.realize();

    
                (token, remainder)
            })
        }

        pub fn liquidate(
            &self,
            account: ComponentAddress,
            payment: Bucket,
            update_data: String,
            update_signature: String,
        ) -> (Vec<Bucket>, Bucket) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let account_component = account;
                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let pair_ids = account.position_ids();
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, Some((update_data, update_signature)));

                let result_liquidate = self._liquidate(&config, &mut pool, &mut account, &oracle, payment);

                account.realize();
                pool.realize();

                let fee_keeper = dec!(100); // TODO: Set fee amount
                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(1000); // TODO: Set reward amount

                Runtime::emit_event(EventLiquidate {
                    account: account_component,
                    position_amounts: result_liquidate.position_amounts,
                    collateral_amounts: result_liquidate.collateral_amounts,
                    collateral_value: result_liquidate.collateral_value,
                    margin: result_liquidate.margin,
                    fee_pool: result_liquidate.fee_pool,
                    fee_protocol: result_liquidate.fee_protocol,
                    fee_treasury: result_liquidate.fee_treasury,
                    fee_referral: result_liquidate.fee_referral,
                    fee_keeper,
                    position_prices: result_liquidate.position_prices,
                    collateral_prices: result_liquidate.collateral_prices,
                });

                (result_liquidate.tokens, reward)
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: PairId, 
            update_data: String,
            update_signature: String,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let account_component = account;
                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let mut pair_ids = account.position_ids();
                pair_ids.insert(pair_id.clone());
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, Some((update_data, update_signature)));

                let result_auto_deleverage = self._auto_deleverage(&config, &mut pool, &mut account, &oracle, &pair_id);

                account.realize();
                pool.realize();

                let fee_keeper = dec!(100); // TODO: Set fee amount
                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(200); // TODO: Set reward amount

                Runtime::emit_event(EventAutoDeleverage {
                    account: account_component,
                    pair_id,
                    amount: result_auto_deleverage.amount,
                    pnl_percent: result_auto_deleverage.pnl_percent,
                    threshold: result_auto_deleverage.threshold,
                    fee_pool: result_auto_deleverage.fee_pool,
                    fee_protocol: result_auto_deleverage.fee_protocol,
                    fee_treasury: result_auto_deleverage.fee_treasury,
                    fee_referral: result_auto_deleverage.fee_referral,
                    fee_keeper,
                    price: result_auto_deleverage.price,
                });

                reward
            })
        }

        pub fn update_pair(
            &self, 
            pair_id: PairId,
            update_data: String,
            update_signature: String,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);

                let pair_ids = HashSet::from([pair_id.clone()]);
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, Some((update_data, update_signature)));

                let rewarded = self._update_pair(&config, &mut pool, &oracle, &pair_id);

                pool.realize();

                if rewarded {
                    ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(1)
                } else {
                    Bucket::new(KEEPER_REWARD_RESOURCE)
                }
            })
        }

        pub fn swap_protocol_fee(&self, mut payment: Bucket) -> (Bucket, Bucket) {
            authorize!(self, {
                assert!(
                    payment.resource_address() == KEEPER_REWARD_RESOURCE, // TODO: Change to protocol fee resource
                    "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT, Runtime::bech32_encode_address(payment.resource_address()), Runtime::bech32_encode_address(KEEPER_REWARD_RESOURCE)
                );

                payment.take(dec!(1)).burn(); // TODO: Set fee amount
                
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = self.fee_distributor.get_protocol_virtual_balance();

                self.fee_distributor.update_protocol_virtual_balance(dec!(0));
                pool.update_virtual_balance(pool.virtual_balance() + amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                (token, payment)
            })
        }

        // --- INTERNAL METHODS ---

        fn _max_age(
            &self,
            config: &VirtualConfig,
        ) -> Instant {
            let current_time = Clock::current_time_rounded_to_seconds();
            current_time.add_seconds(-(config.exchange_config().max_price_age_seconds)).expect(ERROR_ARITHMETIC)
        }

        fn _collaterals(
            &self,
            config: &VirtualConfig,
        ) -> Vec<ResourceAddress> {
            config.collaterals()
        }

        fn _collateral_feeds(
            &self,
            config: &VirtualConfig,
        ) -> HashMap<ResourceAddress, PairId> {
            config.collateral_feeds()
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
            pool.skew_abs_snap() / self._pool_value(pool).max(dec!(1))
        }

        fn _assert_pool_integrity(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            skew_delta: Decimal,
        ) {
            let skew_ratio = self._skew_ratio(pool);
            let skew_ratio_cap = config.exchange_config().skew_ratio_cap;
            assert!(
                skew_ratio < skew_ratio_cap || skew_delta < dec!(0),
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_SKEW_TOO_HIGH, skew_ratio, skew_ratio_cap
            );
        }

        fn _assert_account_integrity(
            &self, 
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount,
            oracle: &VirtualOracle,
        ) {
            let (pnl, margin_positions) = self._value_positions(config, pool, account, oracle);
            let (collateral_value_discounted, margin_collateral) = self._value_collateral(config, account, oracle);
            let account_value = pnl + collateral_value_discounted + account.virtual_balance();
            let margin = margin_positions + margin_collateral;

            assert!(
                account_value > margin,
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_INSUFFICIENT_MARGIN, account_value, margin
            );
        }

        fn _assert_valid_collateral(
            &self, 
            config: &VirtualConfig,
            resource: ResourceAddress,
        ) {
            assert!(
                config.collateral_configs().contains_key(&resource),
                "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_COLLATERAL_INVALID, Runtime::bech32_encode_address(resource), 
                config.collateral_configs().keys().map(|r| Runtime::bech32_encode_address(*r)).collect::<Vec<String>>()
            );
        }

        fn _assert_position_limit(
            &self, 
            config: &VirtualConfig,
            account: &VirtualMarginAccount,
        ) {
            let positions_len = account.positions().len();
            let positions_max = config.exchange_config().positions_max as usize;
            assert!(
                positions_len <= positions_max,
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_POSITIONS_TOO_MANY, positions_len, positions_max
            );
        }

        fn _assert_active_requests_limit(
            &self, 
            config: &VirtualConfig,
            account: &VirtualMarginAccount,
        ) {
            let active_requests_len = account.active_requests_len();
            let active_requests_max = config.exchange_config().active_requests_max as usize;
            assert!(
                active_requests_len <= active_requests_max,
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_ACTIVE_REQUESTS_TOO_MANY, active_requests_len, active_requests_max
            );
        }

        fn _handle_fee_oath(
            &self,
            account: ComponentAddress,
            config: &VirtualConfig,
            fee_oath: Option<Bucket>,
        ) -> VirtualMarginAccount {
            if let Some(fee_oath) = fee_oath {
                let mut account = VirtualMarginAccount::new(account, config.collaterals());
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), HashSet::new(), Instant::new(0), None);
                self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                account
            } else {
                VirtualMarginAccount::new(account, vec![])
            }
        }

        fn _settle_fee_oath(
            &self,
            config: &VirtualConfig,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            fee_oath: Bucket,
        ) {
            let resource = fee_oath.resource_address();
            assert!(
                fee_oath.resource_address() == self.fee_oath_resource,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_FEE_OATH, Runtime::bech32_encode_address(resource), Runtime::bech32_encode_address(self.fee_oath_resource)
            );

            let fee_value = fee_oath.amount();
            let (collateral_value_discounted, margin_collateral) = self._value_collateral(config, account, oracle);
            let collateral_value_approx = collateral_value_discounted + account.virtual_balance() - fee_value;

            assert!(
                collateral_value_approx > margin_collateral,
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_INSUFFICIENT_MARGIN, collateral_value_approx, margin_collateral
            );

            account.update_virtual_balance(account.virtual_balance() - fee_value);
            fee_oath.burn();
        }

        fn _add_liquidity(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            payment: Bucket,
        ) -> Bucket {
            assert!(
                payment.resource_address() == BASE_RESOURCE,
                "{}", ERROR_INVALID_PAYMENT
            );

            let value = payment.amount();
            let fee = value * config.exchange_config().fee_liquidity;
            let pool_value = self._pool_value(pool).max(dec!(1));
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();

            let (lp_amount, lp_price) = if lp_supply.is_zero() {
                (value, dec!(1))
            } else {
                let lp_price = pool_value / lp_supply;
                let lp_amount = (value - fee) / lp_price;
                (lp_amount, lp_price)
            };

            pool.deposit(payment);
            let lp_token = pool.mint_lp(lp_amount);

            Runtime::emit_event(EventAddLiquidity {
                amount: value,
                lp_amount,
                lp_price,
                fee_liquidity: fee,
            });

            lp_token
        }

        fn _remove_liquidity(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            lp_token: Bucket,
        ) -> Bucket {
            assert!(
                lp_token.resource_address() == pool.lp_token_manager().address(),
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_LP_TOKEN, Runtime::bech32_encode_address(lp_token.resource_address()), Runtime::bech32_encode_address(pool.lp_token_manager().address())
            );

            let lp_amount = lp_token.amount();
            let pool_value = self._pool_value(pool).max(dec!(0));
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();

            let lp_price = pool_value / lp_supply;
            let value = lp_amount * lp_price;
            let fee = value * config.exchange_config().fee_liquidity;
            let amount = value - fee;

            pool.burn_lp(lp_token);
            let token = pool.withdraw(amount, TO_ZERO);
            
            self._assert_pool_integrity(config, pool, dec!(0));

            Runtime::emit_event(EventRemoveLiquidity {
                amount,
                lp_amount,
                lp_price,
                fee_liquidity: fee,
            });

            token
        }

        fn _add_collateral(
            &self,
            config: &VirtualConfig, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            mut tokens: Vec<Bucket>,
        ) {
            if let Some(index) = tokens.iter().position(|token| token.resource_address() == BASE_RESOURCE) {
                let base_token = tokens.remove(index);
                let value = base_token.amount();
                pool.deposit(base_token);
                self._settle_account(pool, account, value);
            }
            let amounts = tokens.iter().map(|token| {
                self._assert_valid_collateral(config, token.resource_address());
                (token.resource_address(), token.amount())
            }).collect();

            account.deposit_collateral_batch(tokens);

            Runtime::emit_event(EventAddCollateral {
                account: account.address(),
                amounts,
            });
        }
        
        fn _remove_collateral(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            request: RequestRemoveCollateral,
        ) {
            let target_account_component = request.target_account;
            let mut claims = request.claims.clone();

            let mut tokens = Vec::new();
            claims.retain(|(resource, amount)| {
                if *resource == BASE_RESOURCE {
                    assert!(
                        *amount <= pool.base_tokens_amount(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_REMOVE_COLLATERAL_INSUFFICIENT_POOL_TOKENS, *amount, pool.base_tokens_amount()
                    );

                    let base_token = pool.withdraw(*amount, TO_ZERO);
                    let value = base_token.amount();
                    self._settle_account(pool, account, -value);
                    tokens.push(base_token);
                    false
                } else {
                    true
                }
            });
            tokens.append(&mut account.withdraw_collateral_batch(claims, TO_ZERO));
            
            let mut target_account = Global::<Account>::try_from(target_account_component).expect(ERROR_INVALID_ACCOUNT);
            target_account.try_deposit_batch_or_abort(tokens, Some(ResourceOrNonFungible::Resource(AUTHORITY_RESOURCE)));
            
            self._assert_account_integrity(config, pool, account, oracle);

            Runtime::emit_event(EventRemoveCollateral {
                account: account.address(),
                target_account: target_account_component,
                amounts: request.claims,
                fee_keeper: dec!(0), // TODO
            });
        }

        fn _margin_order(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            request: RequestMarginOrder,
        ) {
            let pair_id = &request.pair_id;
            let amount = request.amount;
            let price_limit = request.price_limit;

            let price = oracle.price(pair_id);
            assert!(
                price_limit.compare(price),
                "{}, VALUE:{}, REQUIRED:{}, OP:{} |", ERROR_MARGIN_ORDER_PRICE_LIMIT, price, price_limit.price(), price_limit.op()
            );

            self._update_pair(config, pool, oracle, pair_id);
            self._settle_funding(pool, account, pair_id);
                
            let (amount_close, amount_open) = {
                let position_amount = account.positions().get(pair_id).map_or(dec!(0), |p| p.amount);
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

            let skew_0 = pool.skew_abs_snap();

            let mut fee_pool = dec!(0);
            let mut fee_protocol = dec!(0);
            let mut fee_treasury = dec!(0);
            let mut fee_referral = dec!(0);
            if !amount_close.is_zero() {
                let result_close = self._close_position(config, pool, account, oracle, pair_id, amount_close);
                fee_pool += result_close.fee_pool;
                fee_protocol += result_close.fee_protocol;
                fee_treasury += result_close.fee_treasury;
                fee_referral += result_close.fee_referral;
            }
            if !amount_open.is_zero() {
                let result_open = self._open_position(config, pool, account, oracle, pair_id, amount_open);
                fee_pool += result_open.fee_pool;
                fee_protocol += result_open.fee_protocol;
                fee_treasury += result_open.fee_treasury;
                fee_referral += result_open.fee_referral;
            }

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            let skew_1 = pool.skew_abs_snap();

            let activated_requests = account.try_set_keeper_requests_status(request.activate_requests, STATUS_ACTIVE);
            let cancelled_requests = account.try_set_keeper_requests_status(request.cancel_requests, STATUS_CANCELLED);

            self._assert_account_integrity(config, pool, account, oracle);
            self._assert_pool_integrity(config, pool, skew_1 - skew_0);

            Runtime::emit_event(EventMarginOrder {
                account: account.address(),
                pair_id: pair_id.clone(),
                price_limit,
                amount_close,
                amount_open,
                activated_requests,
                cancelled_requests,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
                fee_keeper: dec!(0), // TODO
                price,
            });
        }

        fn _swap_debt(
            &self, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            resource: &ResourceAddress, 
            mut payment_token: Bucket, 
        ) -> (Bucket, Bucket) {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT, Runtime::bech32_encode_address(payment_token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );            
            assert!(
                account.virtual_balance() < dec!(0),
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_SWAP_NO_DEBT, account.virtual_balance(), dec!(0)
            );

            let value = payment_token.amount().min(-account.virtual_balance());
            let price_resource = oracle.price_resource(*resource);
            let amount = value / price_resource;

            let available = account.collateral_amount(resource);
            let (amount, value) = if amount > available {
                let value = available * price_resource;
                (available, value)
            } else {
                (amount, value)
            };
            
            pool.deposit(payment_token.take_advanced(value, TO_INFINITY));
            self._settle_account(pool, account, value);
            let token = account.withdraw_collateral_batch(vec![(*resource, amount)], TO_ZERO).pop().unwrap();

            Runtime::emit_event(EventSwapDebt {
                account: account.address(),
                resource: *resource,
                amount,
                price: price_resource,
            });

            (token, payment_token)
        }

        fn _liquidate(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            mut payment_token: Bucket,
        ) -> ResultLiquidate {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT, Runtime::bech32_encode_address(payment_token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );

            let result_positions = self._liquidate_positions(config, pool, account, oracle); //(pnl, margin_positions, fee_protocol, fee_treasury, fee_referral)
            let result_collateral = self._liquidate_collateral(config, account, oracle); // (collateral_value, margin_collateral, mut collateral_tokens)
            
            let account_value = result_positions.pnl + result_collateral.collateral_value + account.virtual_balance();
            let margin = result_positions.margin_positions + result_collateral.margin_collateral;

            assert!(
                account_value < margin,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_LIQUIDATION_SUFFICIENT_MARGIN, account_value, margin
            );
            
            let mut tokens = account.withdraw_collateral_batch(result_collateral.collateral_amounts.clone(), TO_ZERO);
            let deposit_token = payment_token.take_advanced(result_collateral.collateral_value, TO_INFINITY);
            tokens.push(payment_token);
            let value = deposit_token.amount();
            pool.deposit(deposit_token);

            let settlement = (result_positions.pnl + value).min(account.virtual_balance());
            self._settle_account(pool, account, settlement);
            self._settle_fee_distributor(pool, account, result_positions.fee_protocol, result_positions.fee_treasury, result_positions.fee_referral);

            account.update_valid_requests_start();

            ResultLiquidate {
                tokens,
                position_amounts: result_positions.position_amounts,
                collateral_amounts: result_collateral.collateral_amounts,
                collateral_value: result_collateral.collateral_value,
                margin,
                fee_pool: result_positions.fee_pool,
                fee_protocol: result_positions.fee_protocol,
                fee_treasury: result_positions.fee_treasury,
                fee_referral: result_positions.fee_referral,
                position_prices: result_positions.position_prices,
                collateral_prices: result_collateral.collateral_prices,
            }
        }

        fn _auto_deleverage(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
        ) -> ResultAutoDeleverage {
            let exchange_config = config.exchange_config();

            self._update_pair(config, pool, oracle, pair_id);
            self._settle_funding(pool, account, pair_id);

            let skew_ratio_0 = self._skew_ratio(pool);
            assert!(
                skew_ratio_0 > exchange_config.skew_ratio_cap,
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_ADL_SKEW_TOO_LOW, skew_ratio_0, exchange_config.skew_ratio_cap
            );
                
            let position = account.position(pair_id);
            let price = oracle.price(pair_id);
            let amount = position.amount;

            assert!(
                amount != dec!(0),
                "{}, VALUE:{}, REQUIRED:{}, OP:!= |", ERROR_ADL_NO_POSITION, amount, dec!(0)
            );

            let value = position.amount * price;
            let cost = position.cost;

            let pnl_percent = (value - cost) / cost;

            let u = skew_ratio_0 / exchange_config.adl_a - exchange_config.adl_offset / exchange_config.adl_a;
            let threshold = -(u * u * u) - exchange_config.adl_b * u;
            assert!(
                pnl_percent > threshold,
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_ADL_PNL_BELOW_THRESHOLD, pnl_percent, threshold
            );
            
            let amount_close = -amount;
            let result_close = self._close_position(config, pool, account, oracle, pair_id, amount_close);

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            self._assert_account_integrity(config, pool, account, oracle);

            let skew_ratio_1 = self._skew_ratio(pool);
            assert!(
                skew_ratio_1 < skew_ratio_0,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_ADL_SKEW_NOT_REDUCED, skew_ratio_1, skew_ratio_0
            );

            ResultAutoDeleverage {
                amount: amount_close,
                pnl_percent,
                threshold,
                fee_pool: result_close.fee_pool,
                fee_protocol: result_close.fee_protocol,
                fee_treasury: result_close.fee_treasury,
                fee_referral: result_close.fee_referral,
                price,
            }
        }

        fn _open_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
            amount: Decimal, 
        ) -> ResultOpenPosition {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);

            assert!(
                !pair_config.disabled, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_PAIR_DISABLED, pair_config.disabled, true
            );

            let price = oracle.price(pair_id);

            let value = amount * price;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);
            
            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short + amount) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_delta = skew_abs_1 - skew_abs_0;
            let fee = value_abs * (pair_config.fee_0 + skew_abs_delta / pool_value.max(dec!(1)) * pair_config.fee_1).clamp(dec!(0), exchange_config.fee_max);
            let fee_protocol = fee * exchange_config.fee_share_protocol;
            let fee_treasury = fee * exchange_config.fee_share_treasury;
            let fee_referral = fee * exchange_config.fee_share_referral;
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

            self._settle_fee_distributor(pool, account, fee_protocol, fee_treasury, fee_referral);

            self._assert_position_limit(config, account);

            ResultOpenPosition {
                fee_pool: fee - fee_protocol - fee_treasury - fee_referral,
                fee_protocol,
                fee_treasury,
                fee_referral,
            }
        }

        fn _close_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
            amount: Decimal, 
        ) -> ResultClosePosition {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);
            let price = oracle.price(pair_id);
            
            let value = -amount * price;
            let value_abs = value.checked_abs().unwrap();
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);

            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_delta = skew_abs_1 - skew_abs_0;
            let fee = value_abs * (pair_config.fee_0 + skew_abs_delta / pool_value.max(dec!(1)) * pair_config.fee_1).clamp(dec!(0), exchange_config.fee_max);
            let fee_protocol = fee * exchange_config.fee_share_protocol;
            let fee_treasury = fee * exchange_config.fee_share_treasury;
            let fee_referral = fee * exchange_config.fee_share_referral;
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

            self._settle_account(pool, account, pnl);
            self._settle_fee_distributor(pool, account, fee_protocol, fee_treasury, fee_referral);

            ResultClosePosition {
                fee_pool: fee - fee_protocol - fee_treasury - fee_referral,
                fee_protocol,
                fee_treasury,
                fee_referral,
            }
        }

        fn _update_pair(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: &PairId,
        ) -> bool {
            let pair_config = config.pair_config(pair_id);
            let price = oracle.price(pair_id);

            let mut pool_position = pool.position(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price;
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
                let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
                pool_position.funding_2_rate += funding_2_rate_delta;

                if !oi_long.is_zero() && !oi_short.is_zero() {
                    let funding_1_rate = skew * pair_config.funding_1;
                    let funding_2_rate = pool_position.funding_2_rate * pair_config.funding_2;
                    let funding_rate = funding_1_rate + funding_2_rate;

                    let (funding_long_index, funding_short_index, funding_share) = if funding_rate.is_positive() {
                        let funding_long = funding_rate * period;
                        let funding_long_index = funding_long / oi_long;
        
                        let funding_share = funding_long * pair_config.funding_share;
                        let funding_short_index = -(funding_long - funding_share) / oi_short;
        
                        (funding_long_index, funding_short_index, funding_share)
                    } else {
                        let funding_short = -funding_rate * period * price;
                        let funding_short_index = funding_short / oi_short;
        
                        let funding_share = funding_short * pair_config.funding_share;
                        let funding_long_index = -(funding_short - funding_share) / oi_long;
        
                        (funding_long_index, funding_short_index, funding_share)
                    };

                    let funding_pool_0_rate = (oi_long + oi_short) * price * pair_config.funding_pool_0;
                    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
                    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;

                    let funding_pool = funding_pool_rate * period;
                    let funding_pool_index = funding_pool / (oi_long + oi_short);
                    pool.update_unrealized_pool_funding(pool.unrealized_pool_funding() + funding_pool + funding_share);

                    pool_position.funding_long_index += funding_long_index + funding_pool_index;
                    pool_position.funding_short_index += funding_short_index + funding_pool_index;
                }
            }

            let price_delta_ratio = (price - pool_position.last_price).checked_abs().expect(ERROR_ARITHMETIC) / pool_position.last_price;
            pool_position.last_price = price;
            pool_position.last_update = current_time;

            pool.update_position(pair_id, pool_position);

            if period_seconds >= pair_config.update_period_seconds || price_delta_ratio >= pair_config.update_price_delta_ratio {
                true
            } else {
                false
            }
        }

        fn _update_pair_snaps(
            &self, 
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: &PairId,
        ) {
            let price = oracle.price(pair_id);

            let mut pool_position = pool.position(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price;
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
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (pair_id, position) in account.positions().iter() {
                let pair_config = config.pair_config(pair_id);
                let price = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position(pair_id);

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_delta = skew_abs_1 - skew_abs_0;
                let fee = value_abs * (pair_config.fee_0 + skew_abs_delta / pool_value.max(dec!(1)) * pair_config.fee_1).clamp(dec!(0), exchange_config.fee_max);
                let cost = position.cost;
                let funding = if position.amount.is_positive() {
                    position.amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    position.amount * (pool_position.funding_short_index - position.funding_index)            
                };

                let pnl = value - cost - fee - funding;
                let margin = value_abs * pair_config.margin_initial;
                total_pnl += pnl;
                total_margin += margin;
            }

            (total_pnl, total_margin)
        }

        fn _liquidate_positions(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> ResultLiquidatePositions {
            let exchange_config = config.exchange_config();
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            let mut total_fee_pool = dec!(0);
            let mut total_fee_protocol = dec!(0);
            let mut total_fee_treasury = dec!(0);
            let mut total_fee_referral = dec!(0);
            let mut position_amounts = vec![];
            let mut prices = vec![];
            for (pair_id, position) in account.positions().clone().iter() {
                let pair_config = config.pair_config(pair_id);
                let price = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let mut pool_position = pool.position(pair_id);

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_delta = skew_abs_1 - skew_abs_0;
                let fee = value_abs * (pair_config.fee_0 + skew_abs_delta / pool_value.max(dec!(1)) * pair_config.fee_1).clamp(dec!(0), exchange_config.fee_max);
                let cost = position.cost;
                let funding = if position.amount.is_positive() {
                    pool_position.oi_long -= amount;
                    position.amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    pool_position.oi_short += amount;
                    position.amount * (pool_position.funding_short_index - position.funding_index)            
                };
                pool_position.cost -= cost;

                let pnl = value - cost - fee - funding;
                let margin = value_abs * pair_config.margin_maintenance;
                let fee_protocol = fee * exchange_config.fee_share_protocol;
                let fee_treasury = fee * exchange_config.fee_share_treasury;
                let fee_referral = fee * exchange_config.fee_share_referral;
                
                total_pnl += pnl;
                total_margin += margin;
                total_fee_pool += fee - fee_protocol - fee_treasury - fee_referral;
                total_fee_protocol += fee_protocol;
                total_fee_treasury += fee_treasury;
                total_fee_referral += fee_referral;
                position_amounts.push((pair_id.clone(), position.amount));
                prices.push((pair_id.clone(), price));

                pool.update_position(pair_id, pool_position);
                account.remove_position(pair_id);
            }

            ResultLiquidatePositions {
                pnl: total_pnl,
                margin_positions: total_margin,
                fee_pool: total_fee_pool,
                fee_protocol: total_fee_protocol,
                fee_treasury: total_fee_treasury,
                fee_referral: total_fee_referral,
                position_amounts,
                position_prices: prices,
            }
        }

        fn _value_collateral(
            &self, 
            config: &VirtualConfig,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Decimal) {
            let mut total_value_discounted = dec!(0);
            let mut total_margin = dec!(0);
            for (resource, collateral_config) in config.collateral_configs().iter() {
                let price_resource = oracle.price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource;
                let value_discounted = value * collateral_config.discount;
                let margin = value * collateral_config.margin;
                total_value_discounted += value_discounted;
                total_margin += margin;
            }
            (total_value_discounted, total_margin)
        }

        fn _liquidate_collateral(
            &self, 
            config: &VirtualConfig,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> ResultLiquidateCollateral {            
            let mut total_value_discounted = dec!(0);
            let mut total_margin = dec!(0);
            let mut collateral_amounts = vec![];
            let mut prices = vec![];
            for (resource, collateral_config) in config.collateral_configs().iter() {
                let price_resource = oracle.price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource;
                let value_discounted = value * collateral_config.discount;
                let margin = value * collateral_config.margin;

                total_value_discounted += value_discounted;
                total_margin += margin;
                collateral_amounts.push((*resource, amount));
                prices.push((*resource, price_resource));
            }

            ResultLiquidateCollateral {
                collateral_value: total_value_discounted,
                margin_collateral: total_margin,
                collateral_amounts,
                collateral_prices: prices,
            }
        }

        fn _settle_account(
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
            pair_id: &PairId,
        ) {
            let pool_position = pool.position(pair_id);

            let funding = if let Some(position) = account.positions().get(pair_id) {
                if position.amount.is_positive() {
                    position.amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    position.amount * (pool_position.funding_short_index - position.funding_index)            
                }
            } else {
                dec!(0)
            };

            pool.update_unrealized_pool_funding(pool.unrealized_pool_funding() - funding);
            self._settle_account(pool, account, -funding);
        }

        fn _settle_fee_distributor(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            protocol_amount: Decimal,
            treasury_amount: Decimal,
            referral_amount: Decimal,
        ) {
            pool.update_virtual_balance(pool.virtual_balance() - protocol_amount + treasury_amount + referral_amount);
            self.fee_distributor.reward(protocol_amount, treasury_amount, referral_amount, account.address());
        }
        
        fn _save_funding_index(
            &self,
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: &PairId,
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
