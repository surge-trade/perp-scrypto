use scrypto_test::prelude::*;
use exchange::*;

mod common;

use crate::common::*;

#[test]
fn test_hello() {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    let resources = create_resources(account, &mut ledger);
    let components = create_components(account, public_key, &resources, &mut ledger);   

    let pair_configs = vec![PairConfig {
        pair_id: "BTC/USD".into(),
        disabled: false,
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
    }];

    let receipt = ledger.call_method(
        components.exchange_component, 
        "update_pair_configs", 
        manifest_args!(pair_configs)
    );
    receipt.expect_commit_success();

    let fee_oath: Option<ManifestBucket> = None;
    let initial_rule: AccessRule = rule!(allow_all);
    let tokens: Vec<ManifestBucket> = vec![];
    let referral_code: Option<String> = None;
    let reservation: Option<ManifestAddressReservation> = None;

    let receipt = ledger.call_method(
        components.exchange_component, 
        "create_account", 
        manifest_args!(
            fee_oath,
            initial_rule,
            tokens,
            referral_code,
            reservation,
        ));
    let margin_account_component = receipt.expect_commit_success().new_component_addresses()[0];

    let fee_oath: Option<ManifestBucket> = None;
    let delay_seconds: u64 = 0;
    let expiry_seconds: u64 = 10000000000;
    let pair_id: PairId = "BTC/USD".into();
    let amount: Decimal = dec!(0.0000000001);
    let reduce_only: bool = false;
    let price_limit: Limit = Limit::None;
    let activate_requests: Vec<ListIndex> = vec![];
    let cancel_requests: Vec<ListIndex> = vec![];
    let status: Status = STATUS_ACTIVE;

    let receipt = ledger.call_method(
        components.exchange_component, 
        "margin_order_request", 
        manifest_args!(
            fee_oath,
            delay_seconds,
            expiry_seconds,
            margin_account_component,
            pair_id,
            amount,
            reduce_only,
            price_limit,
            activate_requests,
            cancel_requests,
            status,
        )
    );
    receipt.expect_commit_success();

    ledger.create_snapshot();

    // let manifest = ManifestBuilder::new()
    //     .lock_fee_from_faucet()
    //     .call_method(component, "free_token", manifest_args!())
    //     .call_method(
    //         account,
    //         "deposit_batch",
    //         manifest_args!(ManifestExpression::EntireWorktop),
    //     )
    //     .build();
    // let receipt = ledger.execute_manifest(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // receipt.expect_commit_success();
}
