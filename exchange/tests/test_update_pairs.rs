#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_update_pairs_long_skew() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

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

    let base_input_0 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let amount_long_1 = dec!(1);
    let amount_short_1 = dec!(0.7);
    let price_1 = dec!(60000);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_1, amount_short_1, price_1);
    let time_1 = interface.ledger_time();
    
    let pool_details_2 = interface.get_pool_details();
    let pair_details_2 = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    let pool_position_2 = interface.get_pool_position(pair_config.pair_id.clone());
    let price_2 = dec!(65000);
    let time_2 = interface.increment_ledger_time(10000);
    let result_2 = interface.update_pairs(
        vec![pair_config.pair_id.clone()],
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_2,
                timestamp: time_2,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = interface.get_pool_value();
    let oi_long = pair_details_2.oi_long;
    let oi_short = pair_details_2.oi_short;
    let oi_net = oi_long + oi_short;
    let skew = (oi_long - oi_short) * price_2;
    let skew_abs = skew.checked_abs().unwrap();

    let skew_last = (oi_long - oi_short) * price_1;
    let pnl_snap_delta = (pool_position_2.cost - skew) - (pool_position_2.cost - skew_last);

    let period = Decimal::from(time_2.seconds_since_unix_epoch - time_1.seconds_since_unix_epoch);
    let funding_1_rate = skew * pair_config.funding_1;
    let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
    let funding_2_rate = (pair_details_2.funding_2 + funding_2_rate_delta) * pair_config.funding_2;
    let funding_long = (funding_1_rate + funding_2_rate) * period;
    let funding_share = funding_long * pair_config.funding_share;
    let funding_short = -(funding_long - funding_share);

    let funding_pool_0_rate = oi_net * price_2 * pair_config.funding_pool_0;
    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;
    let funding_pool = funding_pool_rate * period;
    
    let funding_pool_index = funding_pool / oi_net;
    let funding_index_long = funding_long / oi_long + funding_pool_index;
    let funding_index_short = funding_short / oi_short + funding_pool_index;

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.unrealized_pool_funding, funding_pool + funding_share);
    assert_eq!(pool_details.pnl_snap, pool_details_2.pnl_snap + pnl_snap_delta);
    assert_eq!(pool_details.skew_ratio, skew_abs / pool_value);

    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, amount_long_1);
    assert_eq!(pair_details.oi_short, amount_short_1);
    assert_eq!(pair_details.funding_2, pair_details_2.funding_2 + funding_2_rate_delta);

    let event: EventPairUpdates = interface.parse_event(&result_2);
    assert_eq!(event.updates.len(), 1);
    assert_eq!(event.updates[0].0, pair_config.pair_id);

    let update = &event.updates[0].1;
    assert_eq!(update.oi_long, oi_long);
    assert_eq!(update.oi_short, oi_short);
    assert_eq!(update.cost, pool_position_2.cost);
    assert_eq!(update.skew_abs_snap, skew_abs);
    assert_eq!(update.pnl_snap, pool_details_2.pnl_snap + pnl_snap_delta);
    assert_eq!(update.funding_2_rate, pool_position_2.funding_2_rate + funding_2_rate_delta);
    assert_eq!(update.funding_long_index, pool_position_2.funding_long_index + funding_index_long);
    assert_eq!(update.funding_short_index, pool_position_2.funding_short_index + funding_index_short);
    assert_eq!(update.last_update, time_2);
    assert_eq!(update.last_price, price_2);
}

#[test]
fn test_update_pairs_short_skew() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

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

    let base_input_0 = dec!(1000000);
    interface.add_liquidity((base_resource, base_input_0)).expect_commit_success();

    let amount_long_1 = dec!(0.7);
    let amount_short_1 = dec!(1);
    let price_1 = dec!(60000);
    interface.make_open_interest(pair_config.pair_id.clone(), amount_long_1, amount_short_1, price_1);
    let time_1 = interface.ledger_time();
    
    let pool_details_2 = interface.get_pool_details();
    let pair_details_2 = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    let pool_position_2 = interface.get_pool_position(pair_config.pair_id.clone());
    let price_2 = dec!(55000);
    let time_2 = interface.increment_ledger_time(10000);
    let result_2 = interface.update_pairs(
        vec![pair_config.pair_id.clone()],
        Some(vec![
            Price {
                pair: pair_config.pair_id.clone(),
                quote: price_2,
                timestamp: time_2,
            },
        ])
    ).expect_commit_success().clone();

    let pool_value = interface.get_pool_value();
    let oi_long = pair_details_2.oi_long;
    let oi_short = pair_details_2.oi_short;
    let oi_net = oi_long + oi_short;
    let skew = (oi_long - oi_short) * price_2;
    let skew_abs = skew.checked_abs().unwrap();

    let skew_last = (oi_long - oi_short) * price_1;
    let pnl_snap_delta = (pool_position_2.cost - skew) - (pool_position_2.cost - skew_last); // as cost has not changed

    let period = Decimal::from(time_2.seconds_since_unix_epoch - time_1.seconds_since_unix_epoch);
    let funding_1_rate = skew * pair_config.funding_1;
    let funding_2_rate_delta = skew * pair_config.funding_2_delta * period;
    let funding_2_rate = (pair_details_2.funding_2 + funding_2_rate_delta) * pair_config.funding_2;
    let funding_short = -(funding_1_rate + funding_2_rate) * period;
    let funding_share = funding_short * pair_config.funding_share;
    let funding_long = -(funding_short - funding_share);

    let funding_pool_0_rate = oi_net * price_2 * pair_config.funding_pool_0;
    let funding_pool_1_rate = skew_abs * pair_config.funding_pool_1;
    let funding_pool_rate = funding_pool_0_rate + funding_pool_1_rate;
    let funding_pool = funding_pool_rate * period;
    
    let funding_pool_index = funding_pool / oi_net;
    let funding_index_long = funding_long / oi_long + funding_pool_index;
    let funding_index_short = funding_short / oi_short + funding_pool_index;

    let pool_details = interface.get_pool_details();
    assert_eq!(pool_details.unrealized_pool_funding, funding_pool + funding_share);
    assert_eq!(pool_details.pnl_snap, pool_details_2.pnl_snap + pnl_snap_delta);
    assert_eq!(pool_details.skew_ratio, skew_abs / pool_value);

    let pair_details = interface.get_pair_details(vec![pair_config.pair_id.clone()])[0].clone();
    assert_eq!(pair_details.oi_long, amount_long_1);
    assert_eq!(pair_details.oi_short, amount_short_1);
    assert_eq!(pair_details.funding_2, pair_details_2.funding_2 + funding_2_rate_delta);

    let event: EventPairUpdates = interface.parse_event(&result_2);
    assert_eq!(event.updates.len(), 1);
    assert_eq!(event.updates[0].0, pair_config.pair_id);

    let update = &event.updates[0].1;
    assert_eq!(update.oi_long, oi_long);
    assert_eq!(update.oi_short, oi_short);
    assert_eq!(update.cost, pool_position_2.cost);
    assert_eq!(update.skew_abs_snap, skew_abs);
    assert_eq!(update.pnl_snap, pool_details_2.pnl_snap + pnl_snap_delta);
    assert_eq!(update.funding_2_rate, pool_position_2.funding_2_rate + funding_2_rate_delta);
    assert_eq!(update.funding_long_index, pool_position_2.funding_long_index + funding_index_long);
    assert_eq!(update.funding_short_index, pool_position_2.funding_short_index + funding_index_short);
    assert_eq!(update.last_update, time_2);
    assert_eq!(update.last_price, price_2);
}
