use radix_engine_interface::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;

#[test]
fn test_runner() {
    let oracle_public_key = "89f446cb59ed1f2bc0e2e3571f2852b9c5eb39ca57c48548950aff433da56b442fac6d73cd4ffd1812d0cc4fb184a680";
    let signature = "a7d57391290b1a72e82f2a56102c0e5c5ffc24bdfa595a40b5e48e49f4a20533e98b7017386dff79d98ca364b284752a003fc1ac5d12cf2d987ce8683a0f270e27ece87fc09425be83a39a0ccba0716bb052d4bba1b3981e9170d55f57fa6812";
    let hash = "b45c9f4654a2a2d261eb3fc19c7a8f2ccc1434ab78af3109331b53aa3c2a2330";
    let prices = r#"[{"pair":"BTC/USD","quote":70875.103068}]"#;

    let oracle_public_key = Bls12381G1PublicKey::try_from(hex::decode(oracle_public_key).unwrap().as_slice()).unwrap();
    let signature = Bls12381G2Signature::try_from(hex::decode(signature).unwrap().as_slice()).unwrap();

    let hash = hex::decode(hash).unwrap();
    let data = prices.as_bytes().to_vec();

    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    // Create accounts
    let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();
    let oracle_package = test_runner.compile_and_publish(".");

    let manifest = ManifestBuilder::new()
        .call_function(
            oracle_package,
            "Oracle",
            "new",
            manifest_args!(OwnerRole::None, oracle_public_key))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let oracle_component = receipt
        .expect_commit_success()
        .new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .call_method(
            oracle_component,
            "hash",
            manifest_args!(data))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    
    println!("{:?}", receipt.expect_commit_success().output::<Vec<u8>>(1));
    println!("{:?}", hash);
}
