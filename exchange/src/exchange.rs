mod errors;
mod events;
mod requests;
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
)]
mod exchange {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;
    const KEEPER_REWARD_RESOURCE: ResourceAddress = _KEEPER_REWARD_RESOURCE;

    extern_blueprint! {
        "package_tdx_2_1pk475g9y8gn7l4qd4rp93ywrmzyu9rwtfrd4jmd4ud4qc28708vrtr",
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
        "package_tdx_2_1phdymw29xcg3wf9sws29w5875sr0jhfen66vr4g8qvadvcy0jr53xr",
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

            // Authority protected methods
            fn update(&self, update: MarginAccountUpdates);
            fn deposit_collateral_batch(&self, tokens: Vec<Bucket>);
            fn withdraw_collateral_batch(&self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>;
        }
    }
    extern_blueprint! {
        "package_tdx_2_1pha2490wxujetdpy0929d3689hgwv469s6936mjml9ypu6an44m4pv",
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
        "package_tdx_2_1p4auuyyu903k5rk5p875s3vk2gh2fln4vtjzlaw9wgkemvfahpr3y7",
        Oracle {
            // Getter methods
            fn prices(&self, max_age: Instant) -> HashMap<PairId, Decimal>;
        }
    }
    extern_blueprint! {
        "package_tdx_2_1phj5qjfpc5vw08rrxg3s4ke8lkewy9nq9n7luddy94cshxyr5d9p3l",
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
        "package_tdx_2_1pkksw5k00mslulh5epu6etlqtxkmhtr4r7cad507p0hm48w005mgm8",
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
            update_max_lock => restrict_to: [OWNER];
            update_is_contingent => restrict_to: [OWNER];
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
                "{}", ERROR_INVALID_AUTHORITY
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

        pub fn update_max_lock(
            &self, 
            max_lock: Decimal,
        ) {
            authorize!(self, {
                self.fee_delegator.update_max_lock(max_lock);
            })
        }

        pub fn update_is_contingent(
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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
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
                let account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
                account.verify_level_2_auth();

                let current_referrer = self.fee_distributor.get_referrer(account.address());
                assert!(
                    current_referrer.is_none(),
                    "{}", ERROR_REFERRAL_ALREADY_SET
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
                let account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
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
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
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
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let payment = self._remove_liquidity(&config, &mut pool, lp_token);
                pool.realize();
                payment
            })
        }

