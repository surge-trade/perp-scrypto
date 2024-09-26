#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_add_collateral_base() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0,
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let base_input_1 = dec!(1000);
    let result = interface.add_collateral(
        margin_account_component,
        vec![(base_resource, base_input_1)],
    ).expect_commit_success().clone();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.virtual_balance, base_input_1);
    assert_eq!(account_details.collaterals.len(), 0);

    let event: EventAddCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.amounts, vec![(base_resource, base_input_1)]);
}

#[test]
fn test_add_collateral_other_asset() {
    let mut interface = get_setup();

    let btc_resource = interface.mint_test_token(dec!(100), DIVISIBILITY_MAXIMUM);

    interface.update_collateral_configs(vec![(
        btc_resource,
        CollateralConfig {
            pair_id: "BTC/USD".into(),
            price_age_max: 5,
            discount: dec!(0.95),
            margin: dec!(0.01),
        }
    )]).expect_commit_success();
    
    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let btc_input_1 = dec!(1);
    let result = interface.add_collateral(
        margin_account_component,
        vec![(btc_resource, btc_input_1)],
    ).expect_commit_success().clone();

    let account_details = interface.get_account_details(margin_account_component, 10, None);
    assert_eq!(account_details.virtual_balance, dec!(0));
    assert_eq!(account_details.collaterals.len(), 1);
    assert_eq!(account_details.collaterals[0].resource, btc_resource);
    assert_eq!(account_details.collaterals[0].amount, btc_input_1);

    let event: EventAddCollateral = interface.parse_event(&result);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.amounts, vec![(btc_resource, btc_input_1)]);
}

#[test]
fn test_add_collateral_exceed_collaterals_max() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();

    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let mut configs_1 = vec![];
    let mut tokens_1 = vec![];
    for _ in 0..exchange_config.collaterals_max {
        let token_input_1 = dec!(100);
        let token_resource_1 = interface.mint_test_token(token_input_1, DIVISIBILITY_MAXIMUM);
        tokens_1.push((token_resource_1, token_input_1));
        configs_1.push((
            token_resource_1,
            CollateralConfig {
                pair_id: "TEST/USD".into(),
                price_age_max: 5,
                discount: dec!(0.95),
                margin: dec!(0.01),
            }
        )); 
    }
    interface.update_collateral_configs(configs_1).expect_commit_success();
    interface.add_collateral(margin_account_component, tokens_1).expect_commit_success();

    let token_input_2 = dec!(100);
    let token_resource_2 = interface.mint_test_token(token_input_2, DIVISIBILITY_MAXIMUM);
    interface.update_collateral_configs(vec![(
        token_resource_2,
        CollateralConfig {
            pair_id: "TEST/USD".into(),
            price_age_max: 5,
            discount: dec!(0.95),
            margin: dec!(0.01),
        }
    )]).expect_commit_success();
    interface.add_collateral(
        margin_account_component,
        vec![(token_resource_2, token_input_2)],
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_COLLATERALS_TOO_MANY));
}

#[test]
fn test_add_collateral_invalid_collateral() {
    let mut interface = get_setup();

    let btc_resource = interface.mint_test_token(dec!(100), DIVISIBILITY_MAXIMUM);
    
    let rule_0 = rule!(allow_all);
    let result = interface.create_account(
        rule_0.clone(),
        vec![],
        None,
    ).expect_commit_success().clone();
    let margin_account_component = result.new_component_addresses()[0];

    let base_input_1 = dec!(1);
    interface.add_collateral(
        margin_account_component,
        vec![(btc_resource, base_input_1)],
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_COLLATERAL))
}
