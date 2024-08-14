#![allow(dead_code)]

use scrypto_test::prelude::*;
use once_cell::sync::OnceCell;
use super::*;

static SETUP: OnceCell<(LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components)> = OnceCell::new();

pub fn setup_dapp_definition(
    public_key: Secp256k1PublicKey,
    account: ComponentAddress,
    resources: &Resources,
    components: &Components,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) {
    let dapp_account = ledger.new_account_advanced(OwnerRole::Updatable(rule!(allow_all)));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, resources.owner_resource, dec!(1))
        .set_metadata(account, "account_type", MetadataValue::String("dapp definition".to_string()))
        .set_metadata(account, "name", MetadataValue::String("Surge".to_string()))
        .set_metadata(account, "description", MetadataValue::String("Feel the Surge!".to_string()))
        .set_metadata(account, "website", MetadataValue::OriginArray(vec![UncheckedOrigin::of("https://surge.trade")]))
        .set_metadata(account, "icon", MetadataValue::Url(UncheckedUrl::of("https://surge.trade/images/icon_dapp.png")))
        .set_metadata(resources.base_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(resources.lp_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(resources.protocol_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(resources.referral_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(resources.keeper_reward_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(components.fee_oath_resource, "dapp_definitions", MetadataValue::GlobalAddressArray(vec![dapp_account.into()]))
        .set_metadata(components.token_wrapper_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.token_wrapper_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.oracle_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.oracle_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.config_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.config_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.account_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.pool_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.pool_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.referral_generator_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.referral_generator_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.fee_distributor_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.fee_distributor_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.fee_delegator_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.fee_delegator_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.permission_registry_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.permission_registry_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.exchange_package, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .set_metadata(components.exchange_component, "dapp_definition", MetadataValue::GlobalAddress(dapp_account.into()))
        .build();
    ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]).expect_commit_success();
}

fn initialize_setup() -> (LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components) {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    let resources = create_resources(account, &mut ledger);
    let components = create_components(account, public_key, &resources, &mut ledger);
    setup_dapp_definition(public_key, account, &resources, &components, &mut ledger);
    let snapshot = ledger.create_snapshot();

    (snapshot, public_key, account, resources, components)
}

pub fn get_setup() -> ExchangeInterface {
    let setup = SETUP.get_or_init(initialize_setup);
    let snapshot = setup.0.clone();
    let public_key = setup.1.clone();
    let account = setup.2.clone();
    let resources = setup.3.clone();
    let components = setup.4.clone();
    
    let ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build_from_snapshot(snapshot);
    let exchange = ExchangeInterface::new(public_key, account, resources, components, ledger);

    exchange
}