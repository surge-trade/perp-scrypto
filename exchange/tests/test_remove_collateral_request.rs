#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_remove_collateral_request_normal() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let claims_1 = vec![(base_resource, dec!(100))];
    let target_account_1 = interface.test_account;
    let result = interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
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
    if let Request::RemoveCollateral(remove_collateral_request) = request_details.request {
        assert_eq!(remove_collateral_request.target_account, target_account_1);
        assert_eq!(remove_collateral_request.claims, claims_1);
    } else {
        panic!("Request is not a RemoveCollateral request");
    }

    let event: EventRequests = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.requests.len(), 1);
}

#[test]
fn test_remove_collateral_request_exceed_max_claims() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let claims_1 = [(base_resource, dec!(100)); 11].to_vec();
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_CLAIMS_TOO_MANY));
}

#[test]
fn test_remove_collateral_request_exceed_max_active() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let exchange_config = interface.get_exchange_config();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let claims_1 = vec![(base_resource, dec!(100))];
    let target_account_1 = interface.test_account;
    for _ in 0..exchange_config.active_requests_max {
        interface.remove_collateral_request(
            expiry_seconds_1,
            margin_account_component,
            target_account_1,
            claims_1.clone(),
        ).expect_commit_success();
    }

    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ACTIVE_REQUESTS_TOO_MANY));
}

#[test]
fn test_remove_collateral_request_invalid_auth() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_2_auth(None, margin_account_component, rule_1).expect_commit_success();

    let expiry_seconds_2 = 10;
    let claims_2 = vec![(base_resource, dec!(100))];
    let target_account_2 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_2,
        margin_account_component,
        target_account_2,
        claims_2.clone(),
    ).expect_auth_assertion_failure();
}
