use scrypto_test::prelude::*;
use super::*;

use ::common::*;
use exchange::*;

pub struct ExchangeInterface {
    pub public_key: Secp256k1PublicKey,
    pub test_account: ComponentAddress,
    pub resources: Resources,
    pub components: Components,
    pub ledger: LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
}

impl ExchangeInterface {
    pub fn new(
        public_key: Secp256k1PublicKey,
        account: ComponentAddress,
        resources: Resources, 
        components: Components, 
        ledger: LedgerSimulator<NoExtension, InMemorySubstateDatabase>
    ) -> Self {
        Self { 
            public_key,
            test_account: account,
            resources, 
            components, 
            ledger 
        }
    }

    // Useful helpers

    pub fn test_account_balance(
        &mut self, 
        resource: ResourceAddress,
    ) -> Decimal {
        self.ledger.get_component_balance(self.test_account, resource)
    }

    pub fn test_account_restrict_deposits(
        &mut self,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.test_account, 
                "set_default_deposit_rule",
                manifest_args!(DefaultDepositRule::Reject)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt.expect_commit_success();
    }

    pub fn test_account_add_authorized_depositor(
        &mut self,
        resource: ResourceAddress,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.test_account, 
                "add_authorized_depositor",
                manifest_args!(ResourceOrNonFungible::Resource(resource))
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt.expect_commit_success();
    }

    pub fn mint_test_token(
        &mut self,
        amount: Decimal,
        divisibility: u8,
    ) -> ResourceAddress {
        self.ledger.create_fungible_resource(amount, divisibility, self.test_account)
    }

    pub fn mint_test_nft(
        &mut self,
    ) -> (ResourceAddress, NonFungibleLocalId) {
        let resource = self.ledger.create_non_fungible_resource_advanced(NonFungibleResourceRoles::default(), self.test_account, 1);
        let id = NonFungibleLocalId::integer(1);

        (resource, id)
    }

    pub fn get_role(
        &mut self,
        component: ComponentAddress,
        role_module: ModuleId,
        role_name: &str,
    ) -> Option<AccessRule> {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_role(component, role_module, RoleKey::new(role_name))
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt.expect_commit_success().output(1)
    }

    pub fn ledger_time(
        &mut self,
    ) -> Instant {
        self.ledger.get_current_time(TimePrecisionV2::Second)
    }

    pub fn increment_ledger_time(
        &mut self,
        seconds: i64,
    ) -> Instant {
        let current_time = self.ledger.get_current_time(TimePrecisionV2::Second);
        let new_time = current_time.add_seconds(seconds).unwrap();
        set_time(new_time, &mut self.ledger);
        new_time
    }

    pub fn parse_event<T: ScryptoEvent>(
        &mut self,
        result: &CommitResult,
    ) -> T {
        result.application_events
            .iter()
            .find_map(|(event_type_identifier, event_data)| {
                if self.ledger.is_event_name_equal::<T>(event_type_identifier) {
                    Some(scrypto_decode::<T>(event_data).unwrap())
                } else {
                    None
                }
            }).unwrap()
    }

    pub fn get_pool_value(
        &mut self,
    ) -> Decimal {
        let pool_details = self.get_pool_details();
        pool_details.base_tokens_amount + pool_details.virtual_balance + pool_details.unrealized_pool_funding + pool_details.pnl_snap
    }

    pub fn make_open_interest(
        &mut self,
        pair_id: PairId,
        amount_long: Decimal,
        amount_short: Decimal,
        price: Decimal,
    ) {
        let result = self.create_account(
            rule!(allow_all), 
            vec![(self.resources.base_resource, dec!(1000000))], 
            None,
        ).expect_commit_success().clone();
        let margin_account_component = result.new_component_addresses()[0];
        self.margin_order_request(
            0, 
            1, 
            margin_account_component, 
            pair_id.clone(), 
            amount_long, 
            false, 
            Limit::None, 
            vec![], 
            vec![], 
            STATUS_ACTIVE
        ).expect_commit_success();
        self.margin_order_request(
            0, 
            1, 
            margin_account_component, 
            pair_id.clone(), 
            -amount_short, 
            false, 
            Limit::None, 
            vec![], 
            vec![], 
            STATUS_ACTIVE
        ).expect_commit_success();
        let time = self.increment_ledger_time(1);
            self.process_request(
            margin_account_component,
            0, 
            Some(vec![
                Price {
                    pair: pair_id,
                    quote: price,
                    timestamp: time,
                },
            ])
        ).expect_commit_success();
    }

