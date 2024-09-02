#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_auto_deleverage() {
    let mut interface = get_setup();
    let exchange_config = interface.get_exchange_config();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(100000),
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

    let base_input_2 = dec!(100000);
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
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let amount_long_5 = dec!(0.225);
    let amount_short_5 = dec!(0.01);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_5, amount_short_5, price_5);

    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1);
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
    let account_details_7 = interface.get_account_details(margin_account_component, 0, None);
    let cost_7 = account_details_7.positions[0].cost;
    let price_7 = dec!(75000);
    let time_7 = interface.increment_ledger_time(10000);
    let result_7 = interface.auto_deleverage(
        margin_account_component,
        pair_config.pair_id.clone(),
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
    let skew_abs_capped = skew_abs.min(oi_long.max(oi_short) * price_7 * dec!(0.1));
    let skew_capped = skew_abs_capped * skew.0.signum();
    let skew_after = (oi_long - oi_short - trade_size_4) * price_7;
    let skew_2 = skew * skew;
    let skew_2_after = skew_after * skew_after;
    let skew_2_delta = skew_2_after - skew_2;

    let period = Decimal::from(time_7.seconds_since_unix_epoch - time_6.seconds_since_unix_epoch);
    let funding_1_rate = skew_capped * pair_config.funding_1;
    let funding_2_rate_delta = skew_capped * pair_config.funding_2_delta * period;
    let funding_2_rate = (pair_details_7.funding_2 + funding_2_rate_delta) * pair_config.funding_2;
    let funding_long = (funding_1_rate + funding_2_rate) * period;
    let funding_share = funding_long * pair_config.funding_share;
    let funding_index_long = funding_long / oi_long;

    let funding_pool_0_rate = oi_net * price_7 * pair_config.funding_pool_0;
    let funding_pool_1_rate = skew_abs_capped * pair_config.funding_pool_1;
    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;
    let funding_pool = funding_pool_rate * period;
    let funding_pool_index = funding_pool / oi_net;
    let funding = (funding_index_long + funding_pool_index) * trade_size_4;

    let value = trade_size_4 * price_7;
    let value_abs = value.checked_abs().unwrap();

    let fee_0 = value_abs * pair_config.fee_0;
    let fee_1 = skew_2_delta * pair_config.fee_1;
    
    let fee = (fee_0 + fee_1) * (dec!(1) - fee_rebate_0);
    let fee_protocol = fee * exchange_config.fee_share_protocol;
    let fee_treasury = fee * exchange_config.fee_share_treasury;
    let fee_referral = fee * fee_referral_0 * exchange_config.fee_share_referral;
    let fee_pool = fee - fee_protocol - fee_treasury - fee_referral;

    let pnl = value - cost_7 - fee - funding;
    let pnl_percent = (value - cost_7) / cost_7;

    let pnl_snap_delta_a = -(oi_long - oi_short) * (price_7 - price_6);
    let pnl_snap_delta_b = value - cost_7;
    let skew_ratio = skew_abs / (pool_value_7 + pnl_snap_delta_a + funding_pool + funding_share);
    let u = skew_ratio / exchange_config.adl_a - exchange_config.adl_offset / exchange_config.adl_a;
    let threshold = -(u * u * u) - exchange_config.adl_b * u;

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.base_tokens_amount, pool_details_7.base_tokens_amount);
    assert_eq!(pool_details.virtual_balance, pool_details_7.virtual_balance - pnl - fee_protocol - fee_treasury - fee_referral);
    assert_eq!(pool_details.unrealized_pool_funding, pool_details_7.unrealized_pool_funding + funding_pool + funding_share - funding);
    assert_eq!(pool_details.pnl_snap, pool_details_7.pnl_snap + pnl_snap_delta_a + pnl_snap_delta_b);
    
    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, amount_long_5);
    assert_eq!(pair_details.oi_short, amount_short_5);
    
    let account_details = interface.get_account_details(margin_account_component, 0, None);
    assert_eq!(account_details.positions.len(), 0);
    assert_eq!(account_details.virtual_balance, account_details_7.virtual_balance + pnl);

    let event: EventAutoDeleverage = interface.parse_event(&result_7);
    assert_eq!(event.account, margin_account_component);
    assert_eq!(event.pair_id, pair_config.pair_id.clone());
    assert_eq!(event.price, price_7);
    assert_eq!(event.amount_close, -trade_size_4);
    assert_eq!(event.pnl, pnl);
    assert_eq!(event.funding, -funding);
    assert_eq!(event.fee_pool, -fee_pool);
    assert_eq!(event.fee_protocol, -fee_protocol);
    assert_eq!(event.fee_treasury, -fee_treasury);
    assert_eq!(event.fee_referral, -fee_referral);
    assert_eq!(event.pnl_percent, pnl_percent);
    assert_eq!(event.threshold, threshold);
}

