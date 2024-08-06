use scrypto_test::prelude::*;
use super::Resources;
use ::common::*;

use std::path::Path;

fn check_compile(
    package_path: &str,
    package_name: &str,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
) -> (Vec<u8>, PackageDefinition) {
    let tests_compiled_dir = Path::new("tests").join("compiled");
    let wasm_path = tests_compiled_dir.join(format!("{}.wasm", package_name));
    let rpd_path = tests_compiled_dir.join(format!("{}.rpd", package_name));

    if wasm_path.exists() && rpd_path.exists() {
        let code = std::fs::read(&wasm_path).expect("Failed to read WASM file");
        let definition: PackageDefinition = manifest_decode(&std::fs::read(&rpd_path).expect("Failed to read RPD file"))
            .expect("Failed to decode RPD file");
        return (code, definition);
    } else {
        let (code, definition) = Compile::compile_with_env_vars(
            package_path, 
            envs.clone(), 
            CompileProfile::Standard, 
            use_coverage
        );

        let compiled_path = Path::new(package_path).join("target").join("wasm32-unknown-unknown").join("release");
        let wasm_path = compiled_path.join(format!("{}.wasm", package_name));
        let rpd_path = compiled_path.join(format!("{}.rpd", package_name));
        
        std::fs::create_dir_all(&tests_compiled_dir).expect("Failed to create tests/compiled directory");
        std::fs::copy(&wasm_path, tests_compiled_dir.join(format!("{}.wasm", package_name)))
            .expect("Failed to copy WASM file to tests/compiled");
        std::fs::copy(&rpd_path, tests_compiled_dir.join(format!("{}.rpd", package_name)))
            .expect("Failed to copy RPD file to tests/compiled");

        return (code, definition);
    }
}

#[derive(Clone)]
pub struct Components {
    pub token_wrapper_package: PackageAddress,
    pub token_wrapper_component: ComponentAddress,
    pub oracle_key_seed: u64,
    pub oracle_package: PackageAddress,
    pub oracle_component: ComponentAddress,
    pub config_package: PackageAddress,
    pub config_component: ComponentAddress,
    pub account_package: PackageAddress,
    pub pool_package: PackageAddress,
    pub pool_component: ComponentAddress,
    pub referral_generator_package: PackageAddress,
    pub referral_generator_component: ComponentAddress,
    pub fee_distributor_package: PackageAddress,
    pub fee_distributor_component: ComponentAddress,
    pub fee_delegator_package: PackageAddress,
    pub fee_delegator_component: ComponentAddress,
    pub fee_oath_resource: ResourceAddress,
    pub permission_registry_package: PackageAddress,
    pub permission_registry_component: ComponentAddress,
    pub exchange_package: PackageAddress,
    pub exchange_component: ComponentAddress,
}