    // Core exchange methods

    pub fn update_exchange_config(
        &mut self,
        exchange_config: ExchangeConfig,
    ) -> TransactionReceiptV1 {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "update_exchange_config", 
            manifest_args!(exchange_config)
        );
        receipt
    }

    pub fn update_pair_configs(
        &mut self, 
        pair_configs: Vec<PairConfig>
    ) -> TransactionReceiptV1 {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "update_pair_configs", 
            manifest_args!(pair_configs)
        );
        receipt
    }

    pub fn update_collateral_configs(
        &mut self,
        collateral_configs: Vec<(ResourceAddress, CollateralConfig)>
    ) -> TransactionReceiptV1 {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "update_collateral_configs", 
            manifest_args!(collateral_configs)
        );
        receipt
    }

    pub fn remove_collateral_configs(
        &mut self,
        collateral_configs: Vec<CollateralConfig>
    ) -> TransactionReceiptV1 {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "remove_collateral_configs", 
            manifest_args!(collateral_configs)
        );
        receipt
    }

    pub fn collect_treasury(
        &mut self,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "collect_treasury", 
                manifest_args!()
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn collect_fee_delegator(
        &mut self,
        fee_delegator: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "collect_fee_delegator", 
                manifest_args!(fee_delegator)
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn mint_referral(
        &mut self,
        fee_referral: Decimal,
        fee_rebate: Decimal,
        max_referrals: u64,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "mint_referral", 
                manifest_args!(fee_referral, fee_rebate, max_referrals)
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn mint_referral_with_allocation(
        &mut self,
        fee_referral: Decimal,
        fee_rebate: Decimal,
        max_referrals: u64,
        allocation_tokens: Vec<(ResourceAddress, Decimal)>,
        allocation_claims: Vec<(ResourceAddress, Decimal)>,
        allocation_count: u64,
    ) -> TransactionReceiptV1 {
        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        let mut bucket_names = vec![];
        for (i, token) in allocation_tokens.into_iter().enumerate() {
            let bucket_name = format!("token{}", i);
            bucket_names.push(bucket_name.clone());
            builder = builder 
                .withdraw_from_account(self.test_account, token.0, token.1)
                .take_all_from_worktop(self.resources.base_resource, bucket_name);
        }
        let manifest = builder
            .with_name_lookup(|manifest, lookup| {
                let buckets: Vec<ManifestBucket> = bucket_names.into_iter().map(|n| lookup.bucket(n)).collect();
                manifest.call_method(
                    self.components.exchange_component, 
                    "mint_referral_with_allocation", 
                    manifest_args!(fee_referral, fee_rebate, max_referrals, buckets, allocation_claims, allocation_count)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn update_referral(
        &mut self,
        referral_id: NonFungibleLocalId,
        fee_referral: Option<Decimal>,
        fee_rebate: Option<Decimal>,
        max_referrals: Option<u64>,
    ) -> TransactionReceiptV1 {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "update_referral", 
            manifest_args!(referral_id, fee_referral, fee_rebate, max_referrals)
        );
        receipt
    }

    pub fn add_referral_allocation(
        &mut self,
        referral_id: NonFungibleLocalId,
        allocation_tokens: Vec<(ResourceAddress, Decimal)>,
        allocation_claims: Vec<(ResourceAddress, Decimal)>,
        allocation_count: u64,
    ) -> TransactionReceiptV1 {
        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        let mut bucket_names = vec![];
        for (i, token) in allocation_tokens.into_iter().enumerate() {
            let bucket_name = format!("token{}", i);
            bucket_names.push(bucket_name.clone());
            builder = builder 
                .withdraw_from_account(self.test_account, token.0, token.1)
                .take_all_from_worktop(self.resources.base_resource, bucket_name);
        }
        let manifest = builder
            .with_name_lookup(|manifest, lookup| {
                let buckets: Vec<ManifestBucket> = bucket_names.into_iter().map(|n| lookup.bucket(n)).collect();
                manifest.call_method(
                    self.components.exchange_component, 
                    "add_referral_allocation", 
                    manifest_args!(referral_id, buckets, allocation_claims, allocation_count)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn get_pairs(
        &mut self,
        n: ListIndex,
        start: Option<ListIndex>,
    ) -> Vec<PairConfig> {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_pairs", 
            manifest_args!(n, start)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_permissions(
        &mut self,
        access_rule: AccessRule,
    ) -> Permissions {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_permissions", 
            manifest_args!(access_rule)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_account_details(
        &mut self,
        margin_account_component: ComponentAddress,
        history_n: ListIndex,
        history_start: Option<ListIndex>,
    ) -> AccountDetails {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_account_details", 
            manifest_args!(margin_account_component, history_n, history_start)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_pool_details(
        &mut self,
    ) -> PoolDetails {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_pool_details", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_pair_details(
        &mut self,
        pair_ids: Vec<PairId>,
    ) -> Vec<PairDetails> {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_pair_details", 
            manifest_args!(pair_ids)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_referral_details(
        &mut self,
        referral_id: NonFungibleLocalId,
    ) -> ReferralDetails {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_referral_details", 
            manifest_args!(referral_id)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_exchange_config(
        &mut self,
    ) -> ExchangeConfig {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_exchange_config", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_pair_configs(
        &mut self,
        n: ListIndex,
        start: Option<ListIndex>,
    ) -> Vec<PairConfig> {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_pair_configs", 
            manifest_args!(n, start)
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_pair_configs_len(
        &mut self,
    ) -> ListIndex {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, "get_pair_configs_len", manifest_args!());
        receipt.expect_commit_success().output(1)
    }

    pub fn get_collateral_configs(
        &mut self,
    ) -> HashMap<ResourceAddress, CollateralConfig> {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_collateral_configs", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_collaterals(
        &mut self,
    ) -> Vec<ResourceAddress> {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_collaterals", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_protocol_balance(
        &mut self,
    ) -> Decimal {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_protocol_balance", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }

    pub fn get_treasury_balance(
        &mut self,
    ) -> Decimal {
        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "get_treasury_balance", 
            manifest_args!()
        );
        receipt.expect_commit_success().output(1)
    }
    
    pub fn add_liquidity(
        &mut self,
        token: (ResourceAddress, Decimal)
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.test_account, token.0, token.1)
            .take_all_from_worktop(token.0, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "add_liquidity", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn remove_liquidity(
        &mut self,
        token: (ResourceAddress, Decimal)
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.test_account, token.0, token.1)
            .take_all_from_worktop(token.0, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "remove_liquidity", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn create_referral_codes(
        &mut self,
        referral_proof: (ResourceAddress, NonFungibleLocalId),
        tokens: Vec<(ResourceAddress, Decimal)>,
        referral_hashes: HashMap<Hash, (Vec<(ResourceAddress, Decimal)>, u64)>,
    ) -> TransactionReceiptV1 {
        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                self.test_account, 
                NonFungibleGlobalId::new(referral_proof.0, referral_proof.1)
            )
            .pop_from_auth_zone("referral");
        let mut bucket_names = vec![];
        for (i, token) in tokens.into_iter().enumerate() {
            let bucket_name = format!("token{}", i);
            bucket_names.push(bucket_name.clone());
            builder = builder 
                .withdraw_from_account(self.test_account, token.0, token.1)
                .take_all_from_worktop(token.0, bucket_name);
        }
        let manifest = builder
            .with_name_lookup(|manifest, lookup| {
                let buckets: Vec<ManifestBucket> = bucket_names.into_iter().map(|n| lookup.bucket(n)).collect();
                manifest.call_method(
                    self.components.exchange_component, 
                    "create_referral_codes", 
                    manifest_args!(lookup.proof("referral"), buckets, referral_hashes)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn create_referral_codes_from_allocation(
        &mut self,
        referral_proof: (ResourceAddress, NonFungibleLocalId),
        allocation_index: ListIndex,
        referral_hashes: HashMap<Hash, u64>,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                self.test_account, 
                NonFungibleGlobalId::new(referral_proof.0, referral_proof.1)
            )
            .pop_from_auth_zone("referral")
            .with_name_lookup(|manifest, lookup| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "create_referral_codes_from_allocation", 
                    manifest_args!(lookup.proof("referral"), allocation_index, referral_hashes)
                )
            })
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn collect_referral_rewards(
        &mut self,
        referral_proof: (ResourceAddress, NonFungibleLocalId),
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                self.test_account, 
                NonFungibleGlobalId::new(referral_proof.0, referral_proof.1)
            )
            .pop_from_auth_zone("referral")
            .with_name_lookup(|manifest, lookup| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "collect_referral_rewards", 
                    manifest_args!(lookup.proof("referral"))
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn create_account(
        &mut self,
        initial_rule: AccessRule,
        tokens: Vec<(ResourceAddress, Decimal)>,
        referral_code: Option<String>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;
        let reservation: Option<ManifestAddressReservation> = None;

        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        let mut bucket_names = vec![];
        for (i, token) in tokens.into_iter().enumerate() {
            let bucket_name = format!("token{}", i);
            bucket_names.push(bucket_name.clone());
            builder = builder 
                .withdraw_from_account(self.test_account, token.0, token.1)
                .take_all_from_worktop(token.0, bucket_name);
        }
        let manifest = builder
            .with_name_lookup(|manifest, lookup| {
                let buckets: Vec<ManifestBucket> = bucket_names.into_iter().map(|n| lookup.bucket(n)).collect();
                manifest.call_method(
                    self.components.exchange_component, 
                    "create_account", 
                    manifest_args!(
                        fee_oath,
                        initial_rule,
                        buckets,
                        referral_code,
                        reservation,
                    )
                )
            })    
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn set_level_1_auth(
        &mut self,
        proof: Option<(ResourceAddress, NonFungibleLocalId)>,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        if let Some(proof) = proof {
            builder = builder
                .create_proof_from_account_of_non_fungible(
                    self.test_account, 
                    NonFungibleGlobalId::new(proof.0, proof.1)
                );
        }
        let manifest = builder
            .call_method(
                self.components.exchange_component, 
                "set_level_1_auth", 
                manifest_args!(fee_oath, margin_account_component, rule)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn set_level_2_auth(
        &mut self,
        proof: Option<(ResourceAddress, NonFungibleLocalId)>,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        if let Some(proof) = proof {
            builder = builder
                .create_proof_from_account_of_non_fungible(
                    self.test_account, 
                    NonFungibleGlobalId::new(proof.0, proof.1)
                );
        }
        let manifest = builder
            .call_method(
                self.components.exchange_component, 
                "set_level_2_auth", 
                manifest_args!(fee_oath, margin_account_component, rule)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn set_level_3_auth(
        &mut self,
        proof: Option<(ResourceAddress, NonFungibleLocalId)>,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        if let Some(proof) = proof {
            builder = builder
                .create_proof_from_account_of_non_fungible(
                    self.test_account, 
                    NonFungibleGlobalId::new(proof.0, proof.1)
                );
        }
        let manifest = builder
            .call_method(
                self.components.exchange_component, 
                "set_level_3_auth", 
                manifest_args!(fee_oath, margin_account_component, rule)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn add_collateral(
        &mut self,
        margin_account_component: ComponentAddress,
        tokens: Vec<(ResourceAddress, Decimal)>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let mut builder = ManifestBuilder::new()
            .lock_fee_from_faucet();
        let mut bucket_names = vec![];
        for (i, token) in tokens.into_iter().enumerate() {
            let bucket_name = format!("token{}", i);
            bucket_names.push(bucket_name.clone());
            builder = builder 
                .withdraw_from_account(self.test_account, token.0, token.1)
                .take_all_from_worktop(token.0, bucket_name);
        }
        let manifest = builder
            .with_name_lookup(|manifest, lookup| {
                let buckets: Vec<ManifestBucket> = bucket_names.into_iter().map(|n| lookup.bucket(n)).collect();
                manifest.call_method(
                    self.components.exchange_component, 
                    "add_collateral", 
                    manifest_args!(
                        fee_oath,
                        margin_account_component, 
                        buckets
                    )
                )
            })    
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn remove_collateral_request(
        &mut self,
        expiry_seconds: u64,
        margin_account_component: ComponentAddress,
        target_account: ComponentAddress,
        claims: Vec<(ResourceAddress, Decimal)>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "remove_collateral_request", 
                manifest_args!(fee_oath, expiry_seconds, margin_account_component, target_account, claims)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn margin_order_request(
        &mut self,
        delay_seconds: u64,
        expiry_seconds: u64,
        margin_account_component: ComponentAddress,
        pair_id: PairId,
        amount: Decimal,
        reduce_only: bool,
        price_limit: Limit,
        activate_requests: Vec<ListIndex>,
        cancel_requests: Vec<ListIndex>,
        status: Status,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "margin_order_request", 
                manifest_args!(
                    fee_oath,
                    delay_seconds,
                    expiry_seconds,
                    margin_account_component,
                    pair_id,
                    amount,
                    reduce_only,
                    price_limit,
                    activate_requests,
                    cancel_requests,
                    status,
                )
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn margin_order_tp_sl_request(
        &mut self,
        delay_seconds: u64,
        expiry_seconds: u64,
        margin_account_component: ComponentAddress,
        pair_id: PairId,
        amount: Decimal,
        reduce_only: bool,
        price_limit: Limit,
        price_tp: Option<Decimal>,
        price_sl: Option<Decimal>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "margin_order_tp_sl_request", 
                manifest_args!(
                    fee_oath, 
                    delay_seconds, 
                    expiry_seconds, 
                    margin_account_component, 
                    pair_id, 
                    amount, 
                    reduce_only, 
                    price_limit, 
                    price_tp, 
                    price_sl
                )
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn cancel_request(
        &mut self,
        margin_account_component: ComponentAddress,
        index: ListIndex,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "cancel_request", 
                manifest_args!(fee_oath, margin_account_component, index)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn cancel_requests(
        &mut self,
        margin_account_component: ComponentAddress,
        indexes: Vec<ListIndex>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "cancel_requests", 
                manifest_args!(fee_oath, margin_account_component, indexes)
            )
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn process_request(
        &mut self,
        margin_account_component: ComponentAddress,
        index: ListIndex,
        prices: Option<Vec<Price>>,
    ) -> TransactionReceiptV1 {
        let price_updates = if let Some(prices) = prices {
            let price_data = scrypto_encode(&prices).unwrap();
            let price_data_hash = keccak256_hash(&price_data).to_vec();
            let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
            Some((price_data, price_signature, 0 as ListIndex))
        } else {
            None
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "process_request", 
                manifest_args!(
                    margin_account_component, 
                    index, 
                    price_updates
                )
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn swap_debt(
        &mut self,
        margin_account_component: ComponentAddress,
        resource: ResourceAddress,
        payment_amount: Decimal,
        prices: Option<Vec<Price>>,
    ) -> TransactionReceiptV1 {
        let price_updates = if let Some(prices) = prices {
            let price_data = scrypto_encode(&prices).unwrap();
            let price_data_hash = keccak256_hash(&price_data).to_vec();
            let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
            Some((price_data, price_signature, 0 as ListIndex))
        } else {
            None
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.test_account, self.resources.base_resource, payment_amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "swap_debt", 
                    manifest_args!(margin_account_component, resource, bucket, price_updates)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn liquidate(
        &mut self,
        margin_account_component: ComponentAddress,
        payment_amount: Decimal,
        prices: Option<Vec<Price>>,
    ) -> TransactionReceiptV1 {
        let price_updates = if let Some(prices) = prices {
            let price_data = scrypto_encode(&prices).unwrap();
            let price_data_hash = keccak256_hash(&price_data).to_vec();
            let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
            Some((price_data, price_signature, 0 as ListIndex))
        } else {
            None
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.test_account, self.resources.base_resource, payment_amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "liquidate", 
                    manifest_args!(margin_account_component, bucket, price_updates)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn auto_deleverage(
        &mut self,
        margin_account_component: ComponentAddress,
        pair_id: PairId,
        prices: Option<Vec<Price>>,
    ) -> TransactionReceiptV1 {
        let price_updates = if let Some(prices) = prices {
            let price_data = scrypto_encode(&prices).unwrap();
            let price_data_hash = keccak256_hash(&price_data).to_vec();
            let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
            Some((price_data, price_signature, 0 as ListIndex))
        } else {
            None
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "auto_deleverage", 
                manifest_args!(margin_account_component, pair_id, price_updates)
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn update_pairs(
        &mut self,
        pair_ids: Vec<PairId>,
        prices: Option<Vec<Price>>,
    ) -> TransactionReceiptV1 {
        let price_updates = if let Some(prices) = prices {
            let price_data = scrypto_encode(&prices).unwrap();
            let price_data_hash = keccak256_hash(&price_data).to_vec();
            let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
            Some((price_data, price_signature, 0 as ListIndex))
        } else {
            None
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "update_pairs", 
                manifest_args!(pair_ids, price_updates)
            )
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn swap_protocol_fee(
        &mut self,
        payment_amount: Decimal,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.test_account, self.resources.protocol_resource, payment_amount)
            .take_all_from_worktop(self.resources.protocol_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "swap_protocol_fee", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.test_account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }
}
