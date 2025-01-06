#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_liquidate_to_margin_long() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(10))], 
        None,
    ).expect_commit_success().new_component_addresses()[0];
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let receiver_details_6 = interface.get_account_details(receiver_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

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

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral);
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);
    assert_eq!(account_details.valid_requests_start, 1);

    let receiver_details = interface.get_account_details(receiver_component, 0, None);
    assert_eq!(receiver_details.positions.len(), 0);
    assert_eq!(receiver_details.virtual_balance, receiver_details_6.virtual_balance - collateral_value_discounted);
    assert_eq!(receiver_details.collaterals.len(), 1);
    assert_eq!(receiver_details.collaterals[0].resource, btc_resource);
    assert_eq!(receiver_details.collaterals[0].amount, btc_input_3);

    let event_liquidate: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event_liquidate.account, margin_account_component);
    assert_eq!(event_liquidate.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event_liquidate.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event_liquidate.account_value, account_value);
    assert_eq!(event_liquidate.margin, margin);
    assert_eq!(event_liquidate.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event_liquidate.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event_liquidate.positions_pnl, pnl);
    assert_eq!(event_liquidate.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate.collateral_value, collateral_value);
    assert_eq!(event_liquidate.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event_liquidate.funding, dec!(0));
    assert_eq!(event_liquidate.fee_pool, -fee_pool);
    assert_eq!(event_liquidate.fee_protocol, -fee_protocol);
    assert_eq!(event_liquidate.fee_treasury, -fee_treasury);
    assert_eq!(event_liquidate.fee_referral, -fee_referral);
    assert_eq!(event_liquidate.pool_loss, dec!(0));

    let event_liquidate_to_margin: EventLiquidateToMargin = interface.parse_event(&result_6);
    assert_eq!(event_liquidate_to_margin.account, margin_account_component);
    assert_eq!(event_liquidate_to_margin.receiver, receiver_component);
    assert_eq!(event_liquidate_to_margin.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate_to_margin.collateral_value, collateral_value);
    assert_eq!(event_liquidate_to_margin.collateral_value_discounted, collateral_value_discounted);

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
}

#[test]
fn test_liquidate_to_margin_short() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(10))], 
        None,
    ).expect_commit_success().new_component_addresses()[0];
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let receiver_details_6 = interface.get_account_details(receiver_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(65000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

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

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral);
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);
    assert_eq!(account_details.valid_requests_start, 1);

    let receiver_details = interface.get_account_details(receiver_component, 0, None);
    assert_eq!(receiver_details.positions.len(), 0);
    assert_eq!(receiver_details.virtual_balance, receiver_details_6.virtual_balance - collateral_value_discounted);
    assert_eq!(receiver_details.collaterals.len(), 1);
    assert_eq!(receiver_details.collaterals[0].resource, btc_resource);
    assert_eq!(receiver_details.collaterals[0].amount, btc_input_3);

    let event_liquidate: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event_liquidate.account, margin_account_component);
    assert_eq!(event_liquidate.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event_liquidate.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event_liquidate.account_value, account_value);
    assert_eq!(event_liquidate.margin, margin);
    assert_eq!(event_liquidate.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event_liquidate.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event_liquidate.positions_pnl, pnl);
    assert_eq!(event_liquidate.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate.collateral_value, collateral_value);
    assert_eq!(event_liquidate.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event_liquidate.funding, dec!(0));
    assert_eq!(event_liquidate.fee_pool, -fee_pool);
    assert_eq!(event_liquidate.fee_protocol, -fee_protocol);
    assert_eq!(event_liquidate.fee_treasury, -fee_treasury);
    assert_eq!(event_liquidate.fee_referral, -fee_referral);
    assert_eq!(event_liquidate.pool_loss, dec!(0));

    let event_liquidate_to_margin: EventLiquidateToMargin = interface.parse_event(&result_6);
    assert_eq!(event_liquidate_to_margin.account, margin_account_component);
    assert_eq!(event_liquidate_to_margin.receiver, receiver_component);
    assert_eq!(event_liquidate_to_margin.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate_to_margin.collateral_value, collateral_value);
    assert_eq!(event_liquidate_to_margin.collateral_value_discounted, collateral_value_discounted);

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
}

