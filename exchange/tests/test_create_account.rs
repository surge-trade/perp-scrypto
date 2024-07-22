#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_create_account_without_referral() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

    let permissions = interface.get_permissions(rule_0);
    assert_eq!(permissions.level_1, indexset!(margin_account_component));
    assert_eq!(permissions.level_2, indexset!(margin_account_component));
    assert_eq!(permissions.level_3, indexset!(margin_account_component));

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.virtual_balance, dec!(0));
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.collaterals.len(), 0);
    assert_eq!(account_details.valid_requests_start, 0);
    assert_eq!(account_details.active_requests.len(), 0);
    assert_eq!(account_details.requests_history.len(), 0);
    assert_eq!(account_details.requests_len, 0);
    assert!(account_details.referral.is_none());

    let event: EventAccountCreation = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.referral_id, None);
}

#[test]
fn test_create_account_with_referral() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();
    
    let referral_code = "test".to_string();
    let referral_hash = keccak256_hash(referral_code.clone().into_bytes());

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let referral_hashes = hashmap!(
        referral_hash => (vec![(base_resource, base_referral_0)], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id.clone()), vec![(base_resource, base_input_0)], referral_hashes).expect_commit_success();

    let rule_1 = rule!(allow_all);
    let result = interface.create_account(
        rule_1,
        vec![],
        Some(referral_code.clone()),
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.virtual_balance, base_referral_0);
    assert!(account_details.referral.is_some());
    assert_eq!(*account_details.referral.unwrap().0.local_id(), referral_id);

    let event: EventAccountCreation = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.referral_id, Some(referral_id));
}

#[test]
fn test_create_account_with_tokens() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let rule_0 = rule!(allow_all);
    let base_input_0 = dec!(10);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, base_input_0)],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.virtual_balance, base_input_0);
    assert!(account_details.referral.is_none());

    let event: EventAccountCreation = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.referral_id, None);

    let event: EventAddCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.amounts, vec![(base_resource, base_input_0)]);
}

// TODO: with address reservation

// TODO: test create account with fee oath