pub fn create_components(
    account: ComponentAddress,
    public_key: Secp256k1PublicKey,
    resources: &Resources,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> Components {
    let use_coverage = true;
    let encoder = &AddressBech32Encoder::for_simulator();

    let envs = &mut btreemap! {
        "RUSTFLAGS".to_owned() => "".to_owned(),
        "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
        "OWNER_RESOURCE".to_owned() => resources.owner_resource.to_string(encoder),
        "AUTHORITY_RESOURCE".to_owned() => resources.authority_resource.to_string(encoder),
        "BASE_AUTHORITY_RESOURCE".to_owned() => resources.base_authority_resource.to_string(encoder),
        "BASE_RESOURCE".to_owned() => resources.base_resource.to_string(encoder),
        "LP_RESOURCE".to_owned() => resources.lp_resource.to_string(encoder),
        "REFERRAL_RESOURCE".to_owned() => resources.referral_resource.to_string(encoder),
        "PROTOCOL_RESOURCE".to_owned() => resources.protocol_resource.to_string(encoder),
        "KEEPER_REWARD_RESOURCE".to_owned() => resources.keeper_reward_resource.to_string(encoder),
    };

    let oracle_key_seed = 1;
    let oracle_key = Bls12381G1PrivateKey::from_u64(oracle_key_seed).unwrap().public_key();

    let (token_wrapper_package, token_wrapper_component) = create_token_wrapper(account, public_key, resources, envs, use_coverage, encoder, ledger);
    let (oracle_package, oracle_component) = create_oracle(oracle_key, resources, envs, use_coverage, encoder, ledger);
    let (config_package, config_component) = create_config(resources, envs, use_coverage, encoder, ledger);
    let account_package = create_account(resources, envs, use_coverage, encoder, ledger);
    let (pool_package, pool_component) = create_pool(resources, envs, use_coverage, encoder, ledger);
    let (referral_generator_package, referral_generator_component) = create_referral_generator(resources, envs, use_coverage, encoder, ledger);
    let (fee_distributor_package, fee_distributor_component) = create_fee_distributor(resources, envs, use_coverage, encoder, ledger);
    let (fee_delegator_package, fee_delegator_component, fee_oath_resource) = create_fee_delegator(resources, envs, use_coverage, encoder, ledger);
    let (permission_registry_package, permission_registry_component) = create_permission_registry(resources, envs, use_coverage, encoder, ledger);
    let (exchange_package, exchange_component) = create_exchange(account, public_key, resources, envs, use_coverage, encoder, ledger);

    Components {
        token_wrapper_package,
        token_wrapper_component,
        oracle_key_seed,
        oracle_package,
        oracle_component,
        config_package,
        config_component,
        account_package,
        pool_package,
        pool_component,
        referral_generator_package,
        referral_generator_component,
        fee_distributor_package,
        fee_distributor_component,
        fee_delegator_package,
        fee_delegator_component,
        fee_oath_resource,
        permission_registry_package,
        permission_registry_component,
        exchange_package,
        exchange_component,
    }
}

fn create_token_wrapper(
    account: ComponentAddress,
    public_key: Secp256k1PublicKey,
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let token_wrapper_package = ledger.publish_package(
        check_compile("../token_wrapper", "token_wrapper", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resources.base_authority_resource, dec!(1))
        .take_all_from_worktop(resources.base_authority_resource, "authority")
        .with_bucket("authority", |manifest, bucket| {
            manifest.call_function(token_wrapper_package, "TokenWrapper", "new", manifest_args!(resources.owner_role.clone(), bucket))
        })
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let token_wrapper_component = receipt.expect_commit_success().new_component_addresses()[0];
    envs.insert("TOKEN_WRAPPER_PACKAGE".to_owned(), token_wrapper_package.to_string(encoder));
    envs.insert("TOKEN_WRAPPER_COMPONENT".to_owned(), token_wrapper_component.to_string(encoder));

    (token_wrapper_package, token_wrapper_component)
}

fn create_oracle(
    oracle_key: Bls12381G1PublicKey,
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let oracle_package = ledger.publish_package(
        check_compile("../oracle", "oracle", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let oracle_component = ledger.call_function(
        oracle_package, 
        "Oracle", 
        "new", 
        manifest_args!(resources.owner_role.clone(), hashmap!(0 as ListIndex => oracle_key))
    ).expect_commit_success().new_component_addresses()[0];
    
    envs.insert("ORACLE_PACKAGE".to_owned(), oracle_package.to_string(encoder));
    envs.insert("ORACLE_COMPONENT".to_owned(), oracle_component.to_string(encoder));

    (oracle_package, oracle_component)
}

fn create_config(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let config_package = ledger.publish_package(
        check_compile("../config", "config", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let config_component = ledger.call_function(
        config_package, 
        "Config", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];
    
    envs.insert("CONFIG_PACKAGE".to_owned(), config_package.to_string(encoder));
    envs.insert("CONFIG_COMPONENT".to_owned(), config_component.to_string(encoder));

    (config_package, config_component)
}

fn create_account(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> PackageAddress {
    let account_package = ledger.publish_package(
        check_compile("../account", "account", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    envs.insert("ACCOUNT_PACKAGE".to_owned(), account_package.to_string(encoder));
    account_package
}

fn create_pool(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let pool_package = ledger.publish_package(
        check_compile("../pool", "pool", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let pool_component = ledger.call_function(
        pool_package, 
        "MarginPool", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("POOL_PACKAGE".to_owned(), pool_package.to_string(encoder));
    envs.insert("POOL_COMPONENT".to_owned(), pool_component.to_string(encoder));

    (pool_package, pool_component)
}

fn create_referral_generator(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let referral_generator_package = ledger.publish_package(
        check_compile("../referral_generator", "referral_generator", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let referral_generator_component = ledger.call_function(
        referral_generator_package, 
        "ReferralGenerator", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("REFERRAL_GENERATOR_PACKAGE".to_owned(), referral_generator_package.to_string(encoder));
    envs.insert("REFERRAL_GENERATOR_COMPONENT".to_owned(), referral_generator_component.to_string(encoder));

    (referral_generator_package, referral_generator_component)
}

fn create_fee_distributor(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let fee_distributor_package = ledger.publish_package(
        check_compile("../fee_distributor", "fee_distributor", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let fee_distributor_component = ledger.call_function(
        fee_distributor_package, 
        "FeeDistributor", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("FEE_DISTRIBUTOR_PACKAGE".to_owned(), fee_distributor_package.to_string(encoder));
    envs.insert("FEE_DISTRIBUTOR_COMPONENT".to_owned(), fee_distributor_component.to_string(encoder));

    (fee_distributor_package, fee_distributor_component)
}

fn create_fee_delegator(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress, ResourceAddress) {
    let fee_delegator_package = ledger.publish_package(
        check_compile("../fee_delegator", "fee_delegator", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let result = ledger.call_function(
        fee_delegator_package, 
        "FeeDelegator", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().clone();

    let fee_delegator_component = result.new_component_addresses()[0];
    let fee_oath_resource = result.new_resource_addresses()[0];

    envs.insert("FEE_DELEGATOR_PACKAGE".to_owned(), fee_delegator_package.to_string(encoder));
    envs.insert("FEE_DELEGATOR_COMPONENT".to_owned(), fee_delegator_component.to_string(encoder));

    (fee_delegator_package, fee_delegator_component, fee_oath_resource)
}

fn create_permission_registry(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let permission_registry_package = ledger.publish_package(
        check_compile("../permission_registry", "permission_registry", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let permission_registry_component = ledger.call_function(
        permission_registry_package, 
        "PermissionRegistry", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("PERMISSION_REGISTRY_PACKAGE".to_owned(), permission_registry_package.to_string(encoder));
    envs.insert("PERMISSION_REGISTRY_COMPONENT".to_owned(), permission_registry_component.to_string(encoder));

    (permission_registry_package, permission_registry_component)
}

fn _create_env_registry(
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let env_registry_package = ledger.publish_package(
        check_compile("../env_registry", "env_registry", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    let env_registry_component = ledger.call_function(
        env_registry_package, 
        "EnvRegistry", 
        "new", 
        manifest_args!(resources.owner_role.clone())
    ).expect_commit_success().new_component_addresses()[0];

    envs.insert("ENV_REGISTRY_PACKAGE".to_owned(), env_registry_package.to_string(encoder));
    envs.insert("ENV_REGISTRY_COMPONENT".to_owned(), env_registry_component.to_string(encoder));

    (env_registry_package, env_registry_component)
}

fn create_exchange(
    account: ComponentAddress,
    public_key: Secp256k1PublicKey, 
    resources: &Resources,
    envs: &mut BTreeMap<String, String>,
    use_coverage: bool,
    encoder: &AddressBech32Encoder,
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (PackageAddress, ComponentAddress) {
    let exchange_package = ledger.publish_package(
        check_compile("../exchange", "exchange", envs, use_coverage),
        BTreeMap::new(),
        resources.owner_role.clone()
    );
    
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resources.authority_resource, dec!(1))
        .take_all_from_worktop(resources.authority_resource, "authority")
        .with_bucket("authority", |manifest, bucket| {
            manifest.call_function(exchange_package, "Exchange", "new", manifest_args!(resources.owner_role.clone(), bucket, None::<Option<ManifestAddressReservation>>))
        })
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    let exchange_component = receipt.expect_commit_success().new_component_addresses()[0];
    envs.insert("EXCHANGE_PACKAGE".to_owned(), exchange_package.to_string(encoder));
    envs.insert("EXCHANGE_COMPONENT".to_owned(), exchange_component.to_string(encoder));

    (exchange_package, exchange_component)
}