#[test]
fn test_liquidate_to_margin_pool_loss() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(10))], 
        None,
    ).expect_commit_success().new_component_addresses()[0];
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let receiver_details_6 = interface.get_account_details(receiver_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    let result_6 = interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_commit_success().clone();

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

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral + pool_loss);
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, dec!(0));
    assert_eq!(account_details.valid_requests_start, 1);

    let receiver_details = interface.get_account_details(receiver_component, 0, None);
    assert_eq!(receiver_details.positions.len(), 0);
    assert_eq!(receiver_details.virtual_balance, receiver_details_6.virtual_balance - collateral_value_discounted);
    assert_eq!(receiver_details.collaterals.len(), 1);
    assert_eq!(receiver_details.collaterals[0].resource, btc_resource);
    assert_eq!(receiver_details.collaterals[0].amount, btc_input_3);

    let event_liquidate: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event_liquidate.account, margin_account_component);
    assert_eq!(event_liquidate.position_prices, vec![(pair_config.pair_id.clone(), price_6)]);
    assert_eq!(event_liquidate.collateral_prices, vec![(btc_resource, price_6)]);
    assert_eq!(event_liquidate.account_value, account_value);
    assert_eq!(event_liquidate.margin, margin);
    assert_eq!(event_liquidate.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event_liquidate.position_amounts, vec![(pair_config.pair_id.clone(), trade_size_4)]);
    assert_eq!(event_liquidate.positions_pnl, pnl);
    assert_eq!(event_liquidate.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate.collateral_value, collateral_value);
    assert_eq!(event_liquidate.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event_liquidate.funding, dec!(0));
    assert_eq!(event_liquidate.fee_pool, -fee_pool);
    assert_eq!(event_liquidate.fee_protocol, -fee_protocol);
    assert_eq!(event_liquidate.fee_treasury, -fee_treasury);
    assert_eq!(event_liquidate.fee_referral, -fee_referral);
    assert_eq!(event_liquidate.pool_loss, pool_loss);

    let event_liquidate_to_margin: EventLiquidateToMargin = interface.parse_event(&result_6);
    assert_eq!(event_liquidate_to_margin.account, margin_account_component);
    assert_eq!(event_liquidate_to_margin.receiver, receiver_component);
    assert_eq!(event_liquidate_to_margin.collateral_amounts, vec![(btc_resource, btc_input_3)]);
    assert_eq!(event_liquidate_to_margin.collateral_value, collateral_value);
    assert_eq!(event_liquidate_to_margin.collateral_value_discounted, collateral_value_discounted);

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
}

#[test]
fn test_liquidate_to_margin_sufficient_margin() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(10))], 
        None,
    ).expect_commit_success().new_component_addresses()[0];

    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
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
fn test_liquidate_to_margin_receiver_insufficient_margin() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![], 
        None,
    ).expect_commit_success().new_component_addresses()[0];
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INSUFFICIENT_MARGIN));
}

#[test]
fn test_liquidate_to_margin_receiver_same_as_margin_account() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let pair_config = default_pair_config("BTC/USD".into());
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

    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate_to_margin(
        margin_account_component, 
        margin_account_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ]),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_LIQUIDATION_RECEIVER_SAME_AS_ACCOUNT));
}

#[test]
fn test_liquidate_to_margin_collaterals_too_many() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let btc_resource = interface.mint_test_token(dec!(100), 8);
    let referral_resource = interface.resources.referral_resource;
    
    let collateral_config = CollateralConfig {
        pair_id: "BTC/USD".to_string(),
        price_age_max: 5,
        discount: dec!(0.90),
        margin: dec!(0.01),
    };
    interface.update_collateral_configs(vec![
        (btc_resource, collateral_config.clone()),
    ]).expect_commit_success();

    let mut test_pair_ids = vec![];
    let mut test_collateral_resources = vec![];
    let mut test_collateral_configs = vec![];
    for i in 0..exchange_config.collaterals_max {
        let collateral_resource = interface.mint_test_token(dec!(100), 8);
        let pair_id = format!("TEST{}/USD", i);
        let collateral_config = CollateralConfig {
            pair_id: pair_id.clone(),
            price_age_max: 5,
            discount: dec!(0.90),
            margin: dec!(0.01),
        };
        test_collateral_resources.push(collateral_resource);
        test_pair_ids.push(pair_id.clone());
        test_collateral_configs.push((collateral_resource, collateral_config));
    }
    interface.update_collateral_configs(test_collateral_configs.clone()).expect_commit_success();

    let receiver_component = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, dec!(10))].into_iter().chain(test_collateral_resources.iter().map(|r| (*r, dec!(1)))).collect(), 
        None,
    ).expect_commit_success().new_component_addresses()[0];

    let pair_config = default_pair_config("BTC/USD".into());
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
        ].into_iter().chain(test_pair_ids.iter().map(|pair_id| Price {
            pair: pair_id.clone(),
            quote: dec!(1),
            timestamp: time_5,
        })).collect::<Vec<Price>>())
    ).expect_commit_success();

    let price_6 = dec!(55000);
    let time_6 = interface.increment_ledger_time(30);
    interface.liquidate_to_margin(
        margin_account_component, 
        receiver_component, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            }
        ].into_iter().chain(test_pair_ids.iter().map(|pair_id| Price {
            pair: pair_id.clone(),
            quote: dec!(1),
            timestamp: time_6,
        })).collect::<Vec<Price>>()),
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_COLLATERALS_TOO_MANY));
}

// TODO: test funding
