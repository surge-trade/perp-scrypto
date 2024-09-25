#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_swap_debt_more_than_debt() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![(btc_resource, btc_input_2)], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(1);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_4 = dec!(10000);
    let btc_price_4 = dec!(100000);
    let time_4 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_4,
                timestamp: time_4,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_4,
                timestamp: time_4,
            },
        ])
    ).expect_commit_success();
    
    let price_5 = dec!(9000);
    let btc_price_5 = dec!(100000);
    let time_5 = interface.increment_ledger_time(10000);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let base_input_6 = dec!(1100);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let btc_balance_6 = interface.test_account_balance(btc_resource);
    let result_6 = interface.swap_debt(
        margin_account_component, 
        btc_resource, 
        (base_resource, base_input_6), 
        None,
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let btc_balance_7 = interface.test_account_balance(btc_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let btc_output_7 = btc_balance_7 - btc_balance_6;

    assert_eq!(base_output_7, base_input_6 + account_details_6.virtual_balance);
    assert_eq!(btc_output_7, -account_details_6.virtual_balance / btc_price_5);

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.virtual_balance, dec!(0));

    let event: EventSwapDebt = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.resource, btc_resource);
    assert_eq!(event.amount, btc_output_7);
    assert_eq!(event.price, btc_price_5);
}

#[test]
fn test_swap_debt_less_than_debt() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![(btc_resource, btc_input_2)], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(1);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_4 = dec!(10000);
    let btc_price_4 = dec!(100000);
    let time_4 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_4,
                timestamp: time_4,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_4,
                timestamp: time_4,
            },
        ])
    ).expect_commit_success();
    
    let price_5 = dec!(9000);
    let btc_price_5 = dec!(100000);
    let time_5 = interface.increment_ledger_time(10000);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let base_input_6 = dec!(900);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let btc_balance_6 = interface.test_account_balance(btc_resource);
    let result_6 = interface.swap_debt(
        margin_account_component, 
        btc_resource, 
        (base_resource, base_input_6), 
        None,
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let btc_balance_7 = interface.test_account_balance(btc_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let btc_output_7 = btc_balance_7 - btc_balance_6;

    assert_eq!(base_output_7, dec!(0));
    assert_eq!(btc_output_7, base_input_6 / btc_price_5);

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.virtual_balance, base_input_6 + account_details_6.virtual_balance);

    let event: EventSwapDebt = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.resource, btc_resource);
    assert_eq!(event.amount, btc_output_7);
    assert_eq!(event.price, btc_price_5);
}

#[test]
fn test_swap_debt_more_than_token() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let xrd_resource = interface.mint_test_token(dec!(100), 18);

    let btc_collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    let xrd_collateral_config = CollateralConfig {
        pair_id: "XRD/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, btc_collateral_config.clone()),
        (xrd_resource, xrd_collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let xrd_input_2 = dec!(100);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![
            (btc_resource, btc_input_2),
            (xrd_resource, xrd_input_2),
        ], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(1);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_4 = dec!(10000);
    let btc_price_4 = dec!(100000);
    let xrd_price_4 = dec!(0.05);
    let time_4 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_4,
                timestamp: time_4,
            },
            Price {
                pair: btc_collateral_config.pair_id.clone(),
                quote: btc_price_4,
                timestamp: time_4,
            },
            Price {
                pair: xrd_collateral_config.pair_id.clone(),
                quote: xrd_price_4,
                timestamp: time_4,
            },
        ])
    ).expect_commit_success();
    
    let price_5 = dec!(9000);
    let btc_price_5 = dec!(100000);
    let xrd_price_5 = dec!(0.05);
    let time_5 = interface.increment_ledger_time(10000);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
            Price {
                pair: btc_collateral_config.pair_id.clone(),
                quote: btc_price_5,
                timestamp: time_5,
            },
            Price {
                pair: xrd_collateral_config.pair_id.clone(),
                quote: xrd_price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let base_input_6 = dec!(1000);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let xrd_balance_6 = interface.test_account_balance(xrd_resource);
    let result_6 = interface.swap_debt(
        margin_account_component, 
        xrd_resource, 
        (base_resource, base_input_6), 
        None,
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let xrd_balance_7 = interface.test_account_balance(xrd_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let xrd_output_7 = xrd_balance_7 - xrd_balance_6;

    assert_eq!(base_output_7, base_input_6 - (xrd_input_2 * xrd_price_5));
    assert_eq!(xrd_output_7, xrd_input_2);

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.virtual_balance, account_details_6.virtual_balance + (xrd_input_2 * xrd_price_5));
    
    let event: EventSwapDebt = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.resource, xrd_resource);
    assert_eq!(event.amount, xrd_output_7);
    assert_eq!(event.price, xrd_price_5);
}

#[test]
fn test_swap_debt_no_debt() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![(btc_resource, btc_input_2)], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let base_input_3 = dec!(1000);
    let price_3 = dec!(10000);
    let btc_price_3 = dec!(100000);
    let time_3 = interface.ledger_time();
    interface.swap_debt(
        margin_account_component, 
        btc_resource, 
        (base_resource, base_input_3), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_3,
                timestamp: time_3,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_3,
                timestamp: time_3,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_SWAP_NO_DEBT));
}

#[test]
fn test_swap_debt_invalid_token() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let xrd_resource = interface.mint_test_token(dec!(100), 18);

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![(btc_resource, btc_input_2)], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(1);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_4 = dec!(10000);
    let btc_price_4 = dec!(100000);
    let time_4 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_4,
                timestamp: time_4,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_4,
                timestamp: time_4,
            },
        ])
    ).expect_commit_success();
    
    let price_5 = dec!(9000);
    let btc_price_5 = dec!(100000);
    let time_5 = interface.increment_ledger_time(10000);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(1000);
    interface.swap_debt(
        margin_account_component, 
        xrd_resource, 
        (base_resource, base_input_6), 
        None,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_COLLATERAL));
}

#[test]
fn test_swap_debt_invalid_payment() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();
    
    let pair_config = pair_config_zero_fees_and_funding("TEST/USD".into());
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let btc_input_2 = dec!(1);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![(btc_resource, btc_input_2)], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(1);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_3,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_4 = dec!(10000);
    let btc_price_4 = dec!(100000);
    let time_4 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_4,
                timestamp: time_4,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_4,
                timestamp: time_4,
            },
        ])
    ).expect_commit_success();
    
    let price_5 = dec!(9000);
    let btc_price_5 = dec!(100000);
    let time_5 = interface.increment_ledger_time(10000);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
            Price {
                pair: collateral_config.pair_id.clone(),
                quote: btc_price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let btc_input_6 = dec!(1);
    interface.swap_debt(
        margin_account_component, 
        btc_resource, 
        (btc_resource, btc_input_6), 
        None,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_PAYMENT_TOKEN));
}
