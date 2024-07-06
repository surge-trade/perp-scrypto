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

}
