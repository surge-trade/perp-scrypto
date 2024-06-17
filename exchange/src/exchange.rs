mod errors;
mod events;
mod requests;
mod structs;
mod virtual_config;
mod virtual_margin_pool;
mod virtual_margin_account;
mod virtual_oracle;

use scrypto::prelude::*;
use common::{PairId, ListIndex, ReferralData, _AUTHORITY_RESOURCE, _BASE_RESOURCE, _LP_RESOURCE, _PROTOCOL_RESOURCE, _KEEPER_REWARD_RESOURCE, _REFERRAL_RESOURCE, TO_ZERO, TO_INFINITY};
use account::*;
use config::*;
use permission_registry::*;
use pool::*;
use referral_generator::*;
use self::errors::*;
use self::events::*;
use self::requests::*;
use self::structs::*;
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
    EventLiquidityChange,
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
    const LP_RESOURCE: ResourceAddress = _LP_RESOURCE;
    const PROTOCOL_RESOURCE: ResourceAddress = _PROTOCOL_RESOURCE;
    const KEEPER_REWARD_RESOURCE: ResourceAddress = _KEEPER_REWARD_RESOURCE;
    const REFERRAL_RESOURCE: ResourceAddress = _REFERRAL_RESOURCE;

    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Config {
            // Constructor
            fn new(initial_rule: AccessRule) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self) -> ConfigInfoCompressed;
            fn get_pair_configs(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<PairConfigCompressed>;
            fn get_pair_configs_by_ids(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PairConfigCompressed>>;
            fn get_pair_configs_len(&self) -> ListIndex;

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
            fn new(level_1: AccessRule, level_2: AccessRule, level_3: AccessRule, referral_id: Option<NonFungibleLocalId>, reservation: Option<GlobalAddressReservation>) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self, collateral_resources: Vec<ResourceAddress>) -> MarginAccountInfo;
            fn get_request(&self, index: ListIndex) -> Option<KeeperRequest>;
            fn get_requests(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)>;
            fn get_requests_tail(&self, n: ListIndex, end: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)>;
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
            fn get_position(&self, pair_id: PairId) -> PoolPosition;
            fn get_positions(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, PoolPosition>;     

            // Authority protected methods
            fn update(&self, update: MarginPoolUpdates);
            fn deposit(&self, token: Bucket);
            fn withdraw(&self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;            
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        ReferralGenerator {
            // Getter methods
            fn get_referral_code(&self, hash: Hash) -> Option<ReferralCode>;

            // Authority protected methods
            fn create_referral_codes(&self, tokens: Vec<Bucket>, referral_id: NonFungibleLocalId, referrals: Vec<(Hash, Vec<(ResourceAddress, Decimal)>, u64)>);
            fn claim_referral_code(&self, hash: Hash) -> (NonFungibleLocalId, Vec<Bucket>);
        }
    }
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        PermissionRegistry {
            // Getter methods
            fn get_permissions(&self, access_rule: AccessRule) -> Permissions;
        
            // Authority protected methods
            fn set_permissions(&self, access_rule: AccessRule, permissions: Permissions);
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
            fn get_protocol_virtual_balance(&self) -> Decimal;
            fn get_treasury_virtual_balance(&self) -> Decimal;

            // Authority protected methods
            fn update_protocol_virtual_balance(&self, protocol_virtual_balance: Decimal);
            fn update_treasury_virtual_balance(&self, treasury_virtual_balance: Decimal);
            fn distribute(&self, amount_protocol: Decimal, amount_treasury: Decimal);
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
            collect_treasury => restrict_to: [OWNER, admin];
            mint_referral => restrict_to: [OWNER, admin];
            update_referral => restrict_to: [OWNER, admin];

            // Get methods
            get_pairs => PUBLIC;
            get_permissions => PUBLIC;
            get_account_details => PUBLIC;
            get_pool_details => PUBLIC;
            get_pair_details => PUBLIC;
            get_exchange_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_pair_configs_by_ids => PUBLIC;
            get_pair_configs_len => PUBLIC;
            get_collateral_config => PUBLIC;
            get_collateral_configs => PUBLIC;
            get_collaterals => PUBLIC;
            get_pool_value => PUBLIC;
            get_skew_ratio => PUBLIC;
            get_protocol_balance => PUBLIC;

            // User methods
            create_referrals => restrict_to: [user];
            create_account => restrict_to: [user];
            set_level_1_auth => restrict_to: [user];
            set_level_2_auth => restrict_to: [user];
            set_level_3_auth => restrict_to: [user];
            collect_referral_rewards => restrict_to: [user];
            add_liquidity => restrict_to: [user];
            remove_liquidity => restrict_to: [user];
            add_collateral => restrict_to: [user];
            remove_collateral_request => restrict_to: [user];
            margin_order_request => restrict_to: [user];
            cancel_request => restrict_to: [user];
            cancel_requests => restrict_to: [user];
            cancel_all_requests => restrict_to: [user];

            // Keeper methods
            process_request => restrict_to: [keeper];
            swap_debt => restrict_to: [keeper];
            liquidate => restrict_to: [keeper];
            auto_deleverage => restrict_to: [keeper];
            update_pairs => restrict_to: [keeper];
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
        referral_generator: Global<ReferralGenerator>,
        permission_registry: Global<PermissionRegistry>,
        oracle: Global<Oracle>,
        fee_distributor: Global<FeeDistributor>,
        fee_oath_resource: ResourceAddress,
    }
    
    impl Exchange {
        pub fn new(
            owner_role: OwnerRole,
            authority_token: Bucket,
            config: ComponentAddress,
            pool: ComponentAddress,
            referral_generator: ComponentAddress,
            permission_registry: ComponentAddress,
            oracle: ComponentAddress,
            fee_distributor: ComponentAddress,
            fee_oath_resource: ResourceAddress,
            reservation: Option<GlobalAddressReservation>,
        ) -> Global<Exchange> {
            let component_reservation = match reservation {
                Some(reservation) => reservation,
                None => Runtime::allocate_component_address(Exchange::blueprint_id()).0
            };

            assert!(
                authority_token.resource_address() == AUTHORITY_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_AUTHORITY, Runtime::bech32_encode_address(authority_token.resource_address()), Runtime::bech32_encode_address(AUTHORITY_RESOURCE)
            );

            let config: Global<Config> = config.into();
            let pool: Global<MarginPool> = pool.into();
            let referral_generator: Global<ReferralGenerator> = referral_generator.into();
            let permission_registry: Global<PermissionRegistry> = permission_registry.into();
            let oracle: Global<Oracle> = oracle.into();
            let fee_distributor: Global<FeeDistributor> = fee_distributor.into();

            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                config,
                pool,
                referral_generator,
                permission_registry,
                oracle,
                fee_distributor,
                fee_oath_resource
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
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

        pub fn collect_treasury(
            &self, 
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = self.fee_distributor.get_treasury_virtual_balance();

                self.fee_distributor.update_treasury_virtual_balance(dec!(0));
                pool.add_virtual_balance(amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn mint_referral(
            &self,
            fee_referral: Decimal,
            fee_rebate: Decimal,
            max_referrals: u64,
        ) -> Bucket {
            let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
            assert!(
                fee_referral >= dec!(0) && fee_referral <= dec!(0.1) &&
                fee_rebate >= dec!(0) && fee_rebate <= dec!(0.1),
                "{}", ERROR_INVALID_REFERRAL_DATA
            );

            let referral_data = ReferralData {
                fee_referral,
                fee_rebate,
                referrals: 0,
                max_referrals,
                balance: dec!(0),
                total_rewarded: dec!(0),
            };
            let referral = referral_manager.mint_ruid_non_fungible(referral_data);

            referral
        }

        pub fn update_referral(
            &self,
            referral_id: NonFungibleLocalId,
            fee_referral: Option<Decimal>,
            fee_rebate: Option<Decimal>,
            max_referrals: Option<u64>,
        ) {
            let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);

            if let Some(fee_referral) = fee_referral {
                assert!(
                    fee_referral >= dec!(0) && fee_referral <= dec!(0.1),
                    "{}", ERROR_INVALID_REFERRAL_DATA
                );
                referral_manager.update_non_fungible_data(&referral_id, "fee_referral", fee_referral);
            }
            if let Some(fee_rebate) = fee_rebate {
                assert!(
                    fee_rebate >= dec!(0) && fee_rebate <= dec!(0.1),
                    "{}", ERROR_INVALID_REFERRAL_DATA
                );
                referral_manager.update_non_fungible_data(&referral_id, "fee_rebate", fee_rebate);
            }
            if let Some(max_referrals) = max_referrals {
                referral_manager.update_non_fungible_data(&referral_id, "max_referrals", max_referrals);
            }
        }

        // --- GET METHODS ---

        pub fn get_pairs(
            &self,
            n: ListIndex,
            start: Option<ListIndex>,
        ) -> Vec<PairId> {
            self.config.get_pair_configs(n, start).into_iter().map(|v| v.pair_id).collect()
        }

        pub fn get_permissions(
            &self, 
            access_rule: AccessRule,
        ) -> Permissions {
            self.permission_registry.get_permissions(access_rule)
        }

        pub fn get_account_details(
            &self, 
            account: ComponentAddress,
            history_len: ListIndex,
            history_start: Option<ListIndex>,
        ) -> AccountDetails {
            let mut config = VirtualConfig::new(self.config);
            let account = VirtualMarginAccount::new(account, config.collaterals());
            let pair_ids = account.position_ids();
            config.load_pair_configs(pair_ids.clone());
            let pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

            self._account_details(&config, &pool, &account, history_len, history_start)
        }

        pub fn get_pool_details(
            &self,
        ) -> PoolDetails {
            let config = VirtualConfig::new(self.config);
            let pool = VirtualLiquidityPool::new(self.pool, HashSet::new());

            self._pool_details(&config, &pool)
        }

        pub fn get_pair_details(
            &self, 
            pair_ids: Vec<PairId>,
        ) -> Vec<PairDetails> {
            let mut config = VirtualConfig::new(self.config);
            let pair_ids_set: HashSet<PairId> = pair_ids.iter().cloned().collect();
            config.load_pair_configs(pair_ids_set.clone());
            let pool = VirtualLiquidityPool::new(self.pool, pair_ids_set);
            pair_ids.into_iter().map(|pair_id| self._pair_details(&config, &pool, &pair_id)).collect()
        }

        pub fn get_exchange_config(
            &self
        ) -> ExchangeConfig {
            self.config.get_info().exchange.decompress()
        }

        pub fn get_pair_configs(
            &self, 
            n: ListIndex, 
            start: Option<ListIndex>,
        ) -> Vec<PairConfig> {
            self.config.get_pair_configs(n, start).into_iter().map(|v| v.decompress()).collect()
        }
        
        pub fn get_pair_configs_by_ids(
            &self, 
            pair_ids: HashSet<PairId>,
        ) -> HashMap<PairId, PairConfig> {
            self.config.get_pair_configs_by_ids(pair_ids).into_iter()
            .map(|(k, v)| (k, v.expect(ERROR_MISSING_PAIR_CONFIG).decompress())).collect()
        }
        
        pub fn get_pair_configs_len(
            &self,
        ) -> ListIndex {
            self.config.get_pair_configs_len()
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

        pub fn get_protocol_balance(
            &self,
        ) -> Decimal {
            self.fee_distributor.get_protocol_virtual_balance()
        }

        // --- USER METHODS ---

        pub fn create_referrals(
            &self, 
            referral_proof: Proof,
            tokens: Vec<Bucket>, 
            referrals: Vec<(Hash, Vec<(ResourceAddress, Decimal)>, u64)>,
        ) {
            authorize!(self, {
                let checked_referral: NonFungible<ReferralData> = referral_proof.check_with_message(REFERRAL_RESOURCE, ERROR_INVALID_REFERRAL).as_non_fungible().non_fungible();
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                let referral_id = checked_referral.local_id();
                let referral_data = checked_referral.data();
                let count: u64 = referrals.iter().map(|(_, _, count)| *count).sum();

                assert!(
                    referral_data.referrals + count <= referral_data.max_referrals,
                    "{}", ERROR_REFERRAL_LIMIT_REACHED
                );

                referral_manager.update_non_fungible_data(referral_id, "referrals", referral_data.referrals + count);
                self.referral_generator.create_referral_codes(tokens, referral_id.clone(), referrals);
            })
        }

        pub fn collect_referral_rewards(
            &self, 
            referral_proof: Proof,
        ) -> Bucket {
            authorize!(self, {
                let checked_referral: NonFungible<ReferralData> = referral_proof.check_with_message(REFERRAL_RESOURCE, ERROR_INVALID_REFERRAL).as_non_fungible().non_fungible();
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                let referral_id = checked_referral.local_id();
                let referral_data = checked_referral.data();

                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = referral_data.balance;

                referral_manager.update_non_fungible_data(referral_id, "balance", dec!(0));
                pool.add_virtual_balance(amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn create_account(
            &self, 
            fee_oath: Option<Bucket>,
            initial_rule: AccessRule,
            mut tokens: Vec<Bucket>,
            referral_code: Option<String>,
            reservation: Option<GlobalAddressReservation>,
        ) -> Global<MarginAccount> {
            authorize!(self, {
                let referral_id = if let Some(referral_code) = referral_code {
                    let referral_code_hash = CryptoUtils::keccak256_hash(referral_code.into_bytes());
                    let (referral_id, referral_tokens) = self.referral_generator.claim_referral_code(referral_code_hash);
                    tokens.extend(referral_tokens);
                    Some(referral_id)
                } else {
                    None
                };

                let account_global = Blueprint::<MarginAccount>::new(
                    initial_rule.clone(),
                    initial_rule.clone(),
                    initial_rule.clone(),
                    referral_id,
                    reservation
                );
                let account_component = account_global.address();

                let mut permissions = self.permission_registry.get_permissions(initial_rule.clone());
                permissions.level_1.insert(account_component);
                permissions.level_2.insert(account_component);
                permissions.level_3.insert(account_component);
                self.permission_registry.set_permissions(initial_rule, permissions);

                let config = VirtualConfig::new(self.config);
                let mut account = VirtualMarginAccount::new(account_component, config.collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());

                self._add_collateral(&config, &mut pool, &mut account, tokens);

                if let Some(fee_oath) = fee_oath {
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), HashSet::new(), Instant::new(0), None);
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                }

                pool.realize();
                account.realize();

                Runtime::emit_event(EventAccountCreation {
                    account: account_component,
                });

                account_global
            })
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

                let old_rule = account.get_level_1_auth();
                let mut permissions = self.permission_registry.get_permissions(old_rule.clone());
                permissions.level_1.shift_remove(&account.address());
                self.permission_registry.set_permissions(old_rule, permissions);

                account.set_level_1_auth(rule.clone());

                let mut permissions = self.permission_registry.get_permissions(rule.clone());
                permissions.level_1.insert(account.address());
                self.permission_registry.set_permissions(rule, permissions);

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

                let old_rule = account.get_level_2_auth();
                let mut permissions = self.permission_registry.get_permissions(old_rule.clone());
                permissions.level_2.shift_remove(&account.address());
                self.permission_registry.set_permissions(old_rule, permissions);

                account.set_level_2_auth(rule.clone());

                let mut permissions = self.permission_registry.get_permissions(rule.clone());
                permissions.level_2.insert(account.address());
                self.permission_registry.set_permissions(rule, permissions);

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

                let old_rule = account.get_level_3_auth();
                let mut permissions = self.permission_registry.get_permissions(old_rule.clone());
                permissions.level_3.shift_remove(&account.address());
                self.permission_registry.set_permissions(old_rule, permissions);

                account.set_level_3_auth(rule.clone());

                let mut permissions = self.permission_registry.get_permissions(rule.clone());
                permissions.level_3.insert(account.address());
                self.permission_registry.set_permissions(rule, permissions);

                account.realize();
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
                let mut account = VirtualMarginAccount::new(account, config.collaterals());
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());

                self._add_collateral(&config, &mut pool, &mut account, tokens);

                if let Some(fee_oath) = fee_oath {
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), HashSet::new(), Instant::new(0), None);
                    self._settle_fee_oath(&config, &mut account, &oracle, fee_oath);
                }

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

                account.push_request(request, 0, expiry_seconds, STATUS_ACTIVE, vec![target_account]);
                self._assert_active_requests_limit(&config, &account);

                account.realize();
            })
        }

        pub fn margin_order_request(
            &self,
            fee_oath: Option<Bucket>,
            delay_seconds: Option<u64>,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: PairId,
            amount: Decimal,
            reduce_only: bool,
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
                    reduce_only,
                    price_limit,
                    activate_requests,
                    cancel_requests,
                });

                account.push_request(request, 0, expiry_seconds, status, vec![]);
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

        pub fn cancel_requests(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            indexes: Vec<ListIndex>,
        ) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                let mut account = self._handle_fee_oath(account, &config, fee_oath);

                account.verify_level_3_auth();
                account.cancel_requests(indexes);
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
            price_updates: Option<(Vec<u8>, Bls12381G2Signature)>,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let mut account = VirtualMarginAccount::new(account, config.collaterals());
                let (request, expired) = account.process_request(index);
                
                if !expired {
                    let mut pair_ids = account.position_ids();
                    if let Request::MarginOrder(request) = &request {
                        pair_ids.insert(request.pair_id.clone());
                    }
                    config.load_pair_configs(pair_ids.clone());
                    let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                    let max_age = self._max_age(&config);
                    let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, price_updates);

                    match request {
                        Request::RemoveCollateral(request) => {
                            self._remove_collateral(&config, &mut pool, &mut account, &oracle, request);
                        },
                        Request::MarginOrder(request) => {
                            self._margin_order(&config, &mut pool, &mut account, &oracle, request);
                        },
                    };

                    pool.realize();
                }

                account.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(config.exchange_config().reward_keeper);
                reward
            })
        }

        pub fn swap_debt(
            &self, 
            account: ComponentAddress, 
            resource: ResourceAddress, // TODO: make list of resources
            payment: Bucket, 
            price_updates: Option<(Vec<u8>, Bls12381G2Signature)>,
        ) -> (Bucket, Bucket) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);
                self._assert_valid_collateral(&config, resource);
                let collaterals = vec![resource];
                let mut account = VirtualMarginAccount::new(account, collaterals);
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                
                let max_age = self._max_age(&config);
                let collateral_feeds = HashMap::from([(resource, config.collateral_feeds().get(&resource).unwrap().clone())]);
                let oracle = VirtualOracle::new(self.oracle, collateral_feeds, HashSet::new(), max_age, price_updates);

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
            price_updates: Option<(Vec<u8>, Bls12381G2Signature)>,
        ) -> (Vec<Bucket>, Bucket) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let pair_ids = account.position_ids();
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, price_updates);

                let tokens = self._liquidate(&config, &mut pool, &mut account, &oracle, payment);

                account.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(config.exchange_config().reward_keeper);
                (tokens, reward)
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: PairId, 
            price_updates: Option<(Vec<u8>, Bls12381G2Signature)>,
        ) -> Bucket {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                let mut account = VirtualMarginAccount::new(account, config.collaterals());

                let mut pair_ids = account.position_ids();
                pair_ids.insert(pair_id.clone());
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids, max_age, price_updates);

                self._auto_deleverage(&config, &mut pool, &mut account, &oracle, &pair_id);

                account.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(config.exchange_config().reward_keeper);
                reward
            })
        }

        pub fn update_pairs(
            &self, 
            pair_ids: Vec<PairId>,
            price_updates: Option<(Vec<u8>, Bls12381G2Signature)>,
        ) -> (Bucket, Vec<bool>) {
            authorize!(self, {
                let mut config = VirtualConfig::new(self.config);
                
                let pair_ids: HashSet<PairId> = pair_ids.into_iter().collect();
                config.load_pair_configs(pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(self.pool, pair_ids.clone());

                let max_age = self._max_age(&config);
                let oracle = VirtualOracle::new(self.oracle, config.collateral_feeds(), pair_ids.clone(), max_age, price_updates);

                let rewarded: Vec<bool> = pair_ids.iter().map(|pair_id| {
                    self._update_pair(&config, &mut pool, &oracle, pair_id)
                }).collect();

                pool.realize();

                let rewards_count = rewarded.iter().filter(|r| **r).count();
                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(rewards_count);
                
                (reward, rewarded)
            })
        }

        pub fn swap_protocol_fee(&self, mut payment: Bucket) -> (Bucket, Bucket) {
            authorize!(self, {
                let config = VirtualConfig::new(self.config);

                assert!(
                    payment.resource_address() == PROTOCOL_RESOURCE,
                    "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT, Runtime::bech32_encode_address(payment.resource_address()), Runtime::bech32_encode_address(PROTOCOL_RESOURCE)
                );

                let burn_amount = config.exchange_config().protocol_burn_amount;
                payment.take_advanced(burn_amount, TO_INFINITY).burn();
                
                let mut pool = VirtualLiquidityPool::new(self.pool, HashSet::new());
                let amount = self.fee_distributor.get_protocol_virtual_balance();

                self.fee_distributor.update_protocol_virtual_balance(dec!(0));
                pool.add_virtual_balance(amount);
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
            let result_value_positions = self._value_positions(config, pool, account, oracle);
            let result_value_collateral = self._value_collateral(config, account, oracle);
            let account_value = result_value_positions.pnl + result_value_collateral.collateral_value_discounted + account.virtual_balance();
            let margin = result_value_positions.margin_positions + result_value_collateral.margin_collateral;

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

        fn _calculate_fee_rate(
            &self,
            pair_config: &PairConfig,
            pool_position: &PoolPosition,
            pool_value: Decimal,
            price: Decimal,
            amount: Decimal, 
        ) -> Decimal {
            let skew_abs_0 = ((pool_position.oi_long - pool_position.oi_short) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_1 = ((pool_position.oi_long - pool_position.oi_short + amount) * price).checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_delta = skew_abs_1 - skew_abs_0;
            let fee_rate_0 = pair_config.fee_0;
            let fee_rate_1 = skew_abs_delta / pool_value.max(dec!(1)) * pair_config.fee_1;
            fee_rate_0 + fee_rate_1
        }

        fn _account_details(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount,
            history_len: ListIndex,
            history_start: Option<ListIndex>,
        ) -> AccountDetails {
            let position_details: Vec<PositionDetails> = account.positions().iter().map(|(pair_id, position)| {
                let pair_config = config.pair_config(pair_id);
                let pool_position = pool.position(pair_id);

                let funding = if position.amount.is_positive() {
                    position.amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    position.amount * (pool_position.funding_short_index - position.funding_index)            
                };
                let margin_initial = position.amount.checked_abs().expect(ERROR_ARITHMETIC) * pair_config.margin_initial;
                let margin_maintenance = position.amount.checked_abs().expect(ERROR_ARITHMETIC) * pair_config.margin_maintenance;

                PositionDetails {
                    pair_id: pair_id.clone(),
                    amount: position.amount,
                    cost: position.cost,
                    funding,
                    margin_initial,
                    margin_maintenance,
                }
            }).collect();

            let collateral_details: Vec<CollateralDetails> = config.collateral_configs().iter().map(|(&resource, collateral_config)| {
                let pair_id = collateral_config.pair_id.clone();
                let amount = account.collateral_amount(&resource);
                let discount = collateral_config.discount;
                let margin = amount * collateral_config.margin;

                CollateralDetails {
                    pair_id,
                    resource,
                    amount,
                    discount,
                    margin,
                }
            }).collect();

            let active_requests = account.active_requests().into_iter()
                .map(|(index, keeper_request)| {
                    RequestDetails {
                        index,
                        request: Request::decode(&keeper_request.request),
                        submission: keeper_request.submission,
                        expiry: keeper_request.expiry,
                        status: keeper_request.status,
                    }
                })
                .collect();

            let requests_history = account.requests_tail(history_len, history_start).into_iter()
                .map(|(index, keeper_request)| {
                    RequestDetails {
                        index,
                        request: Request::decode(&keeper_request.request),
                        submission: keeper_request.submission,
                        expiry: keeper_request.expiry,
                        status: keeper_request.status,
                    }
                })
                .collect();

            AccountDetails {
                virtual_balance: account.virtual_balance(),
                positions: position_details,
                collaterals: collateral_details,
                valid_requests_start: account.valid_requests_start(),
                active_requests,
                requests_history,
                referral: account.referral(),
            }
        }

        fn _pool_details(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
        ) -> PoolDetails {
            let lp_token_manager = ResourceManager::from_address(LP_RESOURCE);
            let lp_supply = lp_token_manager.total_supply().unwrap();
            let pool_value = self._pool_value(pool).max(dec!(1));
            let lp_price = if lp_supply.is_zero() {
                dec!(1)
            } else {
                pool_value / lp_supply
            };

            PoolDetails {
                base_tokens_amount: pool.base_tokens_amount(),
                virtual_balance: pool.virtual_balance(),
                unrealized_pool_funding: pool.unrealized_pool_funding(),
                pnl_snap: pool.pnl_snap(),
                skew_ratio: self._skew_ratio(pool),
                skew_ratio_cap: config.exchange_config().skew_ratio_cap,
                lp_supply,
                lp_price,
            }
        }

        fn _pair_details(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            pair_id: &PairId,
        ) -> PairDetails {
            let pair_config = config.pair_config(pair_id);
            let pool_position = pool.position(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;
            let funding_2_rate = pool_position.funding_2_rate * pair_config.funding_2;

            PairDetails {
                pair_id: pair_id.clone(),
                oi_long,
                oi_short,
                funding_2: funding_2_rate,
                pair_config: pair_config.clone(),
            }
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
            let result_value_collateral = self._value_collateral(config, account, oracle);
            let collateral_value_approx = result_value_collateral.collateral_value_discounted + account.virtual_balance() - fee_value;

            assert!(
                collateral_value_approx > result_value_collateral.margin_collateral,
                "{}, VALUE:{}, REQUIRED:{}, OP:> |", ERROR_INSUFFICIENT_MARGIN, collateral_value_approx, result_value_collateral.margin_collateral
            );

            account.add_virtual_balance(-fee_value);
            fee_oath.burn();
        }

        fn _add_liquidity(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            payment: Bucket,
        ) -> Bucket {
            let lp_token_manager = ResourceManager::from_address(LP_RESOURCE);
            assert!(
                payment.resource_address() == BASE_RESOURCE,
                "{}", ERROR_INVALID_PAYMENT
            );

            let value = payment.amount();
            let fee = value * config.exchange_config().fee_liquidity;
            let pool_value = self._pool_value(pool).max(dec!(1));
            let lp_supply = lp_token_manager.total_supply().unwrap();

            let (lp_amount, lp_price) = if lp_supply.is_zero() {
                (value, dec!(1))
            } else {
                let lp_price = pool_value / lp_supply;
                let lp_amount = (value - fee) / lp_price;
                (lp_amount, lp_price)
            };

            pool.deposit(payment);
            let lp_token = lp_token_manager.mint(lp_amount);
            let (fee_pool, fee_protocol, fee_treasury) = self._settle_fees_basic(config, pool, fee);

            Runtime::emit_event(EventLiquidityChange {
                lp_price,
                lp_amount,
                amount: value,
                fee_pool,
                fee_protocol,
                fee_treasury,
            });

            lp_token
        }

        fn _remove_liquidity(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            lp_token: Bucket,
        ) -> Bucket {
            let lp_token_manager = ResourceManager::from_address(LP_RESOURCE);
            assert!(
                lp_token.resource_address() == LP_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_LP_TOKEN, Runtime::bech32_encode_address(lp_token.resource_address()), Runtime::bech32_encode_address(LP_RESOURCE)
            );

            let lp_amount = lp_token.amount();
            let pool_value = self._pool_value(pool).max(dec!(0));
            let lp_supply = lp_token_manager.total_supply().unwrap();

            let lp_price = pool_value / lp_supply;
            let value = lp_amount * lp_price;
            let fee = value * config.exchange_config().fee_liquidity;
            
            lp_token.burn();
            let token = pool.withdraw(value - fee, TO_ZERO);
            let (fee_pool, fee_protocol, fee_treasury) = self._settle_fees_basic(config, pool, fee);
            
            self._assert_pool_integrity(config, pool, dec!(0));

            Runtime::emit_event(EventLiquidityChange {
                lp_price,
                lp_amount: -lp_amount,
                amount: -token.amount(),
                fee_pool,
                fee_protocol,
                fee_treasury,
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
            let amounts = tokens.iter().map(|token| (token.resource_address(), token.amount())).collect();
            if let Some(index) = tokens.iter().position(|token| token.resource_address() == BASE_RESOURCE) {
                let base_token = tokens.remove(index);
                let value = base_token.amount();
                pool.deposit(base_token);
                self._settle_account(pool, account, value);
            }
            tokens.iter().for_each(|token| self._assert_valid_collateral(config, token.resource_address()));

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
            let amounts = tokens.iter().map(|token| (token.resource_address(), token.amount())).collect();
            
            let mut target_account = Global::<Account>::try_from(target_account_component).expect(ERROR_INVALID_ACCOUNT);
            target_account.try_deposit_batch_or_abort(tokens, Some(ResourceOrNonFungible::Resource(AUTHORITY_RESOURCE)));
            
            self._assert_account_integrity(config, pool, account, oracle);

            Runtime::emit_event(EventRemoveCollateral {
                account: account.address(),
                target_account: target_account_component,
                amounts,
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
            let reduce_only = request.reduce_only;
            let price_limit = request.price_limit;

            let price = oracle.price(pair_id);
            assert!(
                price_limit.compare(price),
                "{}, VALUE:{}, REQUIRED:{}, OP:{} |", ERROR_MARGIN_ORDER_PRICE_LIMIT, price, price_limit.price(), price_limit.op()
            );

            self._update_pair(config, pool, oracle, pair_id);
            let funding = self._settle_funding(pool, account, pair_id);
                
            let (amount_close, amount_open) = {
                let position_amount = account.positions().get(pair_id).map_or(dec!(0), |p| p.amount);

                let amount_close = if position_amount.is_positive() && amount.is_negative() {
                    amount.max(-position_amount)
                } else if position_amount.is_negative() && amount.is_positive() {
                    amount.min(-position_amount)
                } else {
                    dec!(0)
                };

                let amount_open = if !reduce_only {
                    amount - amount_close
                } else {
                    dec!(0)
                };

                (amount_close, amount_open)
            };

            let skew_0 = pool.skew_abs_snap();

            let mut pnl = dec!(0);
            let mut fee_paid = dec!(0);
            if !amount_close.is_zero() {
                let (pnl_close, fee_close) = self._close_position(config, pool, account, oracle, pair_id, amount_close);
                pnl += pnl_close;
                fee_paid += fee_close;
            }
            if !amount_open.is_zero() {
                let fee_open = self._open_position(config, pool, account, oracle, pair_id, amount_open);
                fee_paid += fee_open;
            }
            let (fee_pool, fee_protocol, fee_treasury, fee_referral) = self._settle_fees_referral(config, pool, account, fee_paid);

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            let skew_1 = pool.skew_abs_snap();

            let activated_requests = account.try_set_keeper_requests_status(request.activate_requests, STATUS_ACTIVE);
            let cancelled_requests = account.try_set_keeper_requests_status(request.cancel_requests, STATUS_CANCELLED);

            self._assert_pool_integrity(config, pool, skew_1 - skew_0);

            Runtime::emit_event(EventMarginOrder {
                account: account.address(),
                pair_id: pair_id.clone(),
                price,
                price_limit,
                amount_close,
                amount_open,
                activated_requests,
                cancelled_requests,
                pnl: pnl + funding,
                funding,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
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
        ) -> Vec<Bucket> {
            assert!(
                payment_token.resource_address() == BASE_RESOURCE, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT, Runtime::bech32_encode_address(payment_token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );

            let result_positions = self._liquidate_positions(config, pool, account, oracle); 
            let result_collateral = self._liquidate_collateral(config, account, oracle); 
            
            let virtual_balance = account.virtual_balance();
            let account_value = result_positions.pnl + result_collateral.collateral_value_discounted + virtual_balance;
            let margin = result_positions.margin_positions + result_collateral.margin_collateral;

            assert!(
                account_value < margin,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_LIQUIDATION_SUFFICIENT_MARGIN, account_value, margin
            );

            let value = ResourceManager::from(BASE_RESOURCE).amount_for_withdrawal(result_collateral.collateral_value_discounted, TO_INFINITY);
            assert!(
                payment_token.amount() >= value,
                "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_LIQUIDATION_INSUFFICIENT_PAYMENT, payment_token.amount(), value
            );
            
            let mut tokens = account.withdraw_collateral_batch(result_collateral.collateral_amounts.clone(), TO_ZERO);
            pool.deposit(payment_token.take(value));
            tokens.push(payment_token);

            let settlement = result_positions.pnl + value;
            self._settle_account(pool, account, settlement);
            let pool_loss = if account.virtual_balance().is_negative() {
                let pool_loss = account.virtual_balance();
                self._settle_account(pool, account, -pool_loss);
                pool_loss
            } else {
                dec!(0)
            };
            let (fee_pool, fee_protocol, fee_treasury, fee_referral) = self._settle_fees_referral(config, pool, account, result_positions.fee_paid);

            account.update_valid_requests_start();

            Runtime::emit_event(EventLiquidate {
                account: account.address(),
                position_prices: result_positions.position_prices,
                collateral_prices: result_collateral.collateral_prices,
                account_value,
                margin,
                virtual_balance,
                position_amounts: result_positions.position_amounts,
                positions_pnl: result_positions.pnl,
                collateral_amounts: result_collateral.collateral_amounts,
                collateral_value: result_collateral.collateral_value,
                collateral_value_discounted: result_collateral.collateral_value_discounted,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
                pool_loss,
            });

            tokens
        }

        fn _auto_deleverage(
            &self, 
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
        ) {
            let exchange_config = config.exchange_config();

            self._update_pair(config, pool, oracle, pair_id);
            let funding = self._settle_funding(pool, account, pair_id);

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
            let (pnl, fee_paid) = self._close_position(config, pool, account, oracle, pair_id, amount_close);
            let (fee_pool, fee_protocol, fee_treasury, fee_referral) = self._settle_fees_referral(config, pool, account, fee_paid);

            self._save_funding_index(pool, account, pair_id);
            self._update_pair_snaps(pool, oracle, pair_id);

            let skew_ratio_1 = self._skew_ratio(pool);
            assert!(
                skew_ratio_1 < skew_ratio_0,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_ADL_SKEW_NOT_REDUCED, skew_ratio_1, skew_ratio_0
            );

            Runtime::emit_event(EventAutoDeleverage {
                account: account.address(),
                pair_id: pair_id.clone(),
                price,
                amount_close,
                pnl: pnl + funding,
                funding,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
                pnl_percent,
                threshold,
            });
        }

        fn _open_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
            amount: Decimal, 
        ) -> Decimal {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);
            let fee_rebate = account.fee_rebate();

            assert!(
                !pair_config.disabled, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_PAIR_DISABLED, pair_config.disabled, true
            );

            let price = oracle.price(pair_id);
            let value = amount * price;
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let pool_value = self._pool_value(pool);

            let pool_position = pool.position_mut(pair_id);
            let position = account.position_mut(pair_id);
            
            let fee_rate = self._calculate_fee_rate(pair_config, pool_position, pool_value, price, amount);
            let fee = value_abs * (fee_rate * fee_rebate).clamp(dec!(0), exchange_config.fee_max);
            let cost = value + fee;

            if amount.is_positive() {
                pool_position.oi_long += amount;
            } else {
                pool_position.oi_short -= amount;
            }
            pool_position.cost += cost;
            
            position.amount += amount;
            position.cost += cost;

            self._assert_position_limit(config, account);
            self._assert_account_integrity(config, pool, account, oracle);

            fee
        }

        fn _close_position(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            pair_id: &PairId, 
            amount: Decimal, 
        ) -> (Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let pair_config = config.pair_config(pair_id);
            let fee_rebate = account.fee_rebate();

            let price = oracle.price(pair_id);
            let value = amount * price;
            let value_abs = value.checked_abs().unwrap();
            let pool_value = self._pool_value(pool);

            let pool_position = pool.position_mut(pair_id);
            let position = account.position_mut(pair_id);

            let fee_rate = self._calculate_fee_rate(pair_config, pool_position, pool_value, price, amount);
            let fee = value_abs * (fee_rate * fee_rebate).clamp(dec!(0), exchange_config.fee_max);
            let cost = -amount / position.amount * position.cost;
            let pnl = -value - cost - fee;
        
            if amount.is_negative() {
                pool_position.oi_long += amount;
            } else {
                pool_position.oi_short -= amount;
            }
            pool_position.cost -= cost;

            position.amount += amount;
            position.cost -= cost;

            self._settle_account(pool, account, pnl);

            (pnl, fee)
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

            let pool_position = pool.position_mut(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - pool_position.skew_abs_snap;
            pool_position.skew_abs_snap = skew_abs;

            let pnl = skew - pool_position.cost;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;
            
            let current_time = Clock::current_time_rounded_to_seconds();
            let period_seconds = current_time.seconds_since_unix_epoch - pool_position.last_update.seconds_since_unix_epoch;
            let period = Decimal::from(period_seconds);
            
            let price_delta_ratio = (price - pool_position.last_price).checked_abs().expect(ERROR_ARITHMETIC) / pool_position.last_price;
            pool_position.last_price = price;
            pool_position.last_update = current_time;
            
            let funding_pool_delta = if !period.is_zero() {
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
                        let funding_short = -funding_rate * period;
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

                    pool_position.funding_long_index += funding_long_index + funding_pool_index;
                    pool_position.funding_short_index += funding_short_index + funding_pool_index;

                    funding_pool + funding_share
                } else {
                    dec!(0)
                }
            } else {
                dec!(0)
            };

            pool.add_skew_abs_snap(skew_abs_snap_delta);
            pool.add_pnl_snap(pnl_snap_delta);
            pool.add_unrealized_pool_funding(funding_pool_delta);

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

            let pool_position = pool.position_mut(pair_id);
            let oi_long = pool_position.oi_long;
            let oi_short = pool_position.oi_short;

            let skew = (oi_long - oi_short) * price;
            let skew_abs = skew.checked_abs().expect(ERROR_ARITHMETIC);
            let skew_abs_snap_delta = skew_abs - pool_position.skew_abs_snap;
            pool_position.skew_abs_snap = skew_abs;
            
            let pnl = skew - pool_position.cost;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;

            pool.add_skew_abs_snap(skew_abs_snap_delta);
            pool.add_pnl_snap(pnl_snap_delta);
        }

        fn _value_positions(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> ResultValuePositions {
            let exchange_config = config.exchange_config();
            let pool_value = self._pool_value(pool);
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (pair_id, position) in account.positions().iter() {
                let pair_config = config.pair_config(pair_id);
                let price = oracle.price(pair_id);
                let amount = position.amount;
                let value = amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position(pair_id);

                let fee_rate = self._calculate_fee_rate(pair_config, pool_position, pool_value, price, amount);
                let fee = value_abs * (fee_rate * account.fee_rebate()).clamp(dec!(0), exchange_config.fee_max);
                let cost = position.cost;
                let funding = if amount.is_positive() {
                    amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    amount * (pool_position.funding_short_index - position.funding_index)            
                };

                let pnl = value - cost - fee - funding;
                let margin = value_abs * pair_config.margin_initial;
                total_pnl += pnl;
                total_margin += margin;
            }

            ResultValuePositions {
                pnl: total_pnl,
                margin_positions: total_margin,
            }        
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
            let fee_rebate = account.fee_rebate();
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            let mut total_funding = dec!(0);
            let mut total_fee_paid = dec!(0);
            let mut position_amounts = vec![];
            let mut prices = vec![];
            for (pair_id, position) in account.positions_mut() {
                let pair_config = config.pair_config(pair_id);
                let price = oracle.price(pair_id);
                let amount = position.amount;
                let value = amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position_mut(pair_id);

                let fee_rate = self._calculate_fee_rate(pair_config, pool_position, pool_value, price, amount);
                let fee = value_abs * (fee_rate * fee_rebate).clamp(dec!(0), exchange_config.fee_max);
                let cost = position.cost;
                let funding = if amount.is_positive() {
                    pool_position.oi_long -= amount;
                    amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    pool_position.oi_short += amount;
                    amount * (pool_position.funding_short_index - position.funding_index)            
                };
                pool_position.cost -= cost;

                let pnl = value - cost - fee - funding;
                let margin = value_abs * pair_config.margin_maintenance;
                total_funding += funding;
                
                total_pnl += pnl;
                total_margin += margin;
                total_fee_paid += fee;
                position_amounts.push((pair_id.clone(), amount));
                prices.push((pair_id.clone(), price));

                position.remove();
            }
            pool.add_unrealized_pool_funding(-total_funding);

            ResultLiquidatePositions {
                pnl: total_pnl,
                margin_positions: total_margin,
                fee_paid: total_fee_paid,
                position_amounts,
                position_prices: prices,
            }
        }

        fn _value_collateral(
            &self, 
            config: &VirtualConfig,
            account: &VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> ResultValueCollateral {
            let mut total_value_discounted = dec!(0);
            let mut total_margin = dec!(0);
            for (&resource, &amount) in account.collateral_amounts().iter() {
                let collateral_config = config.collateral_configs().get(&resource).unwrap();
                let price_resource = oracle.price_resource(resource);
                let value = amount * price_resource;
                let value_discounted = value * collateral_config.discount;
                let margin = value * collateral_config.margin;

                total_value_discounted += value_discounted;
                total_margin += margin;
            }

            ResultValueCollateral {
                collateral_value_discounted: total_value_discounted,
                margin_collateral: total_margin,
            }
        }

        fn _liquidate_collateral(
            &self, 
            config: &VirtualConfig,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
        ) -> ResultLiquidateCollateral {           
            let mut total_value = dec!(0); 
            let mut total_value_discounted = dec!(0);
            let mut total_margin = dec!(0);
            let mut collateral_amounts = vec![];
            let mut prices = vec![];
            for (&resource, &amount) in account.collateral_amounts().iter() {
                let collateral_config = config.collateral_configs().get(&resource).unwrap();
                let price_resource = oracle.price_resource(resource);
                let value = amount * price_resource;
                let value_discounted = value * collateral_config.discount;
                let margin = value * collateral_config.margin;

                total_value += value;
                total_value_discounted += value_discounted;
                total_margin += margin;
                collateral_amounts.push((resource, amount));
                prices.push((resource, price_resource));
            }

            ResultLiquidateCollateral {
                collateral_value: total_value,
                collateral_value_discounted: total_value_discounted,
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
            pool.add_virtual_balance(-amount);
            account.add_virtual_balance(amount);
        }

        fn _settle_funding(
            &self,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: &PairId,
        ) -> Decimal {
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

            pool.add_unrealized_pool_funding(-funding);
            self._settle_account(pool, account, -funding);

            -funding
        }

        fn _settle_fees_basic(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            fee_paid: Decimal,
        ) -> (Decimal, Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let fee_protocol = fee_paid * exchange_config.fee_share_protocol;
            let fee_treasury = fee_paid * exchange_config.fee_share_treasury;
            let fee_pool = fee_paid - fee_protocol - fee_treasury;

            pool.add_virtual_balance(-fee_protocol -fee_treasury);
            self.fee_distributor.distribute(fee_protocol, fee_treasury);

            (fee_pool, fee_protocol, fee_treasury)
        }

        fn _settle_fees_referral(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            fee_paid: Decimal,
        ) -> (Decimal, Decimal, Decimal, Decimal) {
            let exchange_config = config.exchange_config();
            let fee_protocol = fee_paid * exchange_config.fee_share_protocol;
            let fee_treasury = fee_paid * exchange_config.fee_share_treasury;
            let fee_referral = fee_paid * exchange_config.fee_share_referral * account.fee_share_referral();
            let fee_pool = fee_paid - fee_protocol - fee_treasury - fee_referral;

            pool.add_virtual_balance(-fee_protocol -fee_treasury -fee_referral);
            self.fee_distributor.distribute(fee_protocol, fee_treasury);
            account.reward_referral(fee_referral);

            (fee_pool, fee_protocol, fee_treasury, fee_referral)
        }
        
        fn _save_funding_index(
            &self,
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            pair_id: &PairId,
        ) {
            let pool_position = pool.position(pair_id);
            let position: &mut AccountPosition = account.position_mut(pair_id);

            let funding_index = if position.amount.is_positive() {
                pool_position.funding_long_index
            } else {
                pool_position.funding_short_index
            };
            position.funding_index = funding_index;
        }
    }
}
