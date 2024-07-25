#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_hello() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    
    interface.update_pair_configs(vec![
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

    let base_input_0 = dec!(100000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let trade_size_2 = dec!(0.0001);
    interface.margin_order_request(
        0,
        10000000000,
        margin_account_component,
        "BTC/USD".into(),
        trade_size_2,
        false,
        Limit::None,
        vec![],
        vec![],
        STATUS_ACTIVE
    ).expect_commit_success();

    let btc_price_3 = dec!(60000);
    let time = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: btc_price_3,
                timestamp: time,
            },
        ])
    ).expect_commit_success();
}
