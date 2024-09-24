#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_process_request_execute() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(100000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let time_2 = interface.ledger_time();
    let expiry_seconds_2 = 10;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.increment_ledger_time(1);
    let result = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_commit_success().clone();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 0);
    assert_eq!(account_details.requests_history.len(), 1);
    assert_eq!(account_details.requests_len, 1);

    let request_details = account_details.requests_history[0].clone();
    assert_eq!(request_details.index, 0);
    assert_eq!(request_details.submission, time_2);
    assert_eq!(request_details.expiry, time_2.add_seconds(expiry_seconds_2 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_EXECUTED);

    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_process_request_expired() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(100000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let time_2 = interface.ledger_time();
    let expiry_seconds_2 = 10;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.increment_ledger_time(expiry_seconds_2 as i64);
    let result = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_commit_success().clone();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 0);
    assert_eq!(account_details.requests_history.len(), 1);
    assert_eq!(account_details.requests_len, 1);

    let request_details = account_details.requests_history[0].clone();
    assert_eq!(request_details.index, 0);
    assert_eq!(request_details.submission, time_2);
    assert_eq!(request_details.expiry, time_2.add_seconds(expiry_seconds_2 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_EXPIRED);

    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_process_request_at_submission() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(10000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_2 = 10000000;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.ledger_time();
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_PROCESS_REQUEST_BEFORE_SUBMISSION));
}

#[test]
fn test_process_request_not_active() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(100000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_2 = 10;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.increment_ledger_time(exchange_config.max_price_age_seconds + 1 as i64);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_commit_success();

    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_PROCESS_REQUEST_NOT_ACTIVE));
}

#[test]
fn test_process_request_price_age_too_old() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(10000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_2 = 10000000;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.increment_ledger_time(exchange_config.max_price_age_seconds + 1 as i64);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3.add_seconds(-exchange_config.max_price_age_seconds as i64).unwrap(),
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_PRICE_TOO_OLD));
}

#[test]
fn test_process_request_no_fresh_price() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;

    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(100000),
            trade_size_min: dec!(0.000001),
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

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(1000);
    let result = interface.create_account(
        rule_1, 
        vec![(base_resource, base_input_1)], 
        None
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_2 = 10000000;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time_3 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_2.into(),
                quote: dec!(60000),
                timestamp: time_3,
            }
        ])
    ).expect_commit_success();

    let expiry_seconds_4 = 10000000;
    let pair_id_4 = "BTC/USD";
    let amount_4 = dec!(0.01);
    interface.margin_order_request(
        0,
        expiry_seconds_4,
        margin_account_component,
        pair_id_4.into(),
        amount_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let _time_5 = interface.increment_ledger_time(exchange_config.max_price_age_seconds + 1 as i64);
    interface.process_request(
        margin_account_component,
        1, 
        None
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_PRICE_TOO_OLD));
}

#[test]
fn test_process_request_before_valid_requests_start() {
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

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        trade_size_min: dec!(0.000001),
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

    let base_input_1 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_1)).expect_commit_success();

    let base_input_2 = dec!(75);
    let btc_input_2 = dec!(0.0006);
    let result_2 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_2),
            (btc_resource, btc_input_2),
        ], 
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result_2.new_component_addresses()[0];

    let trade_size_3 = dec!(0.02);
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

    let trade_size_4 = dec!(0.00000001);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(1000);
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success();

    let price_7 = dec!(60000);
    let time_7 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_7,
                timestamp: time_7,
            },
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_PROCESS_REQUEST_BEFORE_VALID_START));
}
