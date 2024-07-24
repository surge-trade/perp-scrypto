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

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USDT";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
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
        STATUS_ACTIVE,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 1);
    assert_eq!(account_details.requests_history.len(), 1);
    assert_eq!(account_details.requests_len, 1);

    let request_details = account_details.active_requests[0].clone();
    assert_eq!(request_details.index, 0);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_ACTIVE);
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
    let pair_id_1 = "BTC/USDT";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![1, 2];
    let cancel_requests_1 = vec![];
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
        STATUS_ACTIVE,
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
    let pair_id_1 = "BTC/USDT";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![1, 2];
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
        STATUS_ACTIVE,
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
    let pair_id_1 = "BTC/USDT";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
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
            STATUS_ACTIVE,
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
        STATUS_ACTIVE,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ACTIVE_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_request_invalid_auth() {
    let mut interface = get_setup();

    let (badge_resource, _badge_id) = interface.mint_test_nft();

    let rule_0 = rule!(require(badge_resource));
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let delay_seconds_1 = 0;
    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USDT";
    let amount_1 = dec!(100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
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
        STATUS_ACTIVE,
    ).expect_auth_assertion_failure();
}