#[test]
fn test_auto_deleverage_skew_too_low() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(100000),
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

    let base_input_2 = dec!(100000);
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
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let amount_long_5 = dec!(0.2);
    let amount_short_5 = dec!(0.1);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_5, amount_short_5, price_5);

    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1);
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

    let price_7 = dec!(75000);
    let time_7 = interface.increment_ledger_time(10000);
    interface.auto_deleverage(
        margin_account_component,
        pair_config.pair_id.clone(),
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_7,
                timestamp: time_7,
            },
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ADL_SKEW_TOO_LOW));
}

#[test]
fn test_auto_deleverage_no_position() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(100000),
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

    let base_input_2 = dec!(100000);
    interface.add_liquidity((base_resource, base_input_2)).expect_commit_success();

    let base_input_3 = dec!(1000);
    let result_3 = interface.create_account(
        rule!(allow_all), 
        vec![(base_resource, base_input_3)], 
        Some(referral_code_1),
    ).expect_commit_success().clone();
    let margin_account_component = result_3.new_component_addresses()[0];

    let price_4 = dec!(60000);
    let amount_long_4 = dec!(0.23);
    let amount_short_4 = dec!(0.01);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_4, amount_short_4, price_4);

    let price_5 = dec!(75000);
    let time_5 = interface.increment_ledger_time(10000);
    interface.auto_deleverage(
        margin_account_component,
        pair_config.pair_id.clone(),
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_5,
                timestamp: time_5,
            },
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ADL_NO_POSITION));
}

#[test]
fn test_auto_deleverage_below_threshold() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let pair_config = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(100000),
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

    let base_input_2 = dec!(100000);
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
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let price_5 = dec!(60000);
    let amount_long_5 = dec!(0.2);
    let amount_short_5 = dec!(0.01);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_5, amount_short_5, price_5);

    let price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1);
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

    let price_7 = dec!(75000);
    let time_7 = interface.increment_ledger_time(10000);
    interface.auto_deleverage(
        margin_account_component,
        pair_config.pair_id.clone(),
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_7,
                timestamp: time_7,
            },
        ])
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ADL_PNL_BELOW_THRESHOLD));
}

#[test]
fn test_auto_deleverage_skew_not_reduced() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;
    
    let pair_config_btc = PairConfig {
        pair_id: "BTC/USD".into(),
        oi_max: dec!(100000),
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
    let pair_config_xrd = PairConfig {
        pair_id: "XRD/USD".into(),
        oi_max: dec!(100000),
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
    interface.update_pair_configs(vec![pair_config_btc.clone(), pair_config_xrd.clone()]).expect_commit_success();

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

    let base_input_2 = dec!(100000);
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
        pair_config_btc.pair_id.clone(),
        trade_size_4,
        false,
        PriceLimit::None,
        SlippageLimit::None,
        None,
        None,
    ).expect_commit_success();

    let btc_price_5 = dec!(60000);
    let btc_amount_long_5 = dec!(0.05);
    let btc_amount_short_5 = dec!(0.01);
    interface.make_open_interest(pair_config_btc.pair_id.clone(), btc_amount_long_5, btc_amount_short_5, btc_price_5);
    let xrd_price_5 = dec!(0.05);
    let xrd_amount_long_5 = dec!(150000);
    let xrd_amount_short_5 = dec!(10000);
    interface.make_open_interest(pair_config_xrd.pair_id.clone(), xrd_amount_long_5, xrd_amount_short_5, xrd_price_5);

    let btc_price_6 = dec!(60000);
    let time_6 = interface.increment_ledger_time(1);
    interface.process_request(
        margin_account_component,
        0, 
        Some(vec![
            Price {
                pair: pair_config_btc.pair_id.clone(),
                quote: btc_price_6,
                timestamp: time_6,
            },
        ])
    ).expect_commit_success();

    let btc_price_7 = dec!(40000);
    let xrd_price_7 = dec!(0.1);
    let time_7 = interface.increment_ledger_time(10000);
    interface.update_pairs(
        vec![pair_config_btc.pair_id.clone(), pair_config_xrd.pair_id.clone()],
        Some(vec![
            Price {
                pair: pair_config_btc.pair_id.clone(),
                quote: btc_price_7,
                timestamp: time_7,
            },
            Price {
                pair: pair_config_xrd.pair_id.clone(),
                quote: xrd_price_7,
                timestamp: time_7,
            },
        ]),
    ).expect_commit_success();


    interface.auto_deleverage(
        margin_account_component,
        pair_config_btc.pair_id.clone(),
        None,
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ADL_PNL_BELOW_THRESHOLD));
}
