#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_swap_protocol_fee_normal() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let protocol_resource = interface.resources.protocol_resource;

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(2),
        trade_size_min: dec!(0),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_0 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let amount_long_1 = dec!(1);
    let amount_short_1 = dec!(0.7);
    let price_1 = dec!(60000);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_1, amount_short_1, price_1);

    let protocol_input_2 = exchange_config.protocol_burn_amount + dec!(1);
    let base_balance_2 = interface.test_account_balance(base_resource);
    let protocol_balance_2 = interface.test_account_balance(protocol_resource);
    let protocol_fees_2 = interface.get_protocol_balance();
    let pool_details_2 = interface.get_pool_details();
    interface.swap_protocol_fee(
        (protocol_resource, protocol_input_2)
    ).expect_commit_success();

    let base_balance_3 = interface.test_account_balance(base_resource);
    let protocol_balance_3 = interface.test_account_balance(protocol_resource);
    let protocol_fees_3 = interface.get_protocol_balance();

    assert_eq!(base_balance_3, base_balance_2 + protocol_fees_2);
    assert_eq!(protocol_balance_3, protocol_balance_2 - exchange_config.protocol_burn_amount);
    assert_eq!(protocol_fees_3, dec!(0));

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_2.base_tokens_amount - protocol_fees_2);
    assert_eq!(pool_details.virtual_balance, pool_details_2.virtual_balance + protocol_fees_2);
}

#[test]
fn test_swap_protocol_fee_invalid_payment() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let fake_protocol_resource = interface.mint_test_token(dec!(10000000), 18);

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        trade_size_min: dec!(0),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_0 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let amount_long_1 = dec!(1);
    let amount_short_1 = dec!(0.7);
    let price_1 = dec!(60000);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_1, amount_short_1, price_1);

    let protocol_input_2 = exchange_config.protocol_burn_amount + dec!(1);
    interface.swap_protocol_fee(
        (fake_protocol_resource, protocol_input_2)
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_PROTOCOL_TOKEN))
}

#[test]
fn test_swap_protocol_fee_insufficient_payment() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let protocol_resource = interface.resources.protocol_resource;

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        trade_size_min: dec!(0),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_0 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let amount_long_1 = dec!(1);
    let amount_short_1 = dec!(0.7);
    let price_1 = dec!(60000);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_1, amount_short_1, price_1);

    let protocol_input_2 = exchange_config.protocol_burn_amount - dec!(1);
    interface.swap_protocol_fee(
        (protocol_resource, protocol_input_2)
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INSUFFICIENT_PAYMENT))
}

// TODO: insufficient pool tokens
