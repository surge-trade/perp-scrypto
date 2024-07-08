use scrypto_test::prelude::*;
use super::*;

use ::common::*;
use config::*;
use account::*;
use exchange::*;
use oracle::*;

pub struct ExchangeInterface {
    pub public_key: Secp256k1PublicKey,
    pub account: ComponentAddress,
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
            account,
            resources, 
            components, 
            ledger 
        }
    }

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
        collateral_configs: Vec<CollateralConfig>
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
            .deposit_batch(self.account)
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
            .deposit_batch(self.account)
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
            .deposit_batch(self.account)
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
    ) -> Vec<AccessRule> {
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
            manifest_args!(margin_account_component)
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
        amount: Decimal,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.account, self.resources.base_resource, amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "add_liquidity", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn remove_liquidity(
        &mut self,
        amount: Decimal,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.account, self.resources.base_resource, amount)
            .take_all_from_worktop(self.resources.lp_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "remove_liquidity", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn create_referral_codes(
        &mut self,
        referral_id: NonFungibleLocalId,
        usd_amount: Decimal,
        referral_hashes: Vec<(Hash, Vec<(ResourceAddress, Decimal)>, u64)>,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                self.account, 
                NonFungibleGlobalId::new(self.resources.referral_resource, referral_id)
            )
            .pop_from_auth_zone("referral")
            .withdraw_from_account(self.account, self.resources.base_resource, usd_amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_name_lookup(|manifest, lookup| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "create_referral_codes", 
                    manifest_args!(lookup.proof("referral"), lookup.bucket("token"), referral_hashes)
                )
            })
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn collect_referral_rewards(
        &mut self,
        referral_id: NonFungibleLocalId,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                self.account, 
                NonFungibleGlobalId::new(self.resources.referral_resource, referral_id)
            )
            .pop_from_auth_zone("referral")
            .with_name_lookup(|manifest, lookup| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "collect_referral_rewards", 
                    manifest_args!(lookup.proof("referral"))
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn create_account(
        &mut self,
        initial_rule: AccessRule,
        preload_usd_amount: Decimal,
        referral_code: Option<String>,
    ) -> ComponentAddress {
        let fee_oath: Option<ManifestBucket> = None;
        let token: (ResourceAddress, Decimal)  = (self.resources.base_resource, preload_usd_amount);
        let reservation: Option<ManifestAddressReservation> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.account, token.0, token.1)
            .take_all_from_worktop(token.0, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "create_account", 
                    manifest_args!(
                        fee_oath,
                        initial_rule,
                        vec![bucket],
                        referral_code,
                        reservation,
                    )
                )
            })    
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt.expect_commit_success();
        let margin_account_component = receipt.expect_commit_success().new_component_addresses()[0];
        
        margin_account_component
    }

    pub fn set_level_1_auth(
        &mut self,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "set_level_1_auth", 
            manifest_args!(fee_oath, margin_account_component, rule)
        );
        receipt
    }

    pub fn set_level_2_auth(
        &mut self,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "set_level_2_auth", 
            manifest_args!(fee_oath, margin_account_component, rule)
        );
        receipt
    }

    pub fn set_level_3_auth(
        &mut self,
        margin_account_component: ComponentAddress,
        rule: AccessRule,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "set_level_3_auth", 
            manifest_args!(fee_oath, margin_account_component, rule)
        );
        receipt
    }

    pub fn add_collateral(
        &mut self,
        public_key: Secp256k1PublicKey,
        account: ComponentAddress,
        margin_account_component: ComponentAddress,
        token: (ResourceAddress, Decimal),
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, token.0, token.1)
            .take_all_from_worktop(token.0, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "add_collateral", 
                    manifest_args!(
                        fee_oath,
                        margin_account_component, 
                        vec![bucket]
                    )
                )
            })    
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
        receipt
    }

    pub fn remove_collateral_request(
        &mut self,
        expiry_seconds: u64,
        margin_account_component: ComponentAddress,
        claims: Vec<(ResourceAddress, Decimal)>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;
        let target_account: ComponentAddress = self.account;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "remove_collateral_request", 
            manifest_args!(fee_oath, expiry_seconds, margin_account_component, target_account, claims)
        );
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

        let receipt = self.ledger.call_method(
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
        );
        receipt
    }

    pub fn cancel_request(
        &mut self,
        margin_account_component: ComponentAddress,
        index: ListIndex,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "cancel_request", 
            manifest_args!(fee_oath, margin_account_component, index)
        );
        receipt
    }

    pub fn cancel_requests(
        &mut self,
        margin_account_component: ComponentAddress,
        indexes: Vec<ListIndex>,
    ) -> TransactionReceiptV1 {
        let fee_oath: Option<ManifestBucket> = None;

        let receipt = self.ledger.call_method(
            self.components.exchange_component, 
            "cancel_requests", 
            manifest_args!(fee_oath, margin_account_component, indexes)
        );
        receipt
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

    pub fn process_request(
        &mut self,
        margin_account_component: ComponentAddress,
        index: ListIndex,
        prices: Vec<Price>,
    ) -> TransactionReceiptV1 {
        let price_data = scrypto_encode(&prices).unwrap();
        let price_data_hash = keccak256_hash(&price_data).to_vec();
        let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
        let price_updates = Some((price_data, price_signature));
    
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
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn swap_debt(
        &mut self,
        margin_account_component: ComponentAddress,
        resource: ResourceAddress,
        payment_amount: Decimal,
        prices: Vec<Price>,
    ) -> TransactionReceiptV1 {
        let price_data = scrypto_encode(&prices).unwrap();
        let price_data_hash = keccak256_hash(&price_data).to_vec();
        let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
        let price_updates = Some((price_data, price_signature));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.account, self.resources.base_resource, payment_amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "swap_debt", 
                    manifest_args!(margin_account_component, resource, bucket, price_updates)
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn liquidate(
        &mut self,
        margin_account_component: ComponentAddress,
        payment_amount: Decimal,
        prices: Vec<Price>,
    ) -> TransactionReceiptV1 {
        let price_data = scrypto_encode(&prices).unwrap();
        let price_data_hash = keccak256_hash(&price_data).to_vec();
        let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
        let price_updates = Some((price_data, price_signature));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(self.account, self.resources.base_resource, payment_amount)
            .take_all_from_worktop(self.resources.base_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "liquidate", 
                    manifest_args!(margin_account_component, bucket, price_updates)
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn auto_deleverage(
        &mut self,
        margin_account_component: ComponentAddress,
        pair_id: PairId,
        prices: Vec<Price>,
    ) -> TransactionReceiptV1 {
        let price_data = scrypto_encode(&prices).unwrap();
        let price_data_hash = keccak256_hash(&price_data).to_vec();
        let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
        let price_updates = Some((price_data, price_signature));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "auto_deleverage", 
                manifest_args!(margin_account_component, pair_id, price_updates)
            )
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }

    pub fn update_pairs(
        &mut self,
        pair_ids: Vec<PairId>,
        prices: Vec<Price>,
    ) -> TransactionReceiptV1 {
        let price_data = scrypto_encode(&prices).unwrap();
        let price_data_hash = keccak256_hash(&price_data).to_vec();
        let price_signature = Bls12381G1PrivateKey::from_u64(self.components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
        let price_updates = Some((price_data, price_signature));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.components.exchange_component, 
                "update_pairs", 
                manifest_args!(pair_ids, price_updates)
            )
            .deposit_batch(self.account)
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
            .withdraw_from_account(self.account, self.resources.protocol_resource, payment_amount)
            .take_all_from_worktop(self.resources.protocol_resource, "token")
            .with_bucket("token", |manifest, bucket| {
                manifest.call_method(
                    self.components.exchange_component, 
                    "swap_protocol_fee", 
                    manifest_args!(bucket)
                )
            })
            .deposit_batch(self.account)
            .build();
        let receipt = self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&self.public_key)]);
        receipt
    }
}
