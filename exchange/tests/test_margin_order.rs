#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_margin_order_long_open() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(70000)),
        Some(dec!(50000)),
    ).expect_commit_success();

    let pool_value_5 = interface.get_pool_value();
    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    let result_5 = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success().clone();

    let value = trade_size_4 * price_5;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value_5 * pair_config.fee_1;
    let fee_rate = (fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, trade_size_4);
    assert_eq!(pair_details.oi_short, dec!(0));

    let account_position = interface.get_account_details(margin_account_component, 0, None).positions[0].clone();
    assert_eq!(account_position.amount, trade_size_4);
    assert_eq!(account_position.cost, value + fee);
    assert_eq!(account_position.funding, dec!(0));

    let event: EventMarginOrder = interface.parse_event(&result_5);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_5);
    assert_eq!(event.amount_close, dec!(0));
    assert_eq!(event.amount_open, trade_size_4);
    assert_eq!(event.pnl, dec!(0));
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![2, 1]);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}

#[test]
fn test_margin_order_long_close_reduce_only() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_4 * dec!(2),
        true,
        Limit::None,
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value_6 * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}

#[test]
fn test_margin_order_long_close_profit() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(70000)),
        Some(dec!(50000)),
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(70000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = pool_value_6 - trade_size_4 * (price_6 - price_5);
    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![2]);
}

#[test]
fn test_margin_order_long_close_loss() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(70000)),
        Some(dec!(50000)),
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(50000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        2, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = pool_value_6 - trade_size_4 * (price_6 - price_5);
    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;

    let pool_details = interface.get_pool_details();
    println!("pool_details: {:?}", pool_details);
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![1]);
}

#[test]
fn test_margin_order_long_close_funding_positive() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let price_3 = dec!(60000);
    let amount_long_3 = dec!(1);
    let amount_short_3 = dec!(0.8);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_3, amount_short_3, price_3);

    let base_input_4 = dec!(1000);
    let result_4 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_4)], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_4.new_component_addresses()[0];

    let trade_size_5 = dec!(0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_5,
        false,
        Limit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_5,
        false,
        Limit::None,
        None,
        None,
    ).expect_commit_success();

    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1000);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success();

    let pool_value_7 = interface.get_pool_value();
    let pair_details_7 = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    let cost_7 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_7 = dec!(60000);
    let time_7 = interface.increment_ledger_time(10000);
    let result_7 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_7,
                timestamp: time_7,
            },
        ])
    ).expect_commit_success().clone();

    let oi_long = pair_details_7.oi_long;
    let oi_short = pair_details_7.oi_short;
    let oi_net = oi_long + oi_short;
    let skew = (oi_long - oi_short) * price_7;
    let skew_abs = skew.checked_abs().unwrap();
    let skew_abs_after = ((oi_long - oi_short - trade_size_5) * price_7).checked_abs().unwrap();
    let skew_delta = skew_abs_after - skew_abs;

    let period = Decimal::from(time_7.seconds_since_unix_epoch - time_6.seconds_since_unix_epoch);
    let funding_1_rate = skew * pair_config.funding_1;
    let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
    let funding_2_rate = (pair_details_7.funding_2 + funding_2_rate_delta) * pair_config.funding_2;
    let funding_long = (funding_1_rate + funding_2_rate) * period;
    let funding_share = funding_long * pair_config.funding_share;
    let funding_index_long = funding_long / oi_long;

    let funding_pool_0_rate = oi_net * price_7 * pair_config.funding_pool_0;
    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;
    let funding_pool = funding_pool_rate * period;
    let funding_pool_index = funding_pool / oi_net;
    let funding = (funding_index_long + funding_pool_index) * trade_size_5;

    let pool_value = pool_value_7 + funding_pool + funding_share;
    let value = trade_size_5 * price_6;
    let value_abs = value.checked_abs().unwrap();

    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_7 - fee - funding;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, amount_long_3);
    assert_eq!(pair_details.oi_short, amount_short_3);
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_7);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_7);
    assert_eq!(event.amount_close, -trade_size_5);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, -funding);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}

#[test]
fn test_margin_order_short_open() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(50000)),
        Some(dec!(70000)),
    ).expect_commit_success();

    let pool_value_5 = interface.get_pool_value();
    let price_5 = dec!(60000);
    let time_5 = interface.increment_ledger_time(1);
    let result_5 = interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_commit_success().clone();

    let value = trade_size_4 * price_5;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value_5 * pair_config.fee_1;
    let fee_rate = (fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, -trade_size_4);

    let account_position = interface.get_account_details(margin_account_component, 0, None).positions[0].clone();
    assert_eq!(account_position.amount, trade_size_4);
    assert_eq!(account_position.cost, value + fee);
    assert_eq!(account_position.funding, dec!(0));

    let event: EventMarginOrder = interface.parse_event(&result_5);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_5);
    assert_eq!(event.amount_close, dec!(0));
    assert_eq!(event.amount_open, trade_size_4);
    assert_eq!(event.pnl, dec!(0));
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![2, 1]);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}

#[test]
fn test_margin_order_short_close_reduce_only() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_4 * dec!(2),
        true,
        Limit::None,
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value_6 * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}

