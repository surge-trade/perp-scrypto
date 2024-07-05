use scrypto_test::prelude::*;
use radix_engine::system::system_db_reader::SystemDatabaseWriter;
use ::common::*;
use config::*;
use account::*;
use exchange::*;
use oracle::*;
use once_cell::sync::Lazy;

mod common;

use crate::common::*;

static SETUP: Lazy<(LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components)> = Lazy::new(initialize_setup);

fn initialize_setup() -> (LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components) {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    let resources = create_resources(account, &mut ledger);
    let components = create_components(account, public_key, &resources, &mut ledger);   
    let snapshot = ledger.create_snapshot();

    (snapshot, public_key, account, resources, components)
}

fn get_setup() -> (LedgerSimulator<NoExtension, InMemorySubstateDatabase>, Secp256k1PublicKey, ComponentAddress, Resources, Components) {
    let snapshot = SETUP.0.clone();
    let public_key = SETUP.1.clone();
    let account = SETUP.2.clone();
    let resources = SETUP.3.clone();
    let components = SETUP.4.clone();

    let ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build_from_snapshot(snapshot);

    (ledger, public_key, account, resources, components)
}

fn set_time(ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>, time: Instant) {
    let substate_db = ledger.substate_db_mut();
    let substate = ProposerMilliTimestampSubstate {
        epoch_milli: time.seconds_since_unix_epoch * 1000,
    };

    let mut writer = SystemDatabaseWriter::new(substate_db);
    writer
        .write_typed_object_field(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMilliTimestamp.field_index(),
            ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(substate),
        )
        .unwrap();
}

#[test]
fn test_hello() {
    let (mut ledger, public_key, account, resources, components) = get_setup();

    let pair_configs = vec![PairConfig {
        pair_id: "BTC/USD".into(),
        disabled: false,
        update_price_delta_ratio: dec!(0.001),
        update_period_seconds: 600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0),
        funding_2: dec!(0),
        funding_2_delta: dec!(0),
        funding_pool_0: dec!(0),
        funding_pool_1: dec!(0),
        funding_share: dec!(0),
        fee_0: dec!(0.001),
        fee_1: dec!(0),
    }];

    let receipt = ledger.call_method(
        components.exchange_component, 
        "update_pair_configs", 
        manifest_args!(pair_configs)
    );
    receipt.expect_commit_success();

    let fee_oath: Option<ManifestBucket> = None;
    let initial_rule: AccessRule = rule!(allow_all);
    let token: (ResourceAddress, Decimal)  = (resources.base_resource, dec!(1000));
    let referral_code: Option<String> = None;
    let reservation: Option<ManifestAddressReservation> = None;

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, token.0, token.1)
        .take_all_from_worktop(token.0, "token")
        .with_bucket("token", |manifest, bucket| {
            manifest.call_method(
                components.exchange_component, 
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
    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    receipt.expect_commit_success();
    let margin_account_component = receipt.expect_commit_success().new_component_addresses()[0];
    
    let fee_oath: Option<ManifestBucket> = None;
    let token: (ResourceAddress, Decimal)  = (resources.base_resource, dec!(1000));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, token.0, token.1)
        .take_all_from_worktop(token.0, "token")
        .with_bucket("token", |manifest, bucket| {
            manifest.call_method(
                components.exchange_component, 
                "add_collateral", 
                manifest_args!(
                    fee_oath,
                    margin_account_component, 
                    vec![bucket]
                )
            )
        })    
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    receipt.expect_commit_success();

    let fee_oath: Option<ManifestBucket> = None;
    let delay_seconds: u64 = 0;
    let expiry_seconds: u64 = 10000000000;
    let pair_id: PairId = "BTC/USD".into();
    let amount: Decimal = dec!(0.0000000001);
    let reduce_only: bool = false;
    let price_limit: Limit = Limit::None;
    let activate_requests: Vec<ListIndex> = vec![];
    let cancel_requests: Vec<ListIndex> = vec![];
    let status: Status = STATUS_ACTIVE;

    let receipt = ledger.call_method(
        components.exchange_component, 
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
    receipt.expect_commit_success();

    let index = 0u64;
    let prices = vec![
        Price {
            pair: "BTC/USD".into(),
            quote: dec!(60000),
            timestamp: ledger.get_current_time(TimePrecisionV2::Second).add_seconds(-1).unwrap(),
        },
    ];
    let price_data = scrypto_encode(&prices).unwrap();
    let price_data_hash = keccak256_hash(&price_data).to_vec();
    let price_signature = Bls12381G1PrivateKey::from_u64(components.oracle_key_seed).unwrap().sign_v1(&price_data_hash);
    let price_updates = Some((price_data, price_signature));

    let current_time = ledger.get_current_time(TimePrecisionV2::Second);
    set_time(&mut ledger, current_time.add_seconds(1).unwrap());

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            components.exchange_component, 
            "process_request", 
            manifest_args!(
                margin_account_component, 
                index, 
                price_updates
            )
        )
        .deposit_batch(account)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    receipt.expect_commit_success();

    println!("{:?}", receipt.fee_summary);
    println!("{:?}", receipt.fee_details);


    // let manifest = ManifestBuilder::new()
    //     .lock_fee_from_faucet()
    //     .call_method(component, "free_token", manifest_args!())
    //     .call_method(
    //         account,
    //         "deposit_batch",
    //         manifest_args!(ManifestExpression::EntireWorktop),
    //     )
    //     .build();
    // let receipt = ledger.execute_manifest(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // receipt.expect_commit_success();
}
