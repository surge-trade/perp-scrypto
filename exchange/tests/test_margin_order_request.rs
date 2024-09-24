#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_margin_order_request_constant_index() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 10;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![RequestIndexRef::Index(1)];
    let cancel_requests_1 = vec![RequestIndexRef::Index(2)];
    let status_1 = STATUS_ACTIVE;
    let result = interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let expected_submission = time.add_seconds(delay_seconds_1 as i64).unwrap();
    let expected_expiry = expected_submission.add_seconds(expiry_seconds_1 as i64).unwrap();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 1);
    assert_eq!(account_details.requests_history.len(), 1);
    assert_eq!(account_details.requests_len, 1);

    let request_details = account_details.active_requests[0].clone();
    assert_eq!(request_details.index, 0);
    assert_eq!(request_details.submission, expected_submission);
    assert_eq!(request_details.expiry, expected_expiry);
    assert_eq!(request_details.status, status_1);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, amount_1);
        assert_eq!(margin_order_request.reduce_only, reduce_only_1);
        assert_eq!(margin_order_request.price_limit, price_limit_1);
        assert_eq!(margin_order_request.slippage_limit, slippage_limit_1);
        assert_eq!(margin_order_request.activate_requests, vec![1]);
        assert_eq!(margin_order_request.cancel_requests, vec![2]);
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_margin_order_request_relative_index() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 10;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_ACTIVE;
    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_commit_success();

    let delay_seconds_2 = 10;
    let expiry_seconds_2 = 10;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(100);
    let reduce_only_2 = false;
    let price_limit_2 = PriceLimit::None;
    let slippage_limit_2 = SlippageLimit::None;
    let activate_requests_2 = vec![RequestIndexRef::RelativeIndex(1)];
    let cancel_requests_2 = vec![RequestIndexRef::RelativeIndex(-1)];
    let status_2 = STATUS_ACTIVE;
    let result = interface.margin_order_request(
        delay_seconds_2,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        reduce_only_2,
        price_limit_2,
        slippage_limit_2,
        activate_requests_2.clone(),
        cancel_requests_2.clone(),
        status_2,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let expected_submission = time.add_seconds(delay_seconds_2 as i64).unwrap();
    let expected_expiry = expected_submission.add_seconds(expiry_seconds_2 as i64).unwrap();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 2);
    assert_eq!(account_details.requests_history.len(), 2);
    assert_eq!(account_details.requests_len, 2);

    let request_details = account_details.requests_history[0].clone();
    assert_eq!(request_details.index, 1);
    assert_eq!(request_details.submission, expected_submission);
    assert_eq!(request_details.expiry, expected_expiry);
    assert_eq!(request_details.status, status_2);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_2);
        assert_eq!(margin_order_request.amount, amount_2);
        assert_eq!(margin_order_request.reduce_only, reduce_only_2);
        assert_eq!(margin_order_request.price_limit, price_limit_2);
        assert_eq!(margin_order_request.slippage_limit, slippage_limit_2);
        assert_eq!(margin_order_request.activate_requests, vec![2]);
        assert_eq!(margin_order_request.cancel_requests, vec![0]);
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_margin_order_long_request_min_trade_size_not_met() {
    let mut interface = get_setup();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(2),
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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.margin_order_request(
        0,
        10,
        margin_account_component,
        pair_config.pair_id.clone(),
        dec!(0.0000009),
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_TRADE_SIZE_MIN_NOT_MET));
}

#[test]
fn test_margin_order_short_request_min_trade_size_not_met() {
    let mut interface = get_setup();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(2),
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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.margin_order_request(
        0,
        10,
        margin_account_component,
        pair_config.pair_id.clone(),
        dec!(-0.0000009),
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_TRADE_SIZE_MIN_NOT_MET));
}

#[test]
fn test_margin_order_request_exceed_max_activate_requests() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![RequestIndexRef::Index(1), RequestIndexRef::Index(2), RequestIndexRef::Index(3)];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_ACTIVE;
    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_EFFECTED_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_exceed_max_cancel_requests() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![RequestIndexRef::Index(1), RequestIndexRef::Index(2), RequestIndexRef::Index(3)];
    let status_1 = STATUS_ACTIVE;
    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_EFFECTED_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_exceed_max_active() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_ACTIVE;
    for _ in 0..exchange_config.active_requests_max {
        interface.margin_order_request(
            delay_seconds_1,
            expiry_seconds_1,
            margin_account_component,
            pair_id_1.into(),
            amount_1,
            reduce_only_1,
            price_limit_1,
            slippage_limit_1,
            activate_requests_1.clone(),
            cancel_requests_1.clone(),
            status_1,
        ).expect_commit_success();
    }

    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ACTIVE_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_invalid_status() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_EXECUTED;
    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        slippage_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_REQUEST_STATUS));
}

#[test]
fn test_margin_order_request_invalid_auth() {
    let mut interface = get_setup();

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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_3_auth(None, margin_account_component, rule_1).expect_commit_success();

    let delay_seconds_2 = 0;
    let expiry_seconds_2 = 10;
    let pair_id_2 = "BTC/USD";
    let amount_2 = dec!(100);
    let reduce_only_2 = false;
    let price_limit_2 = PriceLimit::None;
    let slippage_limit_2 = SlippageLimit::None;
    let activate_requests_2 = vec![];
    let cancel_requests_2 = vec![];
    let status_2 = STATUS_ACTIVE;
    interface.margin_order_request(
        delay_seconds_2,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        reduce_only_2,
        price_limit_2,
        slippage_limit_2,
        activate_requests_2.clone(),
        cancel_requests_2.clone(),
        status_2,
    ).expect_auth_assertion_failure();
}
