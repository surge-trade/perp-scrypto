#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_cancel_requests_normal() {
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
    let price_limit_1 = PriceLimit::None;
    let slippage_limit_1 = SlippageLimit::None;
    let activate_requests_1 = vec![];
    let cancel_requests_1 = vec![];
    let status_1 = STATUS_ACTIVE;
    for _ in 0..10 {
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

    let result = interface.cancel_requests(
        margin_account_component,
        (0..10).collect(),
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 0);
    assert_eq!(account_details.requests_history.len(), 10);
    assert_eq!(account_details.requests_len, 10);

    for i in 0..10 {
        let request_details = account_details.requests_history[i].clone();
        assert_eq!(request_details.index, (9 - i) as ListIndex);
        assert_eq!(request_details.submission, time);
        assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
        assert_eq!(request_details.status, STATUS_CANCELLED);
    }

    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 10);
}

#[test]
fn test_cancel_requests_duplicate() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    interface.margin_order_request(
        0,
        expiry_seconds_1,
        margin_account_component,
        "BTC/USD".into(),
        dec!(100),
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let result = interface.cancel_requests(
        margin_account_component,
        vec![0, 0],
    ).expect_commit_success().clone();

    let time = interface.ledger_time();
    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 0);
    assert_eq!(account_details.requests_history.len(), 1);
    assert_eq!(account_details.requests_len, 1);

    let request_details = account_details.requests_history[0].clone();
    assert_eq!(request_details.index, 0 as ListIndex);
    assert_eq!(request_details.submission, time);
    assert_eq!(request_details.expiry, time.add_seconds(expiry_seconds_1 as i64).unwrap());
    assert_eq!(request_details.status, STATUS_CANCELLED);

    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

pub fn test_cancel_request_invalid_index() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.cancel_requests(
        margin_account_component,
        vec![0],
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_MISSING_REQUEST));
}

#[test]
pub fn test_cancel_request_not_active_or_dormant() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    interface.add_liquidity((base_resource, dec!(1000000))).expect_commit_success();
    interface.update_pair_configs(vec![
        PairConfig {
            pair_id: "BTC/USD".into(),
            oi_max: dec!(100000),
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

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let pair_id_1 = "BTC/USD";
    let amount_1 = dec!(100);
    interface.margin_order_request(
        0,
        expiry_seconds_1,
        margin_account_component,
        pair_id_1.into(),
        amount_1,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        vec![],
        vec![],
        STATUS_ACTIVE,
    ).expect_commit_success();

    let time = interface.increment_ledger_time(expiry_seconds_1 as i64);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_id_1.into(),
                quote: dec!(60000),
                timestamp: time,
            }
        ])
    ).expect_commit_success();

    interface.cancel_request(
        margin_account_component,
        0,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_CANCEL_REQUEST_NOT_ACTIVE_OR_DORMANT));
}

#[test]
fn test_cancel_request_invalid_auth() {
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

    let (badge_resource_2, _badge_id_2) = interface.mint_test_nft();
    let rule_2 = rule!(require(badge_resource_2));
    interface.set_level_3_auth(None, margin_account_component, rule_2).expect_commit_success();

    interface.cancel_requests(
        margin_account_component,
        vec![0],
    ).expect_auth_assertion_failure();
}
