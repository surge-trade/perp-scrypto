#![allow(dead_code)]

mod tests_common;
use tests_common::*;

#[test]
fn test_set_level_1_auth() {
    let mut interface = get_setup();

    let (badge_resource0, badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, _badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));
    interface.set_level_1_auth(
        (badge_resource0, badge_id0),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), rule_1);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), rule_0);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), rule_0);

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!());
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1);
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!());
}

#[test]
fn test_set_level_2_auth() {
    let mut interface = get_setup();

    let (badge_resource0, badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, _badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));
    interface.set_level_2_auth(
        (badge_resource0, badge_id0),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), rule_0);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), rule_1);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), rule_0);

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!());
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1);
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!());
}

#[test]
fn test_set_level_3_auth() {
    let mut interface = get_setup();

    let (badge_resource0, badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, _badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));
    interface.set_level_3_auth(
        (badge_resource0, badge_id0),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), rule_0);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), rule_0);
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), rule_1);

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!());

    let permissions_b = interface.get_permissions(rule_1);
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));
}

fn test_set_level_1_auth_invalid_auth() {
    let mut interface = get_setup();

    let (badge_resource0, _badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));
    interface.set_level_1_auth(
        (badge_resource1, badge_id1),
        margin_account_component,
        rule_1.clone(),
    ).expect_auth_assertion_failure();
}

fn test_set_level_2_auth_invalid_auth() {
    let mut interface = get_setup();

    let (badge_resource0, _badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));
    interface.set_level_2_auth(
        (badge_resource1, badge_id1),
        margin_account_component,
        rule_1.clone(),
    ).expect_auth_assertion_failure();
}

fn test_set_level_3_auth_invalid_auth() {
    let mut interface = get_setup();

    let (badge_resource0, _badge_id0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource1, badge_id1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource1));

    interface.set_level_3_auth(
        (badge_resource1, badge_id1),
        margin_account_component,
        rule_1.clone(),
    ).expect_auth_assertion_failure();
}

// TODO: check permissions fee growth