#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_liquidate_long() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(75);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(100);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let btc_balance_6 = interface.test_account_balance(btc_resource);
    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let btc_balance_7 = interface.test_account_balance(btc_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let btc_output_7 = btc_balance_7 - btc_balance_6;

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_2_delta = -(value * value); 
    let fee_0 = value_abs * pair_config.fee_0;
    let fee_1 = skew_2_delta * pair_config.fee_1;

    let fee = (fee_0 + fee_1) * (dec!(1) - fee_rebate_0);
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    let collateral_value = btc_input_3 * price_6;
    let collateral_value_discounted = collateral_value * collateral_config.discount;
    let account_value = account_details_6.virtual_balance + collateral_value_discounted + pnl;
    let margin = value_abs * pair_config.margin_maintenance + collateral_value * collateral_config.margin;

    assert_eq!(base_output_7, base_input_6 - collateral_value_discounted);
    assert_eq!(btc_output_7, btc_input_3);

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount + collateral_value_discounted);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral - collateral_value_discounted);
    assert_eq!(pool_details.unrealized_pool_funding, pool_details_6.unrealized_pool_funding);
    assert_eq!(pool_details.pnl_snap, pool_details_6.pnl_snap);
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);

    let event: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event.account_value, account_value);
    assert_eq!(event.margin, margin);
    assert_eq!(event.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event.positions_pnl, pnl);
    assert_eq!(event.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event.collateral_value, collateral_value);
    assert_eq!(event.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.pool_loss, dec!(0));
}

#[test]
fn test_liquidate_short() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(70);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(-0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(100);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let btc_balance_6 = interface.test_account_balance(btc_resource);
    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(65000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let btc_balance_7 = interface.test_account_balance(btc_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let btc_output_7 = btc_balance_7 - btc_balance_6;

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_2_delta = -(value * value); 
    let fee_0 = value_abs * pair_config.fee_0;
    let fee_1 = skew_2_delta * pair_config.fee_1;

    let fee = (fee_0 + fee_1) * (dec!(1) - fee_rebate_0);
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    let collateral_value = btc_input_3 * price_6;
    let collateral_value_discounted = collateral_value * collateral_config.discount;
    let account_value = account_details_6.virtual_balance + collateral_value_discounted + pnl;
    let margin = value_abs * pair_config.margin_maintenance + collateral_value * collateral_config.margin;

    assert_eq!(base_output_7, base_input_6 - collateral_value_discounted);
    assert_eq!(btc_output_7, btc_input_3);

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount + collateral_value_discounted);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral - collateral_value_discounted);
    assert_eq!(pool_details.unrealized_pool_funding, pool_details_6.unrealized_pool_funding);
    assert_eq!(pool_details.pnl_snap, pool_details_6.pnl_snap);
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);

    let event: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event.account_value, account_value);
    assert_eq!(event.margin, margin);
    assert_eq!(event.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event.positions_pnl, pnl);
    assert_eq!(event.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event.collateral_value, collateral_value);
    assert_eq!(event.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.pool_loss, dec!(0));
}

#[test]
fn test_liquidate_pool_loss() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(60);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(100);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let btc_balance_6 = interface.test_account_balance(btc_resource);
    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let btc_balance_7 = interface.test_account_balance(btc_resource);
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let btc_output_7 = btc_balance_7 - btc_balance_6;

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_2_delta = -(value * value); 
    let fee_0 = value_abs * pair_config.fee_0;
    let fee_1 = skew_2_delta * pair_config.fee_1;

    let fee = (fee_0 + fee_1) * (dec!(1) - fee_rebate_0);
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    let collateral_value = btc_input_3 * price_6;
    let collateral_value_discounted = collateral_value * collateral_config.discount;
    let account_value = account_details_6.virtual_balance + collateral_value_discounted + pnl;
    let margin = value_abs * pair_config.margin_maintenance + collateral_value * collateral_config.margin;
    let pool_loss = account_value;

    assert_eq!(base_output_7, base_input_6 - collateral_value_discounted);
    assert_eq!(btc_output_7, btc_input_3);

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount + collateral_value_discounted);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral - collateral_value_discounted + pool_loss);
    assert_eq!(pool_details.unrealized_pool_funding, pool_details_6.unrealized_pool_funding);
    assert_eq!(pool_details.pnl_snap, pool_details_6.pnl_snap);
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, dec!(0));

    let event: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event.account_value, account_value);
    assert_eq!(event.margin, margin);
    assert_eq!(event.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event.positions_pnl, pnl);
    assert_eq!(event.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event.collateral_value, collateral_value);
    assert_eq!(event.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.pool_loss, pool_loss);
}

#[test]
fn test_liquidate_invalid_payment_token() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let fake_base_resource = interface.mint_test_token(dec!(1000), 18);
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(75);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(1);
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate(
        margin_account_component, 
        (fake_base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_PAYMENT_TOKEN));
}

#[test]
fn test_liquidate_sufficient_margin() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(200);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(100);
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_LIQUIDATION_SUFFICIENT_MARGIN));
}

#[test]
fn test_liquidate_insufficient_payment() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(200000),
        update_price_delta_ratio: dec!(0.005),
        update_period_seconds: 3600,
        margin_initial: dec!(0.01),
        margin_maintenance: dec!(0.005),
        funding_1: dec!(0.0000000317),
        funding_2: dec!(0.0000000317),
        funding_2_delta: dec!(0.000000827),
        funding_pool_0: dec!(0.0000000159),
        funding_pool_1: dec!(0.0000000317),
        funding_share: dec!(0.1),
        fee_0: dec!(0.0005),
        fee_1: dec!(0.0000000005),
    };
    interface.update_pair_configs(vec![pair_config.clone()]).expect_commit_success();

    let fee_referral_0 = dec!(0.10);
    let fee_rebate_0 = dec!(0.05);
    let result_0 = interface.mint_referral(fee_referral_0, fee_rebate_0, 1).expect_commit_success().clone();
    
    let referral_id_1 = parse_added_nft_ids(&result_0, referral_resource).first().unwrap().clone();
    let referral_code_1 = "test".to_string();
    let referral_hash_1 = keccak256_hash(referral_code_1.clone().into_bytes());
    let referral_hashes_1 = hashmap!(
        referral_hash_1 => (vec![], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id_1), vec![], referral_hashes_1).expect_commit_success();

    let base_input_2 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(75);
    let btc_input_3 = dec!(0.0006);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![
            (base_resource, base_input_3),
            (btc_resource, btc_input_3),
        ], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success();

    let base_input_6 = dec!(10);
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INSUFFICIENT_PAYMENT));
}

// TODO: test funding