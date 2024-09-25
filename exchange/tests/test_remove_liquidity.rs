#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_remove_liquidity_normal() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let lp_resource = interface.resources.lp_resource;
    let exchange_config = interface.get_exchange_config();

    let base_input_0 = dec!(100000);
    let lp_balance_0 = interface.test_account_balance(lp_resource);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();
    
    let lp_balance_1 = interface.test_account_balance(lp_resource);
    let lp_output_1 = lp_balance_1 - lp_balance_0;
    let base_balance_1 = interface.test_account_balance(base_resource);
    let pool_value_1 = interface.get_pool_value();
    let lp_input_1 = lp_output_1 / dec!(2);
    let result = interface.remove_liquidity((lp_resource, lp_input_1)).expect_commit_success().clone();

    let base_balance_2 = interface.test_account_balance(base_resource);
    let base_output_2 = base_balance_2 - base_balance_1;
    let pool_value_2 = interface.get_pool_value();

    let lp_price = pool_value_1 / lp_output_1;
    let value = lp_input_1 * lp_price;
    let fee = value * exchange_config.fee_liquidity_remove;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_pool = fee - fee_protocol - fee_treasury;

    assert_eq!(base_output_2, value - fee);
    assert_eq!(pool_value_2 - pool_value_1, -value + fee_pool);

    let event: EventLiquidityChange = interface.parse_event(&result);
    assert_eq!(event.lp_price, lp_price);
    assert_eq!(event.lp_amount, -lp_input_1);
    assert_eq!(event.amount, -base_output_2);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
}

#[test]
fn test_remove_liquidity_invalid_token() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let fake_base_resource = interface.mint_test_token(dec!(1000000), 18);

    let base_input_0 = dec!(100000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let fake_base_input_1 = dec!(10000);
    interface.remove_liquidity((fake_base_resource, fake_base_input_1))
        .expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_LP_TOKEN));
}

#[test]
fn test_remove_liquidity_exceed_skew_cap() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let lp_resource = interface.resources.lp_resource;
    let exchange_config = interface.get_exchange_config();

    let pair_config = default_pair_config("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config]).expect_commit_success();

    let result = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(1000))], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let base_input_0 = dec!(100000);
    let lp_balance_0 = interface.test_account_balance(lp_resource);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();
    
    let lp_balance_1 = interface.test_account_balance(lp_resource);
    let lp_output_1 = lp_balance_1 - lp_balance_0;
    let amount_1 = (base_input_0 * dec!(0.95)) * exchange_config.skew_ratio_cap;
    interface.margin_order_request(
        0,
        10000000000,
        margin_account_component,
        "TEST/USD".into(),
        amount_1,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE
    ).expect_commit_success();

    let time = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "TEST/USD".into(),
                quote: dec!(1),
                timestamp: time,
            },
        ])
    ).expect_commit_success();

    let lp_input_1 = lp_output_1 / dec!(2);
    interface.remove_liquidity((lp_resource, lp_input_1))
        .expect_specific_failure(|err| check_error_msg(err, ERROR_SKEW_TOO_HIGH));
}

// TODO: pool value below 0