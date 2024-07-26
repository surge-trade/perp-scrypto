#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_margin_order_tp_sl_request_no_tp_no_sl() {
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
    let price_tp_1 = None;
    let price_sl_1 = None;
    let result = interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
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
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_margin_order_tp_sl_request_with_tp_no_sl() {
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
    let price_tp_1 = Some(dec!(70000));
    let price_sl_1 = None;
    let result = interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 2);
    assert_eq!(account_details.requests_history.len(), 2);
    assert_eq!(account_details.requests_len, 2);

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
        assert_eq!(margin_order_request.activate_requests, vec![1]);
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 1);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Gte(price_tp_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 2);
}

#[test]
fn test_margin_order_tp_sl_request_no_tp_with_sl() {
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
    let price_tp_1 = None;
    let price_sl_1 = Some(dec!(50000));
    let result = interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 2);
    assert_eq!(account_details.requests_history.len(), 2);
    assert_eq!(account_details.requests_len, 2);

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
        assert_eq!(margin_order_request.activate_requests, vec![1]);
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 1);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Lte(price_sl_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 2);
}

#[test]
fn test_margin_order_tp_sl_request_with_tp_with_sl_long() {
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
    let price_tp_1 = Some(dec!(70000));
    let price_sl_1 = Some(dec!(50000));
    let result = interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 3);
    assert_eq!(account_details.requests_history.len(), 3);
    assert_eq!(account_details.requests_len, 3);

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
        assert_eq!(margin_order_request.activate_requests, vec![1, 2]);
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 1);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Gte(price_tp_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, vec![2]);
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 2);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Lte(price_sl_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, vec![1]);
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 3);
}

#[test]
fn test_margin_order_tp_sl_request_with_tp_with_sl_short() {
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
    let amount_1 = dec!(-100);
    let reduce_only_1 = false;
    let price_limit_1 = Limit::None;
    let price_tp_1 = Some(dec!(50000));
    let price_sl_1 = Some(dec!(70000));
    let result = interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 3);
    assert_eq!(account_details.requests_history.len(), 3);
    assert_eq!(account_details.requests_len, 3);

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
        assert_eq!(margin_order_request.activate_requests, vec![1, 2]);
        assert_eq!(margin_order_request.cancel_requests, Vec::<ListIndex>::new());
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 1);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Lte(price_tp_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, vec![2]);
    } else {
        panic!("Request is not a MarginOrder request");
    }

    let request_details = account_details.active_requests[1].clone();
    assert_eq!(request_details.index, 2);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_DORMANT);
    if let Request::MarginOrder(margin_order_request) = request_details.request {
        assert_eq!(margin_order_request.pair_id, pair_id_1);
        assert_eq!(margin_order_request.amount, -amount_1);
        assert_eq!(margin_order_request.reduce_only, true);
        assert_eq!(margin_order_request.price_limit, Limit::Gte(price_sl_1.unwrap()));
        assert_eq!(margin_order_request.activate_requests, Vec::<ListIndex>::new());
        assert_eq!(margin_order_request.cancel_requests, vec![1]);
    } else {
        panic!("Request is not a MarginOrder request");
    }
 
    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 3);
}

#[test]
fn test_margin_order_tp_sl_request_exceed_max_active() {
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
    let price_tp_1 = Some(dec!(70000));
    let price_sl_1 = Some(dec!(50000));
    for _ in 0..exchange_config.active_requests_max / 3 {
        interface.margin_order_tp_sl_request(
            delay_seconds_1,
            expiry_seconds_1,
            margin_account_component,
            pair_id_1.into(),
            amount_1,
            reduce_only_1,
            price_limit_1,
            price_tp_1,
            price_sl_1,
        ).expect_commit_success();
    }

    interface.margin_order_tp_sl_request(
        delay_seconds_1,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        reduce_only_1,
        price_limit_1,
        price_tp_1,
        price_sl_1,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ACTIVE_REQUESTS_TOO_MANY));
}

#[test]
fn test_margin_order_tp_sl_request_invalid_auth() {
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
    let price_tp_2 = Some(dec!(70000));
    let price_sl_2 = Some(dec!(50000));
    interface.margin_order_tp_sl_request(
        delay_seconds_2,
        expiry_seconds_2,
        margin_account_component,
        pair_id_2.into(),
        amount_2,
        reduce_only_2,
        price_limit_2,
        price_tp_2,
        price_sl_2,
    ).expect_auth_assertion_failure();
}
