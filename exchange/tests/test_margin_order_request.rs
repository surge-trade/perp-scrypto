#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_margin_order_request_normal() {
    let mut interface = get_setup();

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
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_ACTIVE;
    let result = interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
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
        assert_eq!(margin_order_request.activate_requests, activate_requests_1);
        assert_eq!(margin_order_request.cancel_requests, cancel_requests_1);
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_margin_order_request_exceed_max_activate_requests() {
    let mut interface = get_setup();

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
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![1, 2, 3];
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
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_EFFECTED_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_exceed_max_cancel_requests() {
    let mut interface = get_setup();

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
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![1, 2, 3];
    let status_1 = STATUS_ACTIVE;
    interface.margin_order_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_EFFECTED_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_exceed_max_active() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();

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
    let price_limit_1 = Limit::None;
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
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ACTIVE_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_invalid_status() {
    let mut interface = get_setup();

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
    let price_limit_1 = Limit::None;
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
        activate_requests_1.clone(),
        cancel_requests_1.clone(),
        status_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_REQUEST_STATUS));
}

#[test]
fn test_margin_order_request_invalid_auth() {
    let mut interface = get_setup();

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
    let price_limit_2 = Limit::None;
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
        activate_requests_2.clone(),
        cancel_requests_2.clone(),
        status_2,
    ).expect_auth_assertion_failure();
}
