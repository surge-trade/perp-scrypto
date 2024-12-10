pub mod errors;
pub mod events;
pub mod requests;
pub mod structs;
mod virtual_config;
mod virtual_margin_pool;
mod virtual_margin_account;
mod virtual_oracle;

use scrypto::prelude::*;
pub use ::common::*;
pub use ::oracle::*;
pub use ::config::*;
pub use ::account::*;
pub use ::pool::*;
pub use ::referral_generator::*;
pub use ::permission_registry::*;
pub use self::errors::*;
pub use self::events::*;
pub use self::requests::*;
pub use self::structs::*;
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
    const RECOVERY_KEY_RESOURCE: ResourceAddress = _RECOVERY_KEY_RESOURCE;
    const FEE_OATH_RESOURCE: ResourceAddress = _FEE_OATH_RESOURCE;

    const ORACLE_COMPONENT: ComponentAddress = _ORACLE_COMPONENT;
    const CONFIG_COMPONENT: ComponentAddress = _CONFIG_COMPONENT;
    const POOL_COMPONENT: ComponentAddress = _POOL_COMPONENT;
    const REFERRAL_GENERATOR_COMPONENT: ComponentAddress = _REFERRAL_GENERATOR_COMPONENT;
    const FEE_DELEGATOR_COMPONENT: ComponentAddress = _FEE_DELEGATOR_COMPONENT;
    const FEE_DISTRIBUTOR_COMPONENT: ComponentAddress = _FEE_DISTRIBUTOR_COMPONENT;
    const PERMISSION_REGISTRY_COMPONENT: ComponentAddress = _PERMISSION_REGISTRY_COMPONENT;

    extern_blueprint! {
        ORACLE_PACKAGE,
        Oracle {
            // Constructor
            // fn new(owner_role: OwnerRole, public_key: Bls12381G1PublicKey) -> Global<Oracle>;

            // Authority protected methods
            fn push_and_get_prices_with_auth(&self, pair_ids: HashSet<PairId>, data: Vec<u8>, signature: Bls12381G2Signature, key_id: ListIndex) -> HashMap<PairId, (Decimal, Instant)>;
            fn get_prices_with_auth(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, (Decimal, Instant)>;

            // User methods
            // fn push_and_get_prices(&self, pair_ids: HashSet<PairId>, data: Vec<u8>, signature: Bls12381G2Signature) -> HashMap<PairId, (Decimal, Instant)>;
            // fn get_prices(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, (Decimal, Instant)>;
        }
    }
    extern_blueprint! {
        CONFIG_PACKAGE,
        Config {
            // Constructor
            // fn new(initial_rule: AccessRule) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self, pair_ids: HashSet<PairId>) -> ConfigInfoCompressed;
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
        MARGIN_ACCOUNT_PACKAGE,
        MarginAccount {
            // Constructor
            fn new(level_1: AccessRule, level_2: AccessRule, level_3: AccessRule, referral_id: Option<NonFungibleLocalId>, dapp_definition: GlobalAddress, reservation: Option<GlobalAddressReservation>) -> Global<MarginAccount>;

            // Getter methods
            fn get_info(&self) -> MarginAccountInfo;
            fn get_request(&self, index: ListIndex) -> Option<KeeperRequest>;
            // fn get_requests(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)>;
            fn get_requests_tail(&self, n: ListIndex, end: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)>;
            fn get_requests_by_indexes(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, Option<KeeperRequest>>;
            // fn get_requests_len(&self) -> ListIndex;
            fn get_active_requests(&self) -> HashMap<ListIndex, KeeperRequest>;

            // Authority protected methods
            fn update(&self, update: MarginAccountUpdates);
            // fn update_referral_id(&self, referral_id: Option<NonFungibleLocalId>);
            fn deposit_collateral_batch(&self, tokens: Vec<Bucket>);
            fn withdraw_collateral_batch(&self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>;
        }
    }
    extern_blueprint! {
        MARGIN_POOL_PACKAGE,
        MarginPool {
            // Constructor
            // fn new(owner_role: OwnerRole) -> Global<MarginPool>;

            // Getter methods
            fn get_info(&self, pair_ids: HashSet<PairId>) -> MarginPoolInfo;
            // fn get_position(&self, pair_id: PairId) -> PoolPosition;
            fn get_positions(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, PoolPosition>;     

            // Authority protected methods
            fn update(&self, update: MarginPoolUpdates);
            fn deposit(&self, token: Bucket);
            fn withdraw(&self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket;            
        }
    }
    extern_blueprint! {
        REFERRAL_GENERATOR_PACKAGE,
        ReferralGenerator {
            // Constructor
            // fn new(owner_role: OwnerRole) -> Global<ReferralGenerator>;

            // Getter methods
            fn get_allocations(&self, referral_id: NonFungibleLocalId) -> Vec<ReferralAllocation>;
            // fn get_referral_code(&self, hash: Hash) -> Option<ReferralCode>;

            // Authority protected methods
            fn add_allocation(&self, tokens: Vec<Bucket>, referral_id: NonFungibleLocalId, claims: Vec<(ResourceAddress, Decimal)>, count: u64) -> (Vec<Bucket>, ListIndex);
            fn create_referral_codes(&self, tokens: Vec<Bucket>, referral_id: NonFungibleLocalId, referral_hashes: HashMap<Hash, (Vec<(ResourceAddress, Decimal)>, u64)>) -> Vec<Bucket>;
            fn create_referral_codes_from_allocation(&self, referral_id: NonFungibleLocalId, allocation_index: ListIndex, referral_hashes: HashSet<Hash>);
            fn claim_referral_code(&self, hash: Hash) -> (NonFungibleLocalId, Vec<Bucket>);
        }
    }
    extern_blueprint! {
        FEE_DISTRIBUTOR_PACKAGE,
        FeeDistributor {
            // Constructor
            // fn new(owner_role: OwnerRole) -> Global<FeeDistributor>;

            // Getter methods
            fn get_protocol_virtual_balance(&self) -> Decimal;
            fn get_treasury_virtual_balance(&self) -> Decimal;

            // Authority protected methods
            fn update_protocol_virtual_balance(&self, protocol_virtual_balance: Decimal);
            fn update_treasury_virtual_balance(&self, treasury_virtual_balance: Decimal);
            fn distribute(&self, amount_protocol: Decimal, amount_treasury: Decimal);
        }
    }
    extern_blueprint! {
        FEE_DELEGATOR_PACKAGE,
        FeeDelegator {
            // Constructor
            // fn new(owner_role: OwnerRole) -> Global<FeeDelegator>;

            // Getter methods
            // fn get_info(&self) -> (Decimal, Decimal, Decimal, Decimal, bool);
            fn get_virtual_balance(&self) -> Decimal;

            // Owner protected methods
            // fn update_max_lock(&self, max_lock: Decimal);
            // fn update_price_multiplier(&self, price_multiplier: Decimal);
            // fn update_is_contingent(&self, is_contingent: bool);

            // Authority protected methods
            fn update_virtual_balance(&self, virtual_balance: Decimal);

            // Depositor methods
            // fn deposit(&self, token: Bucket);
            // fn withdraw(&self, amount: Decimal) -> Bucket;

            // User methods
            // fn lock_fee(&self, amount: Decimal) -> Bucket;
        }
    }
    extern_blueprint! {
        PERMISSION_REGISTRY_PACKAGE,
        PermissionRegistry {
            // Constructor
            // fn new(owner_role: OwnerRole) -> Global<PermissionRegistry>;

            // Getter methods
            fn get_permissions(&self, access_rule: AccessRule) -> Permissions;
        
            // Authority protected methods
            fn set_permissions(&self, access_rule: AccessRule, permissions: Permissions);
        }
    }
    
    enable_method_auth! { 
        roles {
            fee_delegator_admin => updatable_by: [OWNER];
            treasury_admin => updatable_by: [OWNER];
            referral_admin => updatable_by: [OWNER];
            user_admin => updatable_by: [OWNER];

            protocol_swap_user => updatable_by: [OWNER, user_admin];
            liquidity_user => updatable_by: [OWNER, user_admin];
            referral_user => updatable_by: [OWNER, user_admin];
            account_creation_user => updatable_by: [OWNER, user_admin];
            account_management_user => updatable_by: [OWNER, user_admin];
            remove_collateral_request_user => updatable_by: [OWNER, user_admin];
            margin_order_request_user => updatable_by: [OWNER, user_admin];
            cancel_request_user => updatable_by: [OWNER, user_admin];

            keeper_process => updatable_by: [OWNER];
            keeper_swap_debt => updatable_by: [OWNER];
            keeper_liquidate => updatable_by: [OWNER];
            keeper_auto_deleverage => updatable_by: [OWNER];
            keeper_update_pairs => updatable_by: [OWNER];
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

            // Admin methods
            collect_treasury => restrict_to: [treasury_admin];
            collect_fee_delegator =>  restrict_to: [fee_delegator_admin];
            mint_referral => restrict_to: [referral_admin];
            mint_referral_with_allocation => restrict_to: [referral_admin];
            update_referral => restrict_to: [referral_admin];
            add_referral_allocation => restrict_to: [referral_admin];

            // Get methods
            get_pairs => PUBLIC;
            get_permissions => PUBLIC;
            get_account_details => PUBLIC;
            get_pool_details => PUBLIC;
            get_pair_details => PUBLIC;
            get_referral_details => PUBLIC;
            get_exchange_config => PUBLIC;
            get_pair_configs => PUBLIC;
            get_pair_configs_len => PUBLIC;
            get_collateral_configs => PUBLIC;
            get_collaterals => PUBLIC;
            get_protocol_balance => PUBLIC;
            get_treasury_balance => PUBLIC;

            // User methods
            swap_protocol_fee => restrict_to: [protocol_swap_user];
            add_liquidity => restrict_to: [liquidity_user];
            remove_liquidity => restrict_to: [liquidity_user];
            create_referral_codes => restrict_to: [referral_user];
            create_referral_codes_from_allocation => restrict_to: [referral_user];
            collect_referral_rewards => restrict_to: [referral_user];
            create_account => restrict_to: [account_creation_user];
            create_recovery_key => restrict_to: [account_management_user];
            add_auth_rule => restrict_to: [account_management_user];
            set_level_1_auth => restrict_to: [account_management_user];
            set_level_2_auth => restrict_to: [account_management_user];
            set_level_3_auth => restrict_to: [account_management_user];
            add_collateral => restrict_to: [account_management_user];
            remove_collateral_request => restrict_to: [remove_collateral_request_user];
            margin_order_request => restrict_to: [margin_order_request_user];
            margin_order_tp_sl_request => restrict_to: [margin_order_request_user];
            cancel_requests => restrict_to: [cancel_request_user];

            // Keeper methods
            process_request => restrict_to: [keeper_process];
            swap_debt => restrict_to: [keeper_swap_debt];
            liquidate => restrict_to: [keeper_liquidate];
            liquidate_v2 => restrict_to: [keeper_liquidate];
            auto_deleverage => restrict_to: [keeper_auto_deleverage];
            update_pairs => restrict_to: [keeper_update_pairs];
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
    }
    
    impl Exchange {
        pub fn new(
            owner_role: OwnerRole,
            authority_token: Bucket,
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

            Global::<Oracle>::try_from(ORACLE_COMPONENT).expect("Oracle component is not a valid Oracle");
            Global::<Config>::try_from(CONFIG_COMPONENT).expect("Config component is not a valid Config");
            Global::<MarginPool>::try_from(POOL_COMPONENT).expect("Pool component is not a valid MarginPool");
            Global::<ReferralGenerator>::try_from(REFERRAL_GENERATOR_COMPONENT).expect("ReferralGenerator component is not a valid ReferralGenerator");
            Global::<FeeDelegator>::try_from(FEE_DELEGATOR_COMPONENT).expect("FeeDelegator component is not a valid FeeDelegator");
            Global::<FeeDistributor>::try_from(FEE_DISTRIBUTOR_COMPONENT).expect("FeeDistributor component is not a valid FeeDistributor");
            Global::<PermissionRegistry>::try_from(PERMISSION_REGISTRY_COMPONENT).expect("PermissionRegistry component is not a valid PermissionRegistry");

            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                fee_delegator_admin => OWNER;
                treasury_admin => OWNER;
                referral_admin => OWNER;
                user_admin => OWNER;
                
                protocol_swap_user => rule!(allow_all);
                liquidity_user => rule!(allow_all);
                referral_user => rule!(allow_all);
                account_creation_user => rule!(allow_all);
                account_management_user => rule!(allow_all);
                remove_collateral_request_user => rule!(allow_all);
                margin_order_request_user => rule!(allow_all);
                cancel_request_user => rule!(allow_all);

                keeper_process => rule!(allow_all);
                keeper_swap_debt => rule!(allow_all);
                keeper_liquidate => rule!(allow_all);
                keeper_auto_deleverage => rule!(allow_all);
                keeper_update_pairs => rule!(allow_all);
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

        pub fn update_exchange_config(
            &mut self, 
            config: ExchangeConfig,
        ) {
            authorize!(self, {
                Global::<Config>::from(CONFIG_COMPONENT).update_exchange_config(config.clone());
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
                Global::<Config>::from(CONFIG_COMPONENT).update_pair_configs(configs.clone());
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
                Global::<Config>::from(CONFIG_COMPONENT).update_collateral_configs(configs.clone());
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
                Global::<Config>::from(CONFIG_COMPONENT).remove_collateral_config(resource);
            });

            Runtime::emit_event(EventCollateralConfigRemoval {
                resource
            });
        }

        // --- ADMIN METHODS ---

        pub fn collect_treasury(
            &self, 
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let fee_distributor = Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT);
                let amount = fee_distributor.get_treasury_virtual_balance();

                assert!(
                    amount <= pool.base_tokens_amount(),
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, amount, pool.base_tokens_amount()
                );

                fee_distributor.update_treasury_virtual_balance(dec!(0));
                pool.add_virtual_balance(amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn collect_fee_delegator(
            &self,
        ) -> Bucket {
            authorize!(self, {
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let fee_delegator = Global::<FeeDelegator>::from(FEE_DELEGATOR_COMPONENT);
                let amount = fee_delegator.get_virtual_balance();

                assert!(
                    amount <= pool.base_tokens_amount(),
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, amount, pool.base_tokens_amount()
                );

                fee_delegator.update_virtual_balance(dec!(0));
                pool.add_virtual_balance(amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                token
            })
        }

        pub fn mint_referral(
            &self,
            name: String,
            description: String,
            key_image_url: Url,
            fee_referral: Decimal,
            fee_rebate: Decimal,
            max_referrals: u64,
        ) -> Bucket {
            authorize!(self, {
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                assert!(
                    fee_referral >= dec!(0) && fee_referral <= dec!(0.1),
                    "{}, VALUE:{}, REQUIRED:{}, OP:bounds |", ERROR_INVALID_REFERRAL_DATA, fee_referral, "[0, 0.1]"
                );
                assert!(
                    fee_rebate >= dec!(0) && fee_rebate <= dec!(0.1),
                    "{}, VALUE:{}, REQUIRED:{}, OP:bounds |", ERROR_INVALID_REFERRAL_DATA, fee_rebate, "[0, 0.1]"
                );

                let referral_data = ReferralData {
                    name,
                    description,
                    key_image_url,
                    fee_referral,
                    fee_rebate,
                    referrals: 0,
                    max_referrals,
                    balance: dec!(0),
                    total_rewarded: dec!(0),
                };
                let referral = referral_manager.mint_ruid_non_fungible(referral_data);

                referral
            })
        }

        pub fn mint_referral_with_allocation(
            &self,
            name: String,
            description: String,
            key_image_url: Url,
            fee_referral: Decimal,
            fee_rebate: Decimal,
            max_referrals: u64,
            allocation_tokens: Vec<Bucket>,
            allocation_claims: Vec<(ResourceAddress, Decimal)>,
            allocation_count: u64,
        ) -> (Bucket, Vec<Bucket>, ListIndex) {
            let referral = self.mint_referral(name, description, key_image_url, fee_referral, fee_rebate, max_referrals);
            let referral_id = referral.as_non_fungible().non_fungible_local_id();

            let (remainder_tokens, allocation_index) = self.add_referral_allocation(referral_id, allocation_tokens, allocation_claims, allocation_count);

            (referral, remainder_tokens, allocation_index)
        }

        pub fn update_referral(
            &self,
            referral_id: NonFungibleLocalId,
            fee_referral: Option<Decimal>,
            fee_rebate: Option<Decimal>,
            max_referrals: Option<u64>,
        ) {
            authorize!(self, {
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);

                if let Some(fee_referral) = fee_referral {
                    assert!(
                        fee_referral >= dec!(0) && fee_referral <= dec!(0.1),
                        "{}, VALUE:{}, REQUIRED:{}, OP:bounds |", ERROR_INVALID_REFERRAL_DATA, fee_referral, "[0, 0.1]"
                    );
                    referral_manager.update_non_fungible_data(&referral_id, "fee_referral", fee_referral);
                }
                if let Some(fee_rebate) = fee_rebate {
                    assert!(
                        fee_rebate >= dec!(0) && fee_rebate <= dec!(0.1),
                        "{}, VALUE:{}, REQUIRED:{}, OP:bounds |", ERROR_INVALID_REFERRAL_DATA, fee_rebate, "[0, 0.1]"
                    );
                    referral_manager.update_non_fungible_data(&referral_id, "fee_rebate", fee_rebate);
                }
                if let Some(max_referrals) = max_referrals {
                    referral_manager.update_non_fungible_data(&referral_id, "max_referrals", max_referrals);
                }
            })
        }

        pub fn add_referral_allocation(
            &self,
            referral_id: NonFungibleLocalId,
            tokens: Vec<Bucket>,
            claims: Vec<(ResourceAddress, Decimal)>,
            count: u64,
        ) -> (Vec<Bucket>, ListIndex) {
            authorize!(self, {
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                let referral_data: ReferralData = referral_manager.get_non_fungible_data(&referral_id);

                tokens.iter().for_each(|token| {
                    let resource = token.resource_address();
                    if resource != BASE_RESOURCE {
                        self._assert_valid_collateral(&config, token.resource_address());
                    }
                });
                assert!(
                    referral_data.referrals + count <= referral_data.max_referrals,
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_REFERRAL_LIMIT_REACHED, referral_data.referrals + count, referral_data.max_referrals
                );

                referral_manager.update_non_fungible_data(&referral_id, "referrals", referral_data.referrals + count);

                let referral_generator = Global::<ReferralGenerator>::from(REFERRAL_GENERATOR_COMPONENT);
                let (remainder_tokens, allocation_index) = referral_generator.add_allocation(tokens, referral_id.clone(), claims, count);

                (remainder_tokens, allocation_index)
            })
        }

        // --- GET METHODS ---

        pub fn get_pairs(
            &self,
            n: ListIndex,
            start: Option<ListIndex>,
        ) -> Vec<PairId> {
            Global::<Config>::from(CONFIG_COMPONENT).get_pair_configs(n, start).into_iter().map(|v| v.pair_id).collect()
        }

        pub fn get_permissions(
            &self, 
            access_rule: AccessRule,
        ) -> Permissions {
            Global::<PermissionRegistry>::from(PERMISSION_REGISTRY_COMPONENT).get_permissions(access_rule)
        }

        pub fn get_account_details(
            &self, 
            account: ComponentAddress,
            history_n: ListIndex,
            history_start: Option<ListIndex>,
        ) -> AccountDetails {
            let account = VirtualMarginAccount::new(account);
            let pair_ids = account.position_ids();
            let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
            let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());

            self._account_details(&config, &pool, &account, history_n, history_start)
        }

        pub fn get_pool_details(
            &self,
        ) -> PoolDetails {
            let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
            let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());

            self._pool_details(&config, &pool)
        }

        pub fn get_pair_details(
            &self, 
            pair_ids: Vec<PairId>,
        ) -> Vec<PairDetails> {
            let pair_ids_set: HashSet<PairId> = pair_ids.iter().cloned().collect();
            let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids_set.clone());
            let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids_set);
            pair_ids.into_iter().map(|pair_id| self._pair_details(&config, &pool, &pair_id)).collect()
        }

        pub fn get_referral_details(
            &self,
            referral_id: NonFungibleLocalId,
        ) -> ReferralDetails {
            let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
            let referral = referral_manager.get_non_fungible_data(&referral_id);
            let referral_generator = Global::<ReferralGenerator>::from(REFERRAL_GENERATOR_COMPONENT);
            let allocations = referral_generator.get_allocations(referral_id);

            ReferralDetails {
                allocations,
                referral,
            }
        }

        pub fn get_exchange_config(
            &self
        ) -> ExchangeConfig {
            Global::<Config>::from(CONFIG_COMPONENT).get_info(HashSet::new()).exchange.decompress()
        }

        pub fn get_pair_configs(
            &self, 
            n: ListIndex, 
            start: Option<ListIndex>,
        ) -> Vec<PairConfig> {
            Global::<Config>::from(CONFIG_COMPONENT).get_pair_configs(n, start).into_iter().map(|v| v.decompress()).collect()
        }
        
        pub fn get_pair_configs_len(
            &self,
        ) -> ListIndex {
            Global::<Config>::from(CONFIG_COMPONENT).get_pair_configs_len()
        }

        pub fn get_collateral_configs(
            &self, 
        ) -> HashMap<ResourceAddress, CollateralConfig> {
            Global::<Config>::from(CONFIG_COMPONENT).get_info(HashSet::new()).collaterals.into_iter()
                .map(|(k, v)| (k, v.decompress())).collect()
        }

        pub fn get_collaterals(
            &self,
        ) -> Vec<ResourceAddress> {
            Global::<Config>::from(CONFIG_COMPONENT).get_info(HashSet::new()).collaterals.keys().cloned().collect()
        }

        pub fn get_protocol_balance(
            &self,
        ) -> Decimal {
            Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).get_protocol_virtual_balance()
        }

        pub fn get_treasury_balance(
            &self,
        ) -> Decimal {
            Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).get_treasury_virtual_balance()
        }

        // --- USER METHODS ---

        pub fn swap_protocol_fee(&self, mut payment: Bucket) -> (Bucket, Bucket) {
            authorize!(self, {
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());

                assert!(
                    payment.resource_address() == PROTOCOL_RESOURCE,
                    "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PROTOCOL_TOKEN, Runtime::bech32_encode_address(payment.resource_address()), Runtime::bech32_encode_address(PROTOCOL_RESOURCE)
                );

                let burn_amount = config.exchange_config().protocol_burn_amount;
                assert!(
                    payment.amount() >= burn_amount,
                    "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_INSUFFICIENT_PAYMENT, payment.amount(), burn_amount
                );
                payment.take_advanced(burn_amount, TO_INFINITY).burn();
                
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let amount = Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).get_protocol_virtual_balance();

                assert!(
                    amount <= pool.base_tokens_amount(),
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, amount, pool.base_tokens_amount()
                );

                Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).update_protocol_virtual_balance(dec!(0));
                pool.add_virtual_balance(amount);
                let token = pool.withdraw(amount, TO_ZERO);
                
                pool.realize();

                (token, payment)
            })
        }

        pub fn add_liquidity(
            &self,
            payment: Bucket,
        ) -> Bucket {
            authorize!(self, {
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
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
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let token = self._remove_liquidity(&config, &mut pool, lp_token);
                pool.realize();

                token
            })
        }

        pub fn create_referral_codes(
            &self, 
            referral_proof: Proof,
            tokens: Vec<Bucket>, 
            referral_hashes: HashMap<Hash, (Vec<(ResourceAddress, Decimal)>, u64)>, 
        ) -> Vec<Bucket> {
            authorize!(self, {
                let checked_referral: NonFungible<ReferralData> = referral_proof.check_with_message(REFERRAL_RESOURCE, ERROR_INVALID_REFERRAL)
                    .as_non_fungible().non_fungible();
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                let referral_id = checked_referral.local_id();
                let referral_data = checked_referral.data();
                let count: u64 = referral_hashes.iter().map(|(_, (_, count))| *count).sum();

                tokens.iter().for_each(|token| {
                    let resource = token.resource_address();
                    if resource != BASE_RESOURCE {
                        self._assert_valid_collateral(&config, token.resource_address());
                    }
                });
                assert!(
                    referral_data.referrals + count <= referral_data.max_referrals,
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_REFERRAL_LIMIT_REACHED, referral_data.referrals + count, referral_data.max_referrals
                );

                referral_manager.update_non_fungible_data(referral_id, "referrals", referral_data.referrals + count);
                let referral_generator = Global::<ReferralGenerator>::from(REFERRAL_GENERATOR_COMPONENT);
                let remainder_tokens = referral_generator.create_referral_codes(tokens, referral_id.clone(), referral_hashes);

                remainder_tokens
            })
        }

        pub fn create_referral_codes_from_allocation(
            &self, 
            referral_proof: Proof,
            allocation_index: ListIndex,
            referral_hashes: HashSet<Hash>,
        ) {
            authorize!(self, {
                let checked_referral: NonFungible<ReferralData> = referral_proof.check_with_message(REFERRAL_RESOURCE, ERROR_INVALID_REFERRAL)
                    .as_non_fungible().non_fungible();
                let referral_id = checked_referral.local_id();

                let referral_generator = Global::<ReferralGenerator>::from(REFERRAL_GENERATOR_COMPONENT);
                referral_generator.create_referral_codes_from_allocation(referral_id.clone(), allocation_index, referral_hashes);
            })
        }

        pub fn collect_referral_rewards(
            &self, 
            referral_proof: Proof,
        ) -> Bucket {
            authorize!(self, {
                let checked_referral: NonFungible<ReferralData> = referral_proof.check_with_message(REFERRAL_RESOURCE, ERROR_INVALID_REFERRAL)
                    .as_non_fungible().non_fungible();
                let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
                let referral_id = checked_referral.local_id();
                let referral_data = checked_referral.data();

                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let amount = referral_data.balance;

                assert!(
                    amount <= pool.base_tokens_amount(),
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, amount, pool.base_tokens_amount()
                );

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
                    let (referral_id, referral_tokens) = Global::<ReferralGenerator>::from(REFERRAL_GENERATOR_COMPONENT).claim_referral_code(referral_code_hash);
                    tokens.extend(referral_tokens);
                    Some(referral_id)
                } else {
                    None
                };

                let dapp_definition: GlobalAddress = Runtime::global_component().get_metadata("dapp_definition")
                    .expect(ERROR_MISSING_DAPP_DEFINITION)
                    .expect(ERROR_MISSING_DAPP_DEFINITION);
                let account_global = Blueprint::<MarginAccount>::new(
                    initial_rule.clone(),
                    initial_rule.clone(),
                    initial_rule.clone(),
                    referral_id.clone(),
                    dapp_definition,
                    reservation,
                );
                let account_component = account_global.address();

                let permission_registry = Global::<PermissionRegistry>::from(PERMISSION_REGISTRY_COMPONENT);
                let mut permissions = permission_registry.get_permissions(initial_rule.clone());
                permissions.level_1.insert(account_component);
                permissions.level_2.insert(account_component);
                permissions.level_3.insert(account_component);
                permission_registry.set_permissions(initial_rule.clone(), permissions);
                if let AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes)) = initial_rule {
                    for rule_node in rule_nodes {
                        let sub_rule = AccessRule::from(rule_node.clone());
                        let mut permissions = permission_registry.get_permissions(sub_rule.clone());
                        permissions.level_1.insert(account_component);
                        permissions.level_2.insert(account_component);
                        permissions.level_3.insert(account_component);
                        permission_registry.set_permissions(sub_rule, permissions);
                    }
                }

                let mut account = VirtualMarginAccount::new(account_component);
                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._add_collateral(&config, &mut pool, &mut account, tokens);
                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);

                    pool.realize();
                } else {
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                    let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());

                    self._add_collateral(&config, &mut pool, &mut account, tokens);

                    pool.realize();
                }
                account.realize();

                Runtime::emit_event(EventAccountCreation {
                    account: account_component,
                    referral_id,
                });

                account_global
            })
        }

        pub fn create_recovery_key(
            &self,
            fee_oath: Option<Bucket>,
            account: ComponentAddress,
        ) -> Bucket {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_1_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }

                let recovery_key_manager = ResourceManager::from_address(RECOVERY_KEY_RESOURCE);

                let recovery_key_data = RecoveryKeyData {
                    name: "Recovery Key".to_string(),
                    description: "Recovery key for your Surge trading account.".to_string(),
                    key_image_url: Url::of("https://surge.trade/images/recovery_key_1.png"),
                };
                let recovery_key = recovery_key_manager.mint_ruid_non_fungible(recovery_key_data);
                let recovery_key_id = recovery_key.as_non_fungible().non_fungible::<RecoveryKeyData>().global_id().clone();

                let mut rule = account.get_level_1_auth();
                match rule {
                    AccessRule::AllowAll => {
                        rule = AccessRule::from(require(recovery_key_id));
                    }
                    AccessRule::DenyAll => {
                        rule = AccessRule::from(require(recovery_key_id));
                    }
                    AccessRule::Protected(rule_node) => {
                        rule = AccessRule::from(rule_node.or(require(recovery_key_id)));
                    }
                }
                self._set_level_1_auth(&mut account, rule);

                account.realize();

                recovery_key
            })
        }

        pub fn add_auth_rule(
            &self,
            fee_oath: Option<Bucket>,
            account: ComponentAddress,
            level: u8,
            additional_rule: AccessRuleNode,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_1_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }

                assert!(
                    vec![1, 2, 3].contains(&level),
                    "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_INVALID_AUTH_LEVEL, level, vec![1, 2, 3]
                );

                let mut rule = match level {
                    1 => account.get_level_1_auth(),
                    2 => account.get_level_2_auth(),
                    3 => account.get_level_3_auth(),
                    _ => unreachable!(),
                };

                match rule {
                    AccessRule::AllowAll => {
                        rule = AccessRule::from(additional_rule);
                    }
                    AccessRule::DenyAll => {
                        rule = AccessRule::from(additional_rule);
                    }
                    AccessRule::Protected(rule_node) => {
                        rule = AccessRule::from(rule_node.or(additional_rule));
                    }
                }
                
                match level {
                    1 => self._set_level_1_auth(&mut account, rule),
                    2 => self._set_level_2_auth(&mut account, rule),
                    3 => self._set_level_3_auth(&mut account, rule),
                    _ => unreachable!(),
                }

                account.realize();
            })
        }

        pub fn set_level_1_auth(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            rule: AccessRule,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_1_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }
                
                self._set_level_1_auth(&mut account, rule);

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
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_1_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }

                self._set_level_2_auth(&mut account, rule);

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
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_1_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }

                self._set_level_3_auth(&mut account, rule);

                account.realize();
            })
        }

        pub fn add_collateral(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            tokens: Vec<Bucket>,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_3_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._add_collateral(&config, &mut pool, &mut account, tokens);
                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);

                    pool.realize();
                } else {
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                    let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());

                    self._add_collateral(&config, &mut pool, &mut account, tokens);

                    pool.realize();
                }

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
        ) -> ListIndex {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_2_auth();

                let config = if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                    config
                } else {
                    VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new())
                };

                assert!(
                    claims.len() <= 10,
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_CLAIMS_TOO_MANY, claims.len(), 10
                );

                let request = Request::RemoveCollateral(RequestRemoveCollateral {
                    target_account,
                    claims,
                });

                let request_index = account.requests_len();
                account.push_request(request, 0, expiry_seconds, STATUS_ACTIVE, vec![target_account]);
                self._assert_active_requests_limit(&config, &account);

                account.realize();

                request_index
            })
        }

        pub fn margin_order_request(
            &self,
            fee_oath: Option<Bucket>,
            delay_seconds: u64,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: PairId,
            amount: Decimal,
            reduce_only: bool,
            price_limit: PriceLimit,
            slippage_limit: SlippageLimit,
            activate_requests: Vec<RequestIndexRef>,
            cancel_requests: Vec<RequestIndexRef>,
            status: Status,
        ) -> ListIndex {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_3_auth();

                let config = if let Some(fee_oath) = fee_oath {
                    let mut pair_ids = account.position_ids();
                    pair_ids.insert(pair_id.clone());
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                    config
                } else {
                    VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::from([pair_id.clone()]))
                };

                let pair_config = config.pair_config(&pair_id);
                let amount_abs = amount.checked_abs().expect(ERROR_ARITHMETIC);
                assert!(
                    amount_abs >= pair_config.trade_size_min,
                    "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_TRADE_SIZE_MIN_NOT_MET, amount_abs, pair_config.trade_size_min
                );

                assert!(
                    activate_requests.len() <= 2,
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_EFFECTED_REQUESTS_TOO_MANY, activate_requests.len(), 2
                );        
                assert!(cancel_requests.len() <= 2,
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= ", ERROR_EFFECTED_REQUESTS_TOO_MANY, cancel_requests.len(), 2
                );

                let request_index = account.requests_len();
                let activate_requests = activate_requests.iter().map(|r| r.resolve(request_index)).collect();
                let cancel_requests = cancel_requests.iter().map(|r| r.resolve(request_index)).collect();
                let request = Request::MarginOrder(RequestMarginOrder {
                    pair_id,
                    amount,
                    reduce_only,
                    price_limit,
                    slippage_limit,
                    activate_requests,
                    cancel_requests,
                });

                let request_index = account.requests_len();
                account.push_request(request, delay_seconds, expiry_seconds, status, vec![]);
                self._assert_active_requests_limit(&config, &account);

                account.realize();

                request_index
            })
        }

        pub fn margin_order_tp_sl_request(
            &self,
            fee_oath: Option<Bucket>,
            delay_seconds: u64,
            expiry_seconds: u64,
            account: ComponentAddress,
            pair_id: PairId,
            amount: Decimal,
            reduce_only: bool,
            price_limit: PriceLimit,
            slippage_limit: SlippageLimit,
            price_tp: Option<Decimal>,
            price_sl: Option<Decimal>,
        ) -> (ListIndex, Option<ListIndex>, Option<ListIndex>) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_3_auth();

                let config = if let Some(fee_oath) = fee_oath {
                    let mut pair_ids = account.position_ids();
                    pair_ids.insert(pair_id.clone());
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                    config
                } else {
                    VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::from([pair_id.clone()]))
                };

                let pair_config = config.pair_config(&pair_id);
                let amount_abs = amount.checked_abs().expect(ERROR_ARITHMETIC);
                assert!(
                    amount_abs >= pair_config.trade_size_min,
                    "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_TRADE_SIZE_MIN_NOT_MET, amount_abs, pair_config.trade_size_min
                );

                let mut request_index = account.requests_len();
                let index_order = request_index;

                let mut activate_requests_order = vec![];
                let mut cancel_requests_tp = vec![];
                let mut cancel_requests_sl = vec![];

                let (price_limit_tp, index_tp) = if let Some(price) = price_tp {
                    let index_tp = request_index + 1;
                    request_index += 1;
                    activate_requests_order.push(index_tp);
                    cancel_requests_sl.push(index_tp);

                    let price_limit_tp = if amount.is_positive() {
                        PriceLimit::Gte(price)
                    } else {
                        PriceLimit::Lte(price)
                    };

                    (Some(price_limit_tp), Some(index_tp))
                } else {
                    (None, None)
                };
                let (price_limit_sl, index_sl) = if let Some(price) = price_sl {
                    let index_sl = request_index + 1;
                    activate_requests_order.push(index_sl);
                    cancel_requests_tp.push(index_sl);

                    let price_limit_sl = if amount.is_positive() {
                        PriceLimit::Lte(price)
                    } else {
                        PriceLimit::Gte(price)
                    };

                    (Some(price_limit_sl), Some(index_sl))
                } else {
                    (None, None)
                };

                let request_order = Request::MarginOrder(RequestMarginOrder {
                    pair_id: pair_id.clone(),
                    amount,
                    reduce_only,
                    price_limit,
                    slippage_limit,
                    activate_requests: activate_requests_order,
                    cancel_requests: vec![],
                });
                account.push_request(request_order, delay_seconds, expiry_seconds, STATUS_ACTIVE, vec![]);

                if let Some(price_limit_tp) = price_limit_tp {
                    let request_tp = Request::MarginOrder(RequestMarginOrder {
                        pair_id: pair_id.clone(),
                        amount: -amount,
                        reduce_only: true,
                        price_limit: price_limit_tp,
                        slippage_limit,
                        activate_requests: vec![],
                        cancel_requests: cancel_requests_tp,
                    });
                    account.push_request(request_tp, delay_seconds, expiry_seconds, STATUS_DORMANT, vec![]);
                }
                if let Some(price_limit_sl) = price_limit_sl {
                    let request_sl = Request::MarginOrder(RequestMarginOrder {
                        pair_id: pair_id.clone(),
                        amount: -amount,
                        reduce_only: true,
                        price_limit: price_limit_sl,
                        slippage_limit,
                        activate_requests: vec![],
                        cancel_requests: cancel_requests_sl,
                    });
                    account.push_request(request_sl, delay_seconds, expiry_seconds, STATUS_DORMANT, vec![]);
                }

                self._assert_active_requests_limit(&config, &account);

                account.realize();

                (index_order, index_tp, index_sl)
            })
        }

        pub fn cancel_requests(
            &self, 
            fee_oath: Option<Bucket>,
            account: ComponentAddress, 
            indexes: Vec<ListIndex>,
        ) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                account.verify_level_3_auth();

                if let Some(fee_oath) = fee_oath {
                    let pair_ids = account.position_ids();
                    let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                    let pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds_no_max_age(pair_ids), None);

                    self._settle_fee_oath(&config, &pool, &mut account, &oracle, fee_oath);
                }

                account.cancel_requests(indexes);
                account.realize();
            })
        }

        // --- KEEPER METHODS ---

        pub fn process_request(
            &self, 
            account: ComponentAddress, 
            index: ListIndex,
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> Bucket {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                let (request, expired) = account.process_request(index);
                
                let mut pair_ids = account.position_ids();
                if let Request::MarginOrder(request) = &request {
                    pair_ids.insert(request.pair_id.clone());
                }
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());

                if !expired {
                    let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                    let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds(&config, pair_ids.clone()), price_updates);

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
            resource: ResourceAddress, 
            payment: Bucket, 
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> (Bucket, Bucket) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), HashSet::new());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), HashSet::new());
                let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), HashMap::new(), price_updates);

                let (token, remainder) = self._swap_debt(&config, &mut pool, &mut account, &oracle, &resource, payment);
    
                account.realize();
                pool.realize();
    
                (token, remainder)
            })
        }

        pub fn liquidate(
            &self,
            account: ComponentAddress,
            payment: Bucket,
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> (Vec<Bucket>, Bucket) {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                let pair_ids = account.position_ids();
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds(&config, pair_ids.clone()), price_updates);

                let tokens = self._liquidate(&config, &mut pool, &mut account, &oracle, payment);

                account.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(config.exchange_config().reward_keeper);
                (tokens, reward)
            })
        }

        pub fn liquidate_v2(
            &self,
            account: ComponentAddress,
            receiver: ComponentAddress,
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> Bucket {
            authorize!(self, {
                assert!(
                    account != receiver,
                    "{}, VALUE:{}, REQUIRED:{}, OP:!= |", ERROR_LIQUIDATION_RECEIVER_SAME_AS_ACCOUNT, Runtime::bech32_encode_address(receiver), Runtime::bech32_encode_address(account)
                );

                let mut receiver = VirtualMarginAccount::new(receiver);
                receiver.verify_level_3_auth();

                let mut account = VirtualMarginAccount::new(account);
                let pair_ids: HashSet<PairId> = account.position_ids().union(&receiver.position_ids()).cloned().collect();
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());
                let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), self._pair_feeds(&config, pair_ids.clone()), price_updates);

                self._liquidate_v2(&config, &mut pool, &mut account, &mut receiver, &oracle);

                account.realize();
                receiver.realize();
                pool.realize();

                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(config.exchange_config().reward_keeper);
                reward
            })
        }

        pub fn auto_deleverage(
            &self, 
            account: ComponentAddress, 
            pair_id: PairId, 
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> Bucket {
            authorize!(self, {
                let mut account = VirtualMarginAccount::new(account);
                let mut pair_ids = account.position_ids();
                pair_ids.insert(pair_id.clone());
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());

                let pair_feeds = self._pair_feeds(&config, pair_ids.clone());
                let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), pair_feeds, price_updates);

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
            price_updates: Option<(Vec<u8>, Bls12381G2Signature, ListIndex)>,
        ) -> (Bucket, Vec<bool>) {
            authorize!(self, {
                let pair_ids: HashSet<PairId> = pair_ids.into_iter().collect();
                let config = VirtualConfig::new(Global::<Config>::from(CONFIG_COMPONENT), pair_ids.clone());
                let mut pool = VirtualLiquidityPool::new(Global::<MarginPool>::from(POOL_COMPONENT), pair_ids.clone());

                let pair_feeds = self._pair_feeds(&config, pair_ids.clone());
                let oracle = VirtualOracle::new(Global::<Oracle>::from(ORACLE_COMPONENT), config.collateral_feeds(), pair_feeds, price_updates);

                let rewarded: Vec<bool> = pair_ids.iter().map(|pair_id| {
                    self._update_pair(&config, &mut pool, &oracle, pair_id)
                }).collect();

                pool.realize();

                let rewards_count = rewarded.iter().filter(|r| **r).count();
                let reward = ResourceManager::from_address(KEEPER_REWARD_RESOURCE).mint(rewards_count);
                
                (reward, rewarded)
            })
        }

        // --- INTERNAL METHODS ---

        fn _pair_feeds(
            &self,
            config: &VirtualConfig,
            pair_ids: HashSet<PairId>,
        ) -> HashMap<PairId, i64> {
            pair_ids.into_iter().map(|pair_id| {
                let pair_config = config.pair_config(&pair_id);
                (pair_id, pair_config.price_age_max)
            }).collect()
        }

        fn _pair_feeds_no_max_age(
            &self,
            pair_ids: HashSet<PairId>,
        ) -> HashMap<PairId, i64> {
            pair_ids.into_iter().map(|pair_id| {
                (pair_id, i64::MAX)
            }).collect()
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

        fn _assert_lp_resource(
            &self,
            resource: &ResourceAddress,
        ) {
            assert!(
                *resource == LP_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_LP_TOKEN, Runtime::bech32_encode_address(*resource), Runtime::bech32_encode_address(LP_RESOURCE)
            );
        }
        
        fn _assert_base_resource(
            &self,
            resource: &ResourceAddress,
        ) {
            assert!(
                *resource == BASE_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT_TOKEN, Runtime::bech32_encode_address(*resource), Runtime::bech32_encode_address(BASE_RESOURCE)
            );
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
                account_value >= margin,
                "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_INSUFFICIENT_MARGIN, account_value, margin
            );
        }

        fn _assert_valid_collateral(
            &self, 
            config: &VirtualConfig,
            resource: ResourceAddress,
        ) {
            assert!(
                config.collateral_configs().contains_key(&resource),
                "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_INVALID_COLLATERAL, Runtime::bech32_encode_address(resource), 
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

        fn _assert_collaterals_limit(
            &self,
            config: &VirtualConfig,
            account: &VirtualMarginAccount,
        ) {
            let collaterals_len = account.collateral_amounts().len();
            let collaterals_max = config.exchange_config().collaterals_max as usize;
            assert!(
                collaterals_len <= collaterals_max,
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_COLLATERALS_TOO_MANY, collaterals_len, collaterals_max
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

        fn _calculate_fee(
            &self,
            exchange_config: &ExchangeConfig,
            pair_config: &PairConfig,
            pool_position: &PoolPosition,
            fee_rebate: Decimal,
            price: Decimal,
            value: Decimal, 
        ) -> Decimal {
            let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);
            let skew = (pool_position.oi_long - pool_position.oi_short) * price;

            let fee_0 = value_abs * pair_config.fee_0;
            let fee_1 = value * (dec!(2) * skew + value) * pair_config.fee_1;
            let fee_max = value_abs * exchange_config.fee_max;
            let fee = ((fee_0 + fee_1) * fee_rebate).clamp(dec!(0), fee_max);
            
            fee
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
                    -position.amount * (pool_position.funding_short_index - position.funding_index)            
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
                requests_len: account.requests_len(),
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

            PairDetails {
                pair_id: pair_id.clone(),
                pool_position: pool_position.clone(),
                pair_config: pair_config.clone(),
            }
        }

        fn _settle_fee_oath(
            &self,
            config: &VirtualConfig,
            pool: &VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
            fee_oath: Bucket,
        ) {
            let resource = fee_oath.resource_address();
            assert!(
                fee_oath.resource_address() == FEE_OATH_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_FEE_OATH, Runtime::bech32_encode_address(resource), Runtime::bech32_encode_address(FEE_OATH_RESOURCE)
            );

            let fee_value = fee_oath.amount();
            account.add_virtual_balance(-fee_value);
            fee_oath.burn();

            self._assert_account_integrity(config, pool, account, oracle);
        }

        fn _add_liquidity(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            payment: Bucket,
        ) -> Bucket {
            let lp_token_manager = ResourceManager::from_address(LP_RESOURCE);
            self._assert_base_resource(&payment.resource_address());

            let value = payment.amount();
            let fee = value * config.exchange_config().fee_liquidity_add;
            let pool_value = self._pool_value(pool).max(dec!(1));
            let lp_supply = lp_token_manager.total_supply().unwrap();

            let (lp_amount, lp_price) = if lp_supply.is_zero() {
                (value - fee, dec!(1))
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
            self._assert_lp_resource(&lp_token.resource_address());

            let lp_amount = lp_token.amount();
            let pool_value = self._pool_value(pool).max(dec!(0));
            let lp_supply = lp_token_manager.total_supply().unwrap();

            let lp_price = pool_value / lp_supply;
            let value = lp_amount * lp_price;
            let fee = value * config.exchange_config().fee_liquidity_remove;
            let withdraw_amount = value - fee;

            assert!(
                withdraw_amount <= pool.base_tokens_amount(),
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, withdraw_amount, pool.base_tokens_amount()
            );
            
            lp_token.burn();
            let token = pool.withdraw(withdraw_amount, TO_ZERO);
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

        fn _set_level_1_auth(
            &self,
            account: &mut VirtualMarginAccount,
            rule: AccessRule,
        ) {
            let permission_registry = Global::<PermissionRegistry>::from(PERMISSION_REGISTRY_COMPONENT);
            
            let old_rule = account.get_level_1_auth();
            let mut permissions = permission_registry.get_permissions(old_rule.clone());
            permissions.level_1.shift_remove(&account.address());
            permission_registry.set_permissions(old_rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(old_rule_nodes)) = old_rule {
                for old_rule_node in old_rule_nodes {
                    let old_sub_rule = AccessRule::from(old_rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(old_sub_rule.clone());
                    permissions.level_1.shift_remove(&account.address());
                    permission_registry.set_permissions(old_sub_rule, permissions);
                }
            }

            account.set_level_1_auth(rule.clone());

            let mut permissions = permission_registry.get_permissions(rule.clone());
            permissions.level_1.insert(account.address());
            permission_registry.set_permissions(rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes)) = rule {
                for rule_node in rule_nodes {
                    let sub_rule = AccessRule::from(rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(sub_rule.clone());
                    permissions.level_1.insert(account.address());
                    permission_registry.set_permissions(sub_rule, permissions);
                }
            }
        }

        fn _set_level_2_auth(
            &self,
            account: &mut VirtualMarginAccount,
            rule: AccessRule,
        ) {
            let permission_registry = Global::<PermissionRegistry>::from(PERMISSION_REGISTRY_COMPONENT);

            let old_rule = account.get_level_2_auth();
            let mut permissions = permission_registry.get_permissions(old_rule.clone());
            permissions.level_2.shift_remove(&account.address());
            permission_registry.set_permissions(old_rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(old_rule_nodes)) = old_rule {
                for old_rule_node in old_rule_nodes {
                    let old_sub_rule = AccessRule::from(old_rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(old_sub_rule.clone());
                    permissions.level_2.shift_remove(&account.address());
                    permission_registry.set_permissions(old_sub_rule, permissions);
                }
            }

            account.set_level_2_auth(rule.clone());

            let mut permissions = permission_registry.get_permissions(rule.clone());
            permissions.level_2.insert(account.address());
            permission_registry.set_permissions(rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes)) = rule {
                for rule_node in rule_nodes {
                    let sub_rule = AccessRule::from(rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(sub_rule.clone());
                    permissions.level_2.insert(account.address());
                    permission_registry.set_permissions(sub_rule, permissions);
                }
            }
        }

        fn _set_level_3_auth(
            &self,
            account: &mut VirtualMarginAccount,
            rule: AccessRule,
        ) {
            let permission_registry = Global::<PermissionRegistry>::from(PERMISSION_REGISTRY_COMPONENT);

            let old_rule = account.get_level_3_auth();
            let mut permissions = permission_registry.get_permissions(old_rule.clone());
            permissions.level_3.shift_remove(&account.address());
            permission_registry.set_permissions(old_rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(old_rule_nodes)) = old_rule {
                for old_rule_node in old_rule_nodes {
                    let old_sub_rule = AccessRule::from(old_rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(old_sub_rule.clone());
                    permissions.level_3.shift_remove(&account.address());
                    permission_registry.set_permissions(old_sub_rule, permissions);
                }
            }

            account.set_level_3_auth(rule.clone());

            let mut permissions = permission_registry.get_permissions(rule.clone());
            permissions.level_3.insert(account.address());
            permission_registry.set_permissions(rule.clone(), permissions);
            if let AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes)) = rule {
                for rule_node in rule_nodes {
                    let sub_rule = AccessRule::from(rule_node.clone());
                    let mut permissions = permission_registry.get_permissions(sub_rule.clone());
                    permissions.level_3.insert(account.address());
                    permission_registry.set_permissions(sub_rule, permissions);
                }
            }
        }

        fn _add_collateral(
            &self,
            config: &VirtualConfig, 
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            mut tokens: Vec<Bucket>,
        ) {
            let amounts = tokens.iter().map(|token| (token.resource_address(), token.amount())).collect();
            loop {
                if let Some(index) = tokens.iter().position(|token| token.resource_address() == BASE_RESOURCE) {
                    let base_token = tokens.remove(index);
                    let value = base_token.amount();
                    pool.deposit(base_token);
                    self._settle_account(pool, account, value);
                } else {
                    break;
                }
            }
            tokens.iter().for_each(|token| self._assert_valid_collateral(config, token.resource_address()));

            account.deposit_collateral_batch(tokens);
            self._assert_collaterals_limit(config, account);

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
            let mut claims: Vec<(ResourceAddress, Decimal)> = request.claims.iter()
                .fold(HashMap::new(), |mut claims, (resource, amount)| {
                    claims.entry(*resource).and_modify(|a| *a += *amount).or_insert(*amount);
                    claims
                }).into_iter().collect();

            let mut tokens = Vec::new();
            claims.retain(|(resource, amount)| {
                if *resource == BASE_RESOURCE {
                    assert!(
                        *amount <= account.virtual_balance(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_BALANCE, *amount, account.virtual_balance()
                    );
                    assert!(
                        *amount <= pool.base_tokens_amount(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS, *amount, pool.base_tokens_amount()
                    );

                    let base_token = pool.withdraw(*amount, TO_ZERO);
                    let value = base_token.amount();
                    self._settle_account(pool, account, -value);
                    tokens.push(base_token);
                    false
                } else {
                    assert!(
                        *amount <= account.collateral_amount(&resource),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_WITHDRAWAL_INSUFFICIENT_BALANCE, *amount, account.collateral_amount(&resource)
                    );

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
            let slippage_limit = request.slippage_limit;
            let activate_requests = request.activate_requests;
            let cancel_requests = request.cancel_requests;

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
            
            let trade_value = price * (amount_open + amount_close).checked_abs().expect(ERROR_ARITHMETIC);
            assert!(
                slippage_limit.compare(fee_paid, trade_value),
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_MARGIN_ORDER_SLIPPAGE_LIMIT, fee_paid, slippage_limit.allowed_slippage(trade_value),
            );

            let (fee_pool, fee_protocol, fee_treasury, fee_referral) = self._settle_fees_referral(config, pool, account, fee_paid);

            let activated_requests = account.try_set_keeper_requests_status(activate_requests, STATUS_ACTIVE);
            let cancelled_requests = account.try_set_keeper_requests_status(cancel_requests, STATUS_CANCELLED);
            
            let skew_1 = pool.skew_abs_snap();
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
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount, 
            oracle: &VirtualOracle,
            resource: &ResourceAddress, 
            mut payment_token: Bucket, 
        ) -> (Bucket, Bucket) {
            self._assert_base_resource(&payment_token.resource_address());      
            self._assert_valid_collateral(config, *resource);
            assert!(
                account.virtual_balance() < dec!(0),
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_SWAP_NO_DEBT, account.virtual_balance(), dec!(0)
            );

            let collateral_config = config.collateral_configs().get(resource).unwrap();
            let discount = collateral_config.discount * dec!(0.1) + dec!(0.9);

            let value = payment_token.amount().min(-account.virtual_balance());
            let price_resource = oracle.price_resource(*resource) * discount;
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
                amount: token.amount(),
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
            self._assert_base_resource(&payment_token.resource_address());

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
                "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_INSUFFICIENT_PAYMENT, payment_token.amount(), value
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
                funding: result_positions.funding_paid,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
                pool_loss,
            });

            tokens
        }

        fn _liquidate_v2(
            &self,
            config: &VirtualConfig,
            pool: &mut VirtualLiquidityPool,
            account: &mut VirtualMarginAccount,
            receiver: &mut VirtualMarginAccount,
            oracle: &VirtualOracle,
        ) {
            let result_positions = self._liquidate_positions(config, pool, account, oracle); 
            let result_collateral = self._liquidate_collateral(config, account, oracle); 
            
            let virtual_balance = account.virtual_balance();
            let account_value = result_positions.pnl + result_collateral.collateral_value_discounted + virtual_balance;
            let margin = result_positions.margin_positions + result_collateral.margin_collateral;

            assert!(
                account_value < margin,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_LIQUIDATION_SUFFICIENT_MARGIN, account_value, margin
            );

            let value = result_collateral.collateral_value_discounted;

            let tokens = account.withdraw_collateral_batch(result_collateral.collateral_amounts.clone(), TO_ZERO);
            receiver.deposit_collateral_batch(tokens);
            self._settle_account(pool, receiver, -value);
            self._assert_account_integrity(config, pool, receiver, oracle);

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
                funding: result_positions.funding_paid,
                fee_pool,
                fee_protocol,
                fee_treasury,
                fee_referral,
                pool_loss,
            });
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

            let price = oracle.price(pair_id);
            let value = amount * price;

            let pool_position = pool.position_mut(pair_id);
            let position = account.position_mut(pair_id);

            assert!(position.amount.0.signum() * amount.0.signum() >= I192::ZERO);
            
            let fee = self._calculate_fee(exchange_config, pair_config, pool_position, fee_rebate, price, value);
            let cost = value + fee;

            if amount.is_positive() {
                pool_position.oi_long += amount;
                assert!(
                    pool_position.oi_long <= pair_config.oi_max, 
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_PAIR_OI_TOO_HIGH, pool_position.oi_long, pair_config.oi_max
                );
            } else {
                pool_position.oi_short -= amount;
                assert!(
                    pool_position.oi_short <= pair_config.oi_max , 
                    "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_PAIR_OI_TOO_HIGH, pool_position.oi_short, pair_config.oi_max
                );
            }
            pool_position.cost += cost;
            
            position.amount += amount;
            position.cost += cost;
            if position.amount.is_positive() {
                position.funding_index = pool_position.funding_long_index
            } else {
                position.funding_index = pool_position.funding_short_index
            };

            self._update_pair_snaps(pool, oracle, pair_id);

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

            let pool_position = pool.position_mut(pair_id);
            let position = account.position_mut(pair_id);

            assert!(position.amount.0.signum() * amount.0.signum() <= I192::ZERO);
            assert!(position.amount.0.abs() >= amount.0.abs());

            let fee = self._calculate_fee(exchange_config, pair_config, pool_position, fee_rebate, price, value);
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
            self._update_pair_snaps(pool, oracle, pair_id);

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

            let pnl = pool_position.cost - skew;
            let pnl_snap_delta = pnl - pool_position.pnl_snap;
            pool_position.pnl_snap = pnl;
            
            let current_time = Clock::current_time_rounded_to_seconds();
            let period_seconds = current_time.seconds_since_unix_epoch - pool_position.last_update.seconds_since_unix_epoch;
            let period = Decimal::from(period_seconds) / dec!(31536000); // 1 year
            
            let price_delta_ratio = (price - pool_position.last_price).checked_abs().expect(ERROR_ARITHMETIC) / pool_position.last_price;
            pool_position.last_price = price;
            pool_position.last_update = current_time;
            
            let funding_pool_delta = if !period.is_zero() {
                let funding_2_max = oi_long * price;
                let funding_2_min = -oi_short * price;
                let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
                
                if pool_position.funding_2_rate > funding_2_max {
                    let excess = pool_position.funding_2_rate - funding_2_max;
                    let decay = excess * (pair_config.funding_2_decay * period).min(dec!(1));
                    pool_position.funding_2_rate -= decay;

                    if funding_2_rate_delta.is_negative() {
                        if pool_position.funding_2_rate + funding_2_rate_delta < funding_2_min {
                            pool_position.funding_2_rate = funding_2_min;
                        } else {
                            pool_position.funding_2_rate += funding_2_rate_delta;
                        }
                    }
                } else if pool_position.funding_2_rate < funding_2_min {
                    let excess = pool_position.funding_2_rate - funding_2_min;
                    let decay = excess * (pair_config.funding_2_decay * period).min(dec!(1));
                    pool_position.funding_2_rate -= decay;

                    if funding_2_rate_delta.is_positive() {
                        if pool_position.funding_2_rate + funding_2_rate_delta > funding_2_max {
                            pool_position.funding_2_rate = funding_2_max;
                        } else {
                            pool_position.funding_2_rate += funding_2_rate_delta;
                        }
                    }
                } else {
                    if pool_position.funding_2_rate + funding_2_rate_delta > funding_2_max {
                        pool_position.funding_2_rate = funding_2_max;
                    } else if pool_position.funding_2_rate + funding_2_rate_delta < funding_2_min {
                        pool_position.funding_2_rate = funding_2_min;
                    } else {
                        pool_position.funding_2_rate += funding_2_rate_delta;
                    }
                }

                if !oi_long.is_zero() && !oi_short.is_zero() {
                    let funding_1_rate = skew * pair_config.funding_1;
                    let funding_2_rate = pool_position.funding_2_rate.clamp(funding_2_min, funding_2_max) * pair_config.funding_2;
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
            
            let pnl = pool_position.cost - skew;
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
            let fee_rebate = account.fee_rebate();
            
            let mut total_pnl = dec!(0);
            let mut total_margin = dec!(0);
            for (pair_id, position) in account.positions().iter() {
                let pair_config = config.pair_config(pair_id);
                let price = oracle.price(pair_id);
                let amount = position.amount;
                let value = -amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position(pair_id);

                let fee = self._calculate_fee(exchange_config, pair_config, pool_position, fee_rebate, price, value);
                let cost = position.cost;
                let funding = if amount.is_positive() {
                    amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    -amount * (pool_position.funding_short_index - position.funding_index)
                };

                let pnl = -value - cost - fee - funding;
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
            let fee_rebate = account.fee_rebate();

            let pair_ids: Vec<PairId> = account.positions().keys().cloned().collect();
            pair_ids.iter().for_each(|pair_id| {
                self._update_pair(config, pool, oracle, pair_id);
            });
            
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
                let value = -amount * price;
                let value_abs = value.checked_abs().expect(ERROR_ARITHMETIC);

                let pool_position = pool.position_mut(pair_id);

                let fee = self._calculate_fee(exchange_config, pair_config, pool_position, fee_rebate, price, value);
                let cost = position.cost;
                let funding = if amount.is_positive() {
                    pool_position.oi_long -= amount;
                    amount * (pool_position.funding_long_index - position.funding_index)
                } else {
                    pool_position.oi_short += amount;
                    -amount * (pool_position.funding_short_index - position.funding_index)            
                };
                pool_position.cost -= cost;

                let pnl = -value - cost - fee - funding;
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
            pair_ids.iter().for_each(|pair_id| {
                self._update_pair_snaps(pool, oracle, pair_id);
            });

            ResultLiquidatePositions {
                pnl: total_pnl,
                margin_positions: total_margin,
                funding_paid: -total_funding,
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
                if let Some(collateral_config) = config.collateral_configs().get(&resource) {
                    let price_resource = oracle.price_resource(resource);
                    let value = amount * price_resource;
                    let value_discounted = value * collateral_config.discount;
                    let margin = value * collateral_config.margin;

                    total_value_discounted += value_discounted;
                    total_margin += margin;
                }
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
                if let Some(collateral_config) = config.collateral_configs().get(&resource) {
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
            let funding = if let Some(position) = account.positions_mut().get_mut(pair_id) {
                if position.amount.is_positive() {
                    let funding = position.amount * (pool_position.funding_long_index - position.funding_index);
                    position.funding_index = pool_position.funding_long_index;
                    funding
                } else {
                    let funding = -position.amount * (pool_position.funding_short_index - position.funding_index);
                    position.funding_index = pool_position.funding_short_index;
                    funding
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
            Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).distribute(fee_protocol, fee_treasury);

            (-fee_pool, -fee_protocol, -fee_treasury)
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
            Global::<FeeDistributor>::from(FEE_DISTRIBUTOR_COMPONENT).distribute(fee_protocol, fee_treasury);
            account.reward_referral(fee_referral);

            (-fee_pool, -fee_protocol, -fee_treasury, -fee_referral)
        }
    }
}
