use scrypto_test::prelude::*;
use once_cell::sync::Lazy;

// use ::common::*;
use config::*;
// use account::*;
use exchange::*;
use oracle::*;

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

fn get_setup() -> ExchangeInterface {
    let snapshot = SETUP.0.clone();
    let public_key = SETUP.1.clone();
    let account = SETUP.2.clone();
    let resources = SETUP.3.clone();
    let components = SETUP.4.clone();
    
    let ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build_from_snapshot(snapshot);
    let exchange = ExchangeInterface::new(public_key, account, resources, components, ledger);

    exchange
}

#[test]
fn test_hello() {
    let mut exchange = get_setup();

    exchange.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(10000),
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
        }
    ]).expect_commit_success();

    let margin_account_component = exchange.create_account(
        rule!(allow_all), 
        dec!(1000), 
        None
    );

    exchange.margin_order_request(
        0,
        10000000000,
        margin_account_component,
        "BTC/USD".into(),
        dec!(0.0000000001),
        false,
        Limit::None,
        vec![],
        vec![],
        STATUS_ACTIVE
    ).expect_commit_success();

    let time = exchange.increment_ledger_time(1);
    exchange.process_request(
        margin_account_component,
        0, 
        vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time,
            },
        ]
    ).expect_commit_success();
}
