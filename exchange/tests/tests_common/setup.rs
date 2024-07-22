use scrypto_test::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use super::*;

pub static SETUP: Lazy<Mutex<(LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components)>> = Lazy::new(|| Mutex::new(initialize_setup()));

fn initialize_setup() -> (LedgerSimulatorSnapshot, Secp256k1PublicKey, ComponentAddress, Resources, Components) {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    let resources = create_resources(account, &mut ledger);
    let components = create_components(account, public_key, &resources, &mut ledger);   
    let snapshot = ledger.create_snapshot();

    (snapshot, public_key, account, resources, components)
}

pub fn get_setup() -> ExchangeInterface {
    let setup = SETUP.lock().unwrap();
    let snapshot = setup.0.clone();
    let public_key = setup.1.clone();
    let account = setup.2.clone();
    let resources = setup.3.clone();
    let components = setup.4.clone();
    
    let ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build_from_snapshot(snapshot);
    let exchange = ExchangeInterface::new(public_key, account, resources, components, ledger);

    exchange
}