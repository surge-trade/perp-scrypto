#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_add_liquidity_initial() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let lp_resource = interface.resources.lp_resource;
    let exchange_config = interface.get_exchange_config();

    let base_input_0 = dec!(100000);
    let lp_balance_0 = interface.test_account_balance(lp_resource);
    let result = interface.add_liquidity((base_resource, base_input_0)).expect_commit_success().clone();

    let lp_balance_1 = interface.test_account_balance(lp_resource);
    let lp_output_1 = lp_balance_1 - lp_balance_0;
    let pool_value_1 = interface.get_pool_value();

    let fee = base_input_0 * exchange_config.fee_liquidity_add;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_pool = fee - fee_protocol - fee_treasury;

    assert_eq!(lp_output_1, base_input_0);
    assert_eq!(pool_value_1, base_input_0 - fee_protocol - fee_treasury);

    let event: EventLiquidityChange = interface.parse_event(&result);
    assert_eq!(event.lp_price, dec!(1));
    assert_eq!(event.lp_amount, lp_output_1);
    assert_eq!(event.amount, base_input_0);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
}

#[test]
fn test_add_liquidity_normal() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let lp_resource = interface.resources.lp_resource;
    let exchange_config = interface.get_exchange_config();

    let base_input_0 = dec!(100000);
    let base_balance_0 = interface.test_account_balance(lp_resource);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();
    
    let base_balance_1 = interface.test_account_balance(lp_resource);
    let lp_output_1 = base_balance_1 - base_balance_0;
    let pool_value_1 = interface.get_pool_value();
    let base_input_1 = dec!(50000);
    let result = interface.add_liquidity((base_resource, base_input_1)).expect_commit_success().clone();

    let lp_balance_2 = interface.test_account_balance(lp_resource);
    let lp_output_2 = lp_balance_2 - base_balance_1;
    let pool_value_2 = interface.get_pool_value();

    let fee = base_input_1 * exchange_config.fee_liquidity_add;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_pool = fee - fee_protocol - fee_treasury;
    let lp_price = pool_value_1 / lp_output_1;

    assert_eq!(lp_output_2, (base_input_1 - fee) / lp_price);
    assert_eq!(pool_value_2 - pool_value_1, base_input_1 - fee_protocol - fee_treasury);  

    let event: EventLiquidityChange = interface.parse_event(&result);
    assert_eq!(event.lp_price, lp_price);
    assert_eq!(event.lp_amount, lp_output_2);
    assert_eq!(event.amount, base_input_1);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
}

#[test]
fn test_add_liquidity_invalid_token() {
    let mut interface = get_setup();
    let fake_base_resource = interface.mint_test_token(dec!(1000000), 18);

    let fake_base_input_0 = dec!(100000);
    interface.add_liquidity((fake_base_resource, fake_base_input_0))
        .expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_PAYMENT_TOKEN));
}

// TODO: pool value below 1