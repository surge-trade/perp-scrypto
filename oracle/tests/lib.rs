use hex::decode;
use radix_engine_interface::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use common::PairId;

#[test]
fn test_runner() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    // Create accounts
    let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();
    let oracle_package = test_runner.compile_and_publish(".");

    let oracle_private_key_str = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let oracle_public_key = oracle_private_key_str.public_key();

    // let oracle_public_key_str = "89f446cb59ed1f2bc0e2e3571f2852b9c5eb39ca57c48548950aff433da56b442fac6d73cd4ffd1812d0cc4fb184a680";
    // let oracle_public_key_bytes = decode(oracle_public_key_str).expect("Decoding failed");
    // let oracle_public_key = Bls12381G1PublicKey::try_from(oracle_public_key_bytes.as_slice()).unwrap();

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

    let json_data = r#"[{"pair":"BTC/USD","quote":69084.59672442}]"#;
    let data: Vec<u8> = serde_json::to_vec(&json_data).expect("Failed to convert to JSON object");
    // let data = json_data.as_bytes().to_vec();

    let signature_str = "b70f9259cbc5d2960f264cfac91d21063c5bedb8da979773f56532ab7da16bd6c6d292ef3adfffe5229df43f10f7389411090c4c65cf99caf36cf2f493ebfb2b67734b3a6618bffac56854ac1362057b98260b2736a5242356a309e3d3dd32e6";
    let signature_bytes = decode(signature_str).expect("Decoding failed");
    let signature = Bls12381G2Signature::try_from(signature_bytes.as_slice()).unwrap();

    let manifest = ManifestBuilder::new()
        .call_method(
            oracle_component,
            "push_prices",
            manifest_args!(data, signature))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt.expect_commit_success().output::<PairId>(1));
}