#[test]
fn test_margin_order_short_close_profit() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(50000)),
        Some(dec!(70000)),
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(50000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = pool_value_6 - trade_size_4 * (price_6 - price_5);
    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![2]);
}

#[test]
fn test_margin_order_short_close_loss() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
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
        Limit::None,
        Some(dec!(50000)),
        Some(dec!(70000)),
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

    let pool_value_6 = interface.get_pool_value();
    let cost_6 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_6 = dec!(70000);
    let time_6 = interface.increment_ledger_time(10000);
    let result_6 = interface.process_request(
        margin_account_component,
        2, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = pool_value_6 - trade_size_4 * (price_6 - price_5);
    let value = trade_size_4 * price_6;
    let value_abs = value.checked_abs().unwrap();
    let skew_delta = -value_abs; 
    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_6 - fee;
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, dec!(0));
    assert_eq!(pair_details.oi_short, dec!(0));
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_6);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_6);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, dec!(0));
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![1]);
}

#[test]
fn test_margin_order_short_close_funding_positive() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
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
        fee_1: dec!(0.1),
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

    let price_3 = dec!(60000);
    let amount_long_3 = dec!(0.8);
    let amount_short_3 = dec!(1);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_3, amount_short_3, price_3);

    let base_input_4 = dec!(1000);
    let result_4 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_4)], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_4.new_component_addresses()[0];

    let trade_size_5 = dec!(-0.02);
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        trade_size_5,
        false,
        Limit::None,
        None,
        None,
    ).expect_commit_success();
    interface.margin_order_tp_sl_request(
        0,
        10000000000,
        margin_account_component,
        pair_config.pair_id.clone(),
        -trade_size_5,
        false,
        Limit::None,
        None,
        None,
    ).expect_commit_success();

    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1000);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success();

    let pool_details_7 = interface.get_pool_details();
    let pool_value_7 = pool_details_7.base_tokens_amount + pool_details_7.virtual_balance + pool_details_7.unrealized_pool_funding + pool_details_7.pnl_snap;
    let pair_details_7 = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    let cost_7 = interface.get_account_details(margin_account_component, 0, None).positions[0].cost;
    let price_7 = dec!(60000);
    let time_7 = interface.increment_ledger_time(10000);
    let result_7 = interface.process_request(
        margin_account_component,
        1, 
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_7,
                timestamp: time_7,
            },
        ])
    ).expect_commit_success().clone();

    let oi_long = pair_details_7.oi_long;
    let oi_short = pair_details_7.oi_short;
    let oi_net = oi_long + oi_short;
    let skew = (oi_long - oi_short) * price_7;
    let skew_abs = ((oi_long - oi_short) * price_7).checked_abs().unwrap();
    let skew_abs_after = ((oi_long - oi_short - trade_size_5) * price_7).checked_abs().unwrap();
    let skew_delta = skew_abs_after - skew_abs;

    let period = Decimal::from(time_7.seconds_since_unix_epoch - time_6.seconds_since_unix_epoch);
    let funding_1_rate = skew * pair_config.funding_1;
    let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
    let funding_2_rate = (pair_details_7.funding_2 + funding_2_rate_delta) * pair_config.funding_2;
    let funding_short = -(funding_1_rate + funding_2_rate) * period;
    let funding_share = funding_short * pair_config.funding_share;
    let funding_index_short = funding_short / oi_short;

    let funding_pool_0_rate = oi_net * price_7 * pair_config.funding_pool_0;
    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;
    let funding_pool = funding_pool_rate * period;
    let funding_pool_index = funding_pool / oi_net;
    let funding = (funding_index_short + funding_pool_index) * -trade_size_5;

    let pool_value = pool_value_7 + funding_pool + funding_share;
    let value = trade_size_5 * price_6;
    let value_abs = value.checked_abs().unwrap();

    let fee_rate_0 = pair_config.fee_0;
    let fee_rate_1 = skew_delta / pool_value * pair_config.fee_1;
    let fee_rate = ((fee_rate_0 + fee_rate_1) * (dec!(1) - fee_rebate_0)).clamp(dec!(0), exchange_config.fee_max);

    let fee = value_abs * fee_rate;
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_7 - fee - funding;

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_7.base_tokens_amount);
    // assert_eq!(pool_details.virtual_balance, pool_details_7.virtual_balance - (pnl - fee_protocol - fee_treasury - fee_referral));
    assert_eq!(pool_details.unrealized_pool_funding, pool_details_7.unrealized_pool_funding + funding_pool + funding_share - funding);
    assert_eq!(pool_details.pnl_snap, pool_details_7.pnl_snap + (value - cost_7));
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, amount_long_3);
    assert_eq!(pair_details.oi_short, amount_short_3);
    
    let account_positions = interface.get_account_details(margin_account_component, 0, None).positions;
    assert_eq!(account_positions.len(), 0);

    let event: EventMarginOrder = interface.parse_event(&result_7);
    println!("funding: {:?}", event.funding);
    println!("fee: {:?}", event.fee_pool + event.fee_protocol + event.fee_treasury + event.fee_referral);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_7);
    assert_eq!(event.amount_close, -trade_size_5);
    assert_eq!(event.amount_open, dec!(0));
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, -funding);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.activated_requests, vec![] as Vec<ListIndex>);
    assert_eq!(event.cancelled_requests, vec![] as Vec<ListIndex>);
}