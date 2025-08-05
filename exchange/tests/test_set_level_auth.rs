#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_create_recovery_key() {
    let mut interface = get_setup();
    let recovery_key_resource = interface.resources.recovery_key_resource;
    
    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.create_recovery_key(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
    ).expect_commit_success();
    let recovery_key_id = interface.test_account_nft_ids(recovery_key_resource)[0].clone();
    let rule_1 = rule!(require(recovery_key_id.clone()));
    let rule_total = rule!(require(badge_resource_0) || require(recovery_key_id.clone()));

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1);
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!());

    let permissions_total = interface.get_permissions(rule_total);
    assert_eq!(permissions_total.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_total.level_2, indexset!());
    assert_eq!(permissions_total.level_3, indexset!());
}

#[test]
fn test_add_auth_rule_level_1() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = require(badge_resource_1);
    interface.add_auth_rule(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        1,
        rule_1.clone(),
    ).expect_commit_success();

    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.into());
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!());
}

#[test]
fn test_add_auth_rule_level_2() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = require(badge_resource_1);
    interface.add_auth_rule(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        2,
        rule_1.clone(),
    ).expect_commit_success();

    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.into());
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!());
}

#[test]
fn test_add_auth_rule_level_3() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = require(badge_resource_1);
    interface.add_auth_rule(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        3,
        rule_1.clone(),
    ).expect_commit_success();

    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.into());
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));
}

#[test]
fn test_remove_auth_rule_level_1() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let (badge_resource_2, _badge_id_2) = interface.mint_test_nft();
    let rule_0 = require(badge_resource_0);
    let rule_1 = require(badge_resource_1);
    let rule_2 = require(badge_resource_2);

    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1) || require(badge_resource_2));
    let result = interface.create_account(
        rule_total.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        1,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule!(require(badge_resource_0) || require(badge_resource_2))));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));

    let permissions_c = interface.get_permissions(rule_2.clone().into());
    assert_eq!(permissions_c.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_c.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_c.level_3, indexset!(margin_account_component));

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        1,
        rule_2.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone().into()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));

    let permissions_c = interface.get_permissions(rule_2.clone().into());
    assert_eq!(permissions_c.level_1, indexset!());
    assert_eq!(permissions_c.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_c.level_3, indexset!(margin_account_component));

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        1,
        rule_0.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule!(deny_all)));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!());
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));

    let permissions_c = interface.get_permissions(rule_2.clone().into());
    assert_eq!(permissions_c.level_1, indexset!());
    assert_eq!(permissions_c.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_c.level_3, indexset!(margin_account_component));
}

#[test]
fn test_remove_auth_rule_level_2() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_0 = require(badge_resource_0);
    let rule_1 = require(badge_resource_1);
    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1));
    let result = interface.create_account(
        rule_total.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        2,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule!(require(badge_resource_0))));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        2,
        rule_0.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule!(deny_all)));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_total.clone()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!());
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));
}

#[test]
fn test_remove_auth_rule_level_3() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_0 = require(badge_resource_0);
    let rule_1 = require(badge_resource_1);
    let rule_total = rule!(require(badge_resource_0) || require(badge_resource_1));
    let result = interface.create_account(
        rule_total.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        3,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone().into()));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!(margin_account_component));

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!());

    interface.remove_auth_rule(
        Some((badge_resource_0, badge_id_0.clone())),
        margin_account_component,
        3,
        rule_0.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_total.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule!(deny_all)));

    let permissions_a = interface.get_permissions(rule_0.clone().into());
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!());

    let permissions_b = interface.get_permissions(rule_1.clone().into());
    assert_eq!(permissions_b.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_b.level_3, indexset!());
}

#[test]
fn test_set_level_1_auth() {
    let mut interface = get_setup();

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_1_auth(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_1.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

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

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_2_auth(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_1.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_0.clone()));

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

    let (badge_resource_0, badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_3_auth(
        Some((badge_resource_0, badge_id_0)),
        margin_account_component,
        rule_1.clone(),
    ).expect_commit_success();

    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_1"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_2"), Some(rule_0.clone()));
    assert_eq!(interface.get_role(margin_account_component, ModuleId::Main, "level_3"), Some(rule_1.clone()));

    let permissions_a = interface.get_permissions(rule_0);
    assert_eq!(permissions_a.level_1, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_2, indexset!(margin_account_component));
    assert_eq!(permissions_a.level_3, indexset!());

    let permissions_b = interface.get_permissions(rule_1);
    assert_eq!(permissions_b.level_1, indexset!());
    assert_eq!(permissions_b.level_2, indexset!());
    assert_eq!(permissions_b.level_3, indexset!(margin_account_component));
}

#[test]
fn test_create_recovery_key_invalid_auth() {
    let mut interface = get_setup();
    
    let (badge_resource_0, _badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    interface.create_recovery_key(
        None,
        margin_account_component,
    ).expect_auth_assertion_failure();
}

#[test]
fn test_add_auth_rule_invalid_auth() {
    let mut interface = get_setup();

    let (badge_resource_0, _badge_id_0) = interface.mint_test_nft();
    let rule_0 = rule!(require(badge_resource_0));
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = require(badge_resource_1);
    interface.add_auth_rule(
        None,
        margin_account_component,
        1,
        rule_1.clone(),
    ).expect_auth_assertion_failure();
}

#[test]
fn test_set_level_1_auth_invalid_auth() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_1_auth(None, margin_account_component, rule_1).expect_commit_success();

    let (badge_resource_2, _badge_id_2) = interface.mint_test_nft();
    let rule_2 = rule!(require(badge_resource_2));
    interface.set_level_1_auth(
        None,
        margin_account_component,
        rule_2,
    ).expect_auth_assertion_failure();
}

#[test]
fn test_set_level_2_auth_invalid_auth() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_1_auth(None, margin_account_component, rule_1).expect_commit_success();

    let (badge_resource_2, _badge_id_2) = interface.mint_test_nft();
    let rule_2 = rule!(require(badge_resource_2));
    interface.set_level_2_auth(
        None,
        margin_account_component,
        rule_2,
    ).expect_auth_assertion_failure();
}

#[test]
fn test_set_level_3_auth_invalid_auth() {
    let mut interface = get_setup();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let (badge_resource_1, _badge_id_1) = interface.mint_test_nft();
    let rule_1 = rule!(require(badge_resource_1));
    interface.set_level_1_auth(None, margin_account_component, rule_1).expect_commit_success();

    let (badge_resource_2, _badge_id_2) = interface.mint_test_nft();
    let rule_2 = rule!(require(badge_resource_2));
    interface.set_level_1_auth(
        None,
        margin_account_component,
        rule_2,
    ).expect_auth_assertion_failure();
}

// TODO: check permissions fee growth