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
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);
    assert_eq!(account_details.valid_requests_start, 1);

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

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
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
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_value);
    assert_eq!(account_details.valid_requests_start, 1);

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

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
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
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.pool_position.oi_long, dec!(0));
    assert_eq!(pair_details.pool_position.oi_short, dec!(0));

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, dec!(0));
    assert_eq!(account_details.valid_requests_start, 1);

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

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, 1);
}

#[test]
fn test_liquidate_many_positions_collaterals_and_orders() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let mut pair_ids = hashset![];
    let mut position_pair_ids = vec![];
    let mut collateral_resources = vec![];
    let mut collateral_configs = vec![];
    let mut pair_configs = vec![];
    for i in 0..exchange_config.collaterals_max {
        let collateral_resource = interface.mint_test_token(dec!(100), 8);
        let pair_id = format!("TEST{}/USD", i);
        let collateral_config = CollateralConfig {
            pair_id: pair_id.clone(),
            price_age_max: 5,
            discount: dec!(0.90),
            margin: dec!(0.01),
        };
        collateral_resources.push(collateral_resource);
        pair_ids.insert(pair_id.clone());
        collateral_configs.push((collateral_resource, collateral_config));
    }
    for i in 0..exchange_config.positions_max {
        let pair_id = format!("TEST{}/USD", i);
        let pair_config = PairConfig {
            pair_id: pair_id.clone(),
            price_age_max: 5,
            oi_max: dec!(200000),
            trade_size_min: dec!(0),
            update_price_delta_ratio: dec!(0.005),
            update_period_seconds: 3600,
            margin_initial: dec!(0.01),
            margin_maintenance: dec!(0.005),
            funding_1: dec!(1),
            funding_2: dec!(1),
            funding_2_delta: dec!(100),
            funding_2_decay: dec!(100),
            funding_pool_0: dec!(0.02),
            funding_pool_1: dec!(0.25),
            funding_share: dec!(0.02),
            fee_0: dec!(0.0005),
            fee_1: dec!(0.0000000005),
        };
        pair_ids.insert(pair_id.clone());
        position_pair_ids.push(pair_id.clone());
        pair_configs.push(pair_config);
    }
    let pair_ids = pair_ids.into_iter().collect::<Vec<_>>();
    interface.update_collateral_configs(collateral_configs.clone()).expect_commit_success();
    interface.update_pair_configs(pair_configs.clone()).expect_commit_success();

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

    let base_input_3 = dec!(20);
    let collateral_input_3 = dec!(20);
    let tokens_3 = vec![(base_resource, base_input_3)]
        .into_iter().chain(collateral_resources.iter().map(|resource| (*resource, collateral_input_3))).collect();
    let result_3 = interface.create_account(
        rule!(allow_all), 
        tokens_3, 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let trade_size_4 = dec!(150);
    for pair_id in position_pair_ids.iter() {
        interface.margin_order_tp_sl_request(
            0,
            10000000000,
            margin_account_component,
            pair_id.clone(),
            trade_size_4,
            false,
            PriceLimit::None,
            SlippageLimit::None,
            None,
            None,
            ).expect_commit_success();
    }
    let price_5 = dec!(10);
    let time_5 = interface.increment_ledger_time(1);
    let prices_5: Vec<Price> = pair_ids.iter().map(|pair_id| Price {
        pair: pair_id.clone(),
        quote: price_5,
        timestamp: time_5,
    }).collect();
    for i in 0..position_pair_ids.len() {
        let transaction = interface.process_request(
            margin_account_component,
            i as ListIndex, 
            Some(prices_5.clone()),
        );
        // println!("transaction: {:?}", transaction.fee_summary);
        transaction.expect_commit_success();
    }
    for _ in 0..exchange_config.active_requests_max {
        let transaction = interface.margin_order_request(
            0, 
            10000000000, 
            margin_account_component, 
            position_pair_ids[0].clone(), 
            dec!(100), 
            false, 
            PriceLimit::Gte(dec!(10000)), 
            SlippageLimit::None, 
            vec![RequestIndexRef::Index(1), RequestIndexRef::Index(2)], 
            vec![RequestIndexRef::Index(3), RequestIndexRef::Index(4)], 
            STATUS_ACTIVE
        );
        // println!("transaction: {:?}", transaction.fee_summary);
        transaction.expect_commit_success();
    }

    let base_input_6 = dec!(10000);
    let base_balance_6 = interface.test_account_balance(base_resource);
    let collateral_balances_6: Vec<_> = collateral_resources.iter().map(|resource| interface.test_account_balance(*resource)).collect();
    let pool_details_6 = interface.get_pool_details();
    let account_details_6 = interface.get_account_details(margin_account_component, 0, None);
    let cost_6 = account_details_6.positions[0].cost;
    let price_6 = dec!(5);
    let time_6 = interface.increment_ledger_time(30);
    let prices_6: Vec<Price> = pair_ids.iter().map(|pair_id| Price {
        pair: pair_id.clone(),
        quote: price_6,
        timestamp: time_6,
    }).collect();
    let transaction_6 = interface.liquidate(
        margin_account_component, 
        (base_resource, base_input_6), 
        Some(prices_6),
    );
    // println!("transaction_6: {:?}", transaction_6.fee_summary);
    let result_6 = transaction_6.expect_commit_success().clone();

    let base_balance_7 = interface.test_account_balance(base_resource);
    let collateral_balances_7: Vec<_> = collateral_resources.iter().map(|resource| interface.test_account_balance(*resource)).collect();
    let base_output_7 = base_balance_7 - base_balance_6 + base_input_6;
    let collateral_outputs_7: Vec<_> = collateral_balances_7.iter().zip(collateral_balances_6.iter()).map(|(a, b)| *a - *b).collect();

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_2_delta = -(value * value); 
    let fee_0 = value_abs * pair_configs[0].fee_0;
    let fee_1 = skew_2_delta * pair_configs[0].fee_1;

    let fee = (fee_0 + fee_1) * (dec!(1) - fee_rebate_0) * exchange_config.positions_max;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = (value - cost_6) * exchange_config.positions_max - fee;
    let collateral_value = collateral_input_3 * price_6 * exchange_config.collaterals_max;
    let collateral_value_discounted = collateral_value * collateral_configs[0].1.discount;
    let account_value = account_details_6.virtual_balance + collateral_value_discounted + pnl;
    let margin = value_abs * pair_configs[0].margin_maintenance * exchange_config.positions_max + collateral_value * collateral_configs[0].1.margin;
    let pool_loss = account_value.min(dec!(0));

    assert_eq!(base_output_7, base_input_6 - collateral_value_discounted);
    for &collateral_output_7 in collateral_outputs_7.iter() {
        assert_eq!(collateral_output_7, collateral_input_3);
    }

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_6.base_tokens_amount + collateral_value_discounted);
    assert_eq!(pool_details.virtual_balance, pool_details_6.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral - collateral_value_discounted + pool_loss);
    assert_eq!(pool_details.pnl_snap, dec!(0));
    
    for pair_id in position_pair_ids.iter() {
        let pair_details = interface.get_pair_details(vec![pair_id.clone()])[0].clone();
        assert_eq!(pair_details.pool_position.oi_long, dec!(0));
        assert_eq!(pair_details.pool_position.oi_short, dec!(0));
    }

    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, dec!(0));
    assert_eq!(account_details.valid_requests_start, (exchange_config.positions_max + exchange_config.active_requests_max) as ListIndex);

    let event_liquidate: EventLiquidate = interface.parse_event(&result_6);
    assert_eq!(event_liquidate.account, margin_account_component);
    assert_eq!(event_liquidate.position_prices.into_iter().collect::<HashSet<_>>(), position_pair_ids.iter().map(|pair_id| (pair_id.clone(), price_6)).collect::<HashSet<_>>());
    assert_eq!(event_liquidate.collateral_prices.into_iter().collect::<HashSet<_>>(), collateral_resources.iter().map(|resource| (*resource, price_6)).collect::<HashSet<_>>());
    assert_eq!(event_liquidate.account_value, account_value);
    assert_eq!(event_liquidate.margin, margin);
    assert_eq!(event_liquidate.virtual_balance, account_details_6.virtual_balance);
    assert_eq!(event_liquidate.position_amounts.into_iter().collect::<HashSet<_>>(), position_pair_ids.iter().map(|pair_id| (pair_id.clone(), trade_size_4)).collect::<HashSet<_>>());
    assert_eq!(event_liquidate.positions_pnl, pnl);
    assert_eq!(event_liquidate.collateral_amounts.into_iter().collect::<HashSet<_>>(), collateral_resources.iter().map(|resource| (*resource, collateral_input_3)).collect::<HashSet<_>>());
    assert_eq!(event_liquidate.collateral_value, collateral_value);
    assert_eq!(event_liquidate.collateral_value_discounted, collateral_value_discounted);
    assert_eq!(event_liquidate.funding, dec!(0));
    assert_eq!(event_liquidate.fee_pool, -fee_pool);
    assert_eq!(event_liquidate.fee_protocol, -fee_protocol);
    assert_eq!(event_liquidate.fee_treasury, -fee_treasury);
    assert_eq!(event_liquidate.fee_referral, -fee_referral);
    assert_eq!(event_liquidate.pool_loss, pool_loss);

    let event_requests_start: EventValidRequestsStart = interface.parse_event(&result_6);
    assert_eq!(event_requests_start.account, margin_account_component);
    assert_eq!(event_requests_start.valid_requests_start, (exchange_config.positions_max + exchange_config.active_requests_max) as ListIndex);
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