#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_remove_collateral_base() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let base_claim_1 = dec!(100);
    let claims_1 = vec![(base_resource, base_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    let time_2 = interface.increment_ledger_time(1);
    let base_balance_2 = interface.test_account_balance(base_resource);
    let result = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_commit_success().clone();

    let base_balance_3 = interface.test_account_balance(base_resource);
    let base_output_3 = base_balance_3 - base_balance_2;
    assert_eq!(base_output_3, base_claim_1);

    let event: EventRemoveCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.target_account, target_account_1);
    assert_eq!(event.amounts, claims_1);
}

#[test]
fn test_remove_collateral_other() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let btc_claim_1 = dec!(1);
    let claims_1 = vec![(btc_resource, btc_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    let time_2 = interface.increment_ledger_time(1);
    let btc_balance_2 = interface.test_account_balance(btc_resource);
    let result = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_commit_success().clone();

    let btc_balance_3 = interface.test_account_balance(btc_resource);
    let btc_output_3 = btc_balance_3 - btc_balance_2;
    assert_eq!(btc_output_3, btc_claim_1);

    // TODO: check account collateral amounts

    let event: EventRemoveCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.target_account, target_account_1);
    assert_eq!(event.amounts, claims_1);
}

#[test]
fn test_remove_collateral_base_too_much() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let base_claim_1 = dec!(101);
    let claims_1 = vec![(base_resource, base_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    let time_2 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_WITHDRAWAL_INSUFFICIENT_BALANCE));
}

#[test]
fn test_remove_collateral_other_too_much() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let btc_claim_1 = dec!(2);
    let claims_1 = vec![(btc_resource, btc_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    let time_2 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_WITHDRAWAL_INSUFFICIENT_BALANCE));
}


#[test]
fn test_remove_collateral_protected_target_account() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let base_claim_1 = dec!(100);
    let claims_1 = vec![(base_resource, base_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    interface.test_account_restrict_deposits();

    let time_2 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_specific_failure(|err| {
        match err {
            RuntimeError::ApplicationError(ApplicationError::AccountError(radix_engine::blueprints::account::AccountError::NotAnAuthorizedDepositor { depositor: _ })) => true,
            _ => false,
        }
    });
}

#[test]
fn test_remove_collateral_protected_target_account_with_authorized_depositor() {
    let mut interface = get_setup();
    let authority_resource = interface.resources.authority_resource;
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![(base_resource, dec!(100)), (btc_resource, dec!(1))],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let expiry_seconds_1 = 10;
    let base_claim_1 = dec!(100);
    let claims_1 = vec![(base_resource, base_claim_1)];
    let target_account_1 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_1,
        margin_account_component,
        target_account_1,
        claims_1.clone(),
    ).expect_commit_success();

    interface.test_account_restrict_deposits();
    interface.test_account_add_authorized_depositor(authority_resource);

    let time_2 = interface.increment_ledger_time(1);
    let base_balance_2 = interface.test_account_balance(base_resource);
    let result = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_2,
            }
        ])
    ).expect_commit_success().clone();

    let base_balance_3 = interface.test_account_balance(base_resource);
    let base_output_3 = base_balance_3 - base_balance_2;
    assert_eq!(base_output_3, base_claim_1);

    let event: EventRemoveCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.target_account, target_account_1);
    assert_eq!(event.amounts, claims_1);
}

#[test]
fn test_remove_collateral_insufficient_margin() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);

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
    interface.update_collateral_configs(vec![
        (btc_resource, CollateralConfig {
            pair_id: "BTC/USD".to_string(),
            discount: dec!(0.90),
            margin: dec!(0.01),
        }),
    ]).expect_commit_success();

    let base_input_0 = dec!(100000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let rule_1 = rule!(allow_all);
    let base_input_1 = dec!(100);
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

    let expiry_seconds_4 = 10;
    let base_claim_4 = dec!(100);
    let claims_4 = vec![(base_resource, base_claim_4)];
    let target_account_4 = interface.test_account;
    interface.remove_collateral_request(
        expiry_seconds_4,
        margin_account_component,
        target_account_4,
        claims_4.clone(),
    ).expect_commit_success();

    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: "BTC/USD".into(),
                quote: dec!(60000),
                timestamp: time_5,
            }
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INSUFFICIENT_MARGIN));
}