        pub fn add_collateral(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            tokens: Vec<Bucket>,
        ) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_collateral_configs();
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };
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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let config = VirtualConfig::new(self.config);
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };

                account.verify_level_2_auth();

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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };

                account.verify_level_3_auth();

                let request = Request::MarginOrder(RequestMarginOrder {
                    pair_id,
                    amount,
                    price_limit,
                    activate_requests,
                    cancel_requests,
                });
                account.push_request(request, expiry_seconds, status);

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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };

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
                let mut account = if let Some(fee_oath) = fee_oath {
                    let mut config = VirtualConfig::new(self.config);
                    config.load_collateral_configs();
                    let mut account = VirtualMarginAccount::new(account, config.collaterals());
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), Instant::new(0));
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                    account
                } else {
                    VirtualMarginAccount::new(account, vec![])
                };

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
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
                config.load_collateral_configs();

                let mut account = VirtualMarginAccount::new(account, config.collaterals());
                let (request, submission) = account.process_request(index);
                
                let mut pair_ids = account.position_ids();
                if let Request::MarginOrder(request) = &request {
                    pair_ids.insert(request.pair_id);
                }
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids);

                let max_age = self._max_age(&config);
                let max_age = if max_age.compare(submission, TimeComparisonOperator::Gt) {
                    max_age
                } else {
                    submission
                };
                
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), max_age);

                match request {
                    Request::RemoveCollateral(request) => {
                        self._remove_collateral(&config, &mut pool, &mut account, &oracle, request);
                    },
                    Request::MarginOrder(request) => {
                        self._margin_order(&config, &mut pool, &mut account, &oracle, request);
                    },
                }

                account.realize();
                pool.realize();

                ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(100) // TODO: Set reward amount
            })
        }

        pub fn swap_debt(
            &self, 
            account: ComponentAddress, 
            resource: ResourceAddress, 
            payment: Bucket, 
        ) -> (Bucket, Bucket) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
                config.load_collateral_configs();

                self._assert_valid_collateral(&config, resource);
                let mut account = VirtualMarginAccount::new(account, vec![resource]);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), max_age);

                let (token, remainder) = self._swap_debt(&mut pool, &mut account, &oracle, &resource, payment);
    
                account.realize();
                pool.realize();
    
                (token, remainder)
            })
        }

        pub fn liquidate(
            &self,
            account: ComponentAddress,
            payment_tokens: Bucket,
        ) -> (Vec<Bucket>, Bucket) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
                config.load_collateral_configs();

                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let pair_ids = account.position_ids();
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids);

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), max_age);

                let tokens = self._liquidate(&config, &mut pool, &mut account, &oracle, payment_tokens);

                account.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(1000); // TODO: Set reward amount

                (tokens, reward)
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: PairId, 
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();
                config.load_collateral_configs();

                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let mut pair_ids = account.position_ids();
                pair_ids.insert(pair_id);
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids);

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), max_age);

                self._auto_deleverage(&config, &mut pool, &mut account, &oracle, pair_id);

                account.realize();
                pool.realize();

                ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(200) // TODO: Set reward amount
            })
        }

        pub fn update_pair(
            &self, 
            pair_id: PairId,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                config.load_exchange_config();

                let pair_ids = HashSet::from([pair_id]);
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids);

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), max_age);

                let rewarded = self._update_pair(&config, &mut pool, &oracle, pair_id);

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
                    "{}", ERROR_INVALID_PAYMENT
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
            assert!(
                self._skew_ratio(pool) < config.exchange_config().skew_ratio_cap || skew_delta > dec!(0),
                "{}", ERROR_SKEW_TOO_HIGH
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
                "{}", ERROR_INSUFFICIENT_MARGIN
            );
        }

        fn _assert_valid_collateral(
            &self, 
            config: &VirtualConfig,
            resource: ResourceAddress,
        ) {
            assert!(
                config.collateral_configs().contains_key(&resource),
                "{}", ERROR_COLLATERAL_INVALID
            );
        }

        fn _assert_position_limit(
            &self, 
            config: &VirtualConfig,
            account: &VirtualMarginAccount,
        ) {
            assert!(
                account.positions().len() <= config.exchange_config().positions_max as usize,
                "{}", ERROR_POSITIONS_TOO_MANY
            );
        }

        fn _settle_fee_oath(
            &self,
            config: &VirtualConfig,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            fee_oath: Bucket,
        ) {
            assert!(
                fee_oath.resource_address() == self.fee_oath_resource,
                "{}", ERROR_INVALID_FEE_OATH
            );

            let (collateral_value_discounted, margin_collateral) = self._value_collateral(config, account, oracle);
            let collateral_value_approx = collateral_value_discounted + account.virtual_balance();
            let fee_value = fee_oath.amount();

            assert!(
                collateral_value_approx - fee_value > margin_collateral,
                "{}", ERROR_INSUFFICIENT_MARGIN
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

            let pool_value = self._pool_value(pool);
            let value = payment.amount();
            let fee = value * config.exchange_config().fee_liquidity;
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();
            
            let lp_amount = if lp_supply.is_zero() {
                value
            } else {
                (value - fee) / pool_value.max(dec!(1)) * lp_supply
            };

            pool.deposit(payment);
            let lp_token = pool.mint_lp(lp_amount);

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
                "{}", ERROR_INVALID_LP_TOKEN
            );

            let pool_value = self._pool_value(pool);
            let lp_supply = pool.lp_token_manager().total_supply().unwrap();
            let lp_amount = lp_token.amount();

            let value = lp_amount / lp_supply * pool_value.max(dec!(0));
            let fee = value * config.exchange_config().fee_liquidity;

            pool.burn_lp(lp_token);
            let payment = pool.withdraw(value - fee, TO_ZERO);
            
            self._assert_pool_integrity(config, pool, dec!(0));

            payment
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
            for token in tokens.iter() {
                self._assert_valid_collateral(config, token.resource_address());
            }

            account.deposit_collateral_batch(tokens);
        }
        
        fn _remove_collateral(
            &self, 
            config: &VirtualConfig,
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
                    let value = base_token.amount();
                    self._settle_account(pool, account, -value);
                    tokens.push(base_token);
                    false
                } else {
                    true
                }
            });
            tokens.append(&mut account.withdraw_collateral_batch(claims, TO_ZERO));
            
            let mut target_account = Global::<Account>::try_from(target_account).expect(ERROR_INVALID_ACCOUNT);
            target_account.try_deposit_batch_or_abort(tokens, Some(ResourceOrNonFungible::Resource(AUTHORITY_RESOURCE)));
            
            self._assert_account_integrity(config, pool, account, oracle);
        }

        fn _margin_order(
            &self, 
            config: &VirtualConfig,
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

            self._update_pair(config, pool, oracle, pair_id);
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

            let skew_0 = pool.skew_abs_snap();

            if !amount_close.is_zero() {
                self._close_position(config, pool, account, oracle, pair_id, amount_close);
            }
            if !amount_open.is_zero() {
                self._open_position(config, pool, account, oracle, pair_id, amount_open);
            }

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            let skew_1 = pool.skew_abs_snap();

            let status_updates = request.activate_requests.into_iter().map(|index| (index, STATUS_ACTIVE))
                .chain(request.cancel_requests.into_iter().map(|index| (index, STATUS_CANCELLED)))
                .collect();
            account.try_set_keeper_request_statuses(status_updates);

            self._assert_account_integrity(config, pool, account, oracle);
            self._assert_pool_integrity(config, pool, skew_1 - skew_0);
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

            let available = account.collateral_amount(resource);
            let (amount, value) = if amount > available {
                let value = available * price_resource;
                (available, value)
            } else {
                (amount, value)
            };
            
            pool.deposit(payment_token.take_advanced(value, TO_INFINITY));
            self._settle_account(pool, account, value);
            let collateral = account.withdraw_collateral_batch(vec![(*resource, amount)], TO_ZERO).pop().unwrap();

            (collateral, payment_token)
        }

        fn _liquidate(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            mut payment_token: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}", ERROR_INVALID_PAYMENT
            );

            let (pnl, margin_positions, fee_protocol, fee_treasury, fee_referral) = self._liquidate_positions(config, pool, account, oracle);
            let (collateral_value, margin_collateral, mut collateral_tokens) = self._liquidate_collateral(config, account, oracle);
            let account_value = pnl + collateral_value + account.virtual_balance();
            let margin = margin_positions + margin_collateral;

            assert!(
                account_value < margin,
                "{}", ERROR_LIQUIDATION_SUFFICIENT_MARGIN
            );
            
            let deposit_token = payment_token.take_advanced(collateral_value, TO_INFINITY);
            let value = deposit_token.amount();

            pool.deposit(deposit_token);
            let settlement = (pnl + value).min(account.virtual_balance());
            self._settle_account(pool, account, settlement);
            self._settle_fee_distributor(pool, account, fee_protocol, fee_treasury, fee_referral);

            account.update_valid_requests_start();

            collateral_tokens.push(payment_token);
            collateral_tokens
        }

        fn _auto_deleverage(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: PairId, 
        ) {
            let exchange_config = config.exchange_config();

            self._update_pair(config, pool, oracle, pair_id);
            self._settle_funding(pool, account, pair_id);

            let skew_ratio_0 = self._skew_ratio(pool);
            assert!(
                skew_ratio_0 > exchange_config.skew_ratio_cap,
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

            let u = skew_ratio_0 / exchange_config.adl_a - exchange_config.adl_offset / exchange_config.adl_a;
            let threshold = -(u * u * u) - exchange_config.adl_b * u;
            assert!(
                pnl_percent > threshold,
                "{}", ERROR_ADL_PNL_BELOW_THRESHOLD
            );
            
            let amount_close = -amount;
            self._close_position(config, pool, account, oracle, pair_id, amount_close);

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            self._assert_account_integrity(config, pool, account, oracle);

            let skew_ratio_1 = self._skew_ratio(pool);
            assert!(
                skew_ratio_1 < skew_ratio_0,
                "{}", ERROR_ADL_SKEW_NOT_REDUCED
            );
        }

        fn _open_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: PairId, 
            amount: Decimal, 
        ) {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);

            assert!(
                !pair_config.disabled, 
                "{}", ERROR_PAIR_DISABLED
            );

            let price_token = oracle.price(pair_id);

            let value = amount * price_token;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);
            
            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short + amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
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
        }

        fn _close_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: PairId, 
            amount: Decimal, 
        ) {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);
            let price_token = oracle.price(pair_id);
            
            let value = amount * price_token;
            let value_abs = value.checked_abs().unwrap();
            let pool_value = self._pool_value(pool);

            let mut pool_position = pool.position(pair_id);
            let mut position = account.position(pair_id);

            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
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
        }

        fn _update_pair(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            oracle: &VirtualOracle,
            pair_id: PairId,
        ) -> bool {
            let pair_config = config.pair_config(pair_id);
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
                        let funding_short = -funding_rate * period * price_token;
                        let funding_short_index = funding_short / oi_short;
        
                        let funding_share = funding_short * pair_config.funding_share;
                        let funding_long_index = -(funding_short - funding_share) / oi_long;
        
                        (funding_long_index, funding_short_index, funding_share)
                    };

                    let funding_pool_0_rate = (oi_long + oi_short) * price_token * pair_config.funding_pool_0;
                    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
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
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> (Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (&pair_id, position) in account.positions().iter() {
                let pair_config = config.pair_config(pair_id);
                let price_token = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position(pair_id);

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
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
        ) -> (Decimal, Decimal, Decimal, Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            let mut total_fee_protocol = dec!(0);
            let mut total_fee_treasury = dec!(0);
            let mut total_fee_referral = dec!(0);
            for (&pair_id, position) in account.positions().clone().iter() {
                let pair_config = config.pair_config(pair_id);
                let price_token = oracle.price(pair_id);
                let amount = position.amount;
                let value = position.amount * price_token;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let mut pool_position = pool.position(pair_id);

                let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
                let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short - amount) * price_token).checked_abs().expect(ERROR_ARITHMETIC);
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
                total_pnl += pnl;
                total_margin += margin;
                total_fee_protocol += fee * exchange_config.fee_share_protocol;
                total_fee_treasury += fee * exchange_config.fee_share_treasury;
                total_fee_referral += fee * exchange_config.fee_share_referral;

                pool.update_position(pair_id, pool_position);
                account.remove_position(pair_id);
            }

            (total_pnl, total_margin, total_fee_protocol, total_fee_treasury, total_fee_referral)
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
        ) -> (Decimal, Decimal, Vec<Bucket>) {            
            let mut total_value_discounted = dec!(0);
            let mut total_margin = dec!(0);
            let mut withdraw_collateral = vec![];
            for (resource, collateral_config) in config.collateral_configs().iter() {
                let price_resource = oracle.price_resource(*resource);
                let amount = account.collateral_amount(resource);
                let value = amount * price_resource;
                let value_discounted = value * collateral_config.discount;
                let margin = value * collateral_config.margin;
                total_value_discounted += value_discounted;
                total_margin += margin;
                withdraw_collateral.push((*resource, amount));
            }
            let collateral_tokens = account.withdraw_collateral_batch(withdraw_collateral, TO_ZERO);

            (total_value_discounted, total_margin, collateral_tokens)
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
            pair_id: PairId,
        ) {
            let pool_position = pool.position(pair_id);

            let funding = if let Some(position) = account.positions().get(&pair_id) {
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
