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

    let oracle_private_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let oracle_public_key = oracle_private_key.public_key();

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

    let json_data = r#"[
            {
            "pair": "BTC/USD",
            "quote": 69210.14315615
            }
        ]"#;
    // let data: Vec<u8> = serde_json::to_vec(&json_data).expect("Failed to convert to JSON object");
    let data = json_data.as_bytes().to_vec();

    let signature_str = "9276a69d4b5153564b8fa45e484b2e01bd76ca98196ad212b3082034a00dc554680398e433b45cb17297f83c624d0a780444676b68afdccfe61b94c56075e73a4be5ba4d4ac94ea6550abd9fff387ec8ebf6a2724e600db212bbf6408578f439";
    let signature_bytes = hex::decode(signature_str).expect("Decoding failed");
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
