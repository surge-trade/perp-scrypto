// use radix_engine_interface::prelude::*;
// use scrypto_test::prelude::*;
// use scrypto_unit::*;
// use common::PairId;

// #[derive(ScryptoSbor, Debug)]
// struct Price {
//     pair: String,
//     quote: Decimal,
//     timestamp: i64,
// }

// #[test]
// fn test_example() {
//     let mut test_runner = TestRunnerBuilder::new().without_trace().build();

//     // Create accounts
//     let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();
//     let oracle_package = test_runner.compile_and_publish(".");

//     let oracle_private_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
//     let oracle_public_key = oracle_private_key.public_key();

//     let manifest = ManifestBuilder::new()
//         .call_function(
//             oracle_package,
//             "Oracle",
//             "new",
//             manifest_args!(OwnerRole::None, oracle_public_key))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let oracle_component = receipt
//         .expect_commit_success()
//         .new_component_addresses()[0];

//     let price = Price {
//         pair: "BTC/USD".to_string(),
//         quote: dec!(64449.495),
//         timestamp: 1714099255,
//     };
//     let prices = vec![price];
//     let data = scrypto_encode(&prices).unwrap();

//     let manifest = ManifestBuilder::new()
//         .call_method(
//             oracle_component,
//             "hash",
//             manifest_args!(data.clone()))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let hash: Vec<u8> = receipt.expect_commit_success().output(1);

//     let signature = oracle_private_key.sign_v1(hash.as_slice());

//     let manifest = ManifestBuilder::new()
//         .call_method(
//             oracle_component,
//             "push_prices",
//             manifest_args!(vec!["BTC/USD"], 0i64, data, signature))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let prices: HashMap<PairId, Decimal> = receipt.expect_commit_success().output(1);
//     println!("{:?}", prices);
// }

// #[test]
// fn test_api() {
//     let mut test_runner = TestRunnerBuilder::new().build();

//     let oracle_public_key = "b9dca0b122bc34356550c32beb31c726f993fcf1fb16aecdbe95b5181e8505b98c5f1286969664d69c4358dc16261640";
//     let signature = "aa9ff53b151b210f42c4b681b14e1d0081c045c69117d621ff07ebf6f8d8ba116dd553638fa66f4879c165d0d53154e30454d6120da74ae2c16ef186a9dcc1d1ec6d1abc0e00b3b7d2c5fe8637b585b45c38711398dbdf10c88b3d4238d9efc6";
//     let _hash = "189e2cddc616bd9f0c8529aa23343cd34a796a6dc86608094227e7b504bed881";
//     let data = "5c202101030c074254432f555344a000a46cb5d1238a732b0c0000000000000000000000000000057a1e326600000000";

//     // Create accounts
//     let (public_key, _private_key, _account_component) = test_runner.new_allocated_account();
//     let oracle_package = test_runner.compile_and_publish(".");

//     let oracle_public_key = Bls12381G1PublicKey::try_from(hex::decode(oracle_public_key).unwrap().as_slice()).unwrap();

//     let manifest = ManifestBuilder::new()
//         .call_function(
//             oracle_package,
//             "Oracle",
//             "new",
//             manifest_args!(OwnerRole::None, oracle_public_key))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let oracle_component = receipt
//         .expect_commit_success()
//         .new_component_addresses()[0];

//     let data = hex::decode(data).unwrap();

//     let manifest = ManifestBuilder::new()
//         .call_method(
//             oracle_component,
//             "hash",
//             manifest_args!(data.clone()))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let hash: Vec<u8> = receipt.expect_commit_success().output(1);
//     assert!(hex::encode(hash) == _hash);

//     let signature = Bls12381G2Signature::try_from(hex::decode(signature).unwrap().as_slice()).unwrap();

//     let manifest = ManifestBuilder::new()
//         .call_method(
//             oracle_component,
//             "push_prices",
//             manifest_args!(vec!["BTC/USD"], 0i64, data, signature))
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     let prices: HashMap<PairId, Decimal> = receipt.expect_commit_success().output(1);
//     println!("{:?}", prices);
// }

// // ------------------------------------------------------Fee Summary-------------------------------------------------------
// // Execution Cost Units Consumed           :                   6050691
// // Finalization Cost Units Consumed        :                    305050
// // Execution Cost in XRD                   :                0.30253455
// // Finalization Cost in XRD                :                 0.0152525
// // Tipping Cost in XRD                     :                         0
// // Storage Cost in XRD                     :             0.03328323307
// // Royalty Costs in XRD                    :                         0

// // ------------------------------------------------------Fee Summary-------------------------------------------------------
// // Execution Cost Units Consumed           :                   6111839
// // Finalization Cost Units Consumed        :                    305061
// // Execution Cost in XRD                   :                0.30559195
// // Finalization Cost in XRD                :                0.01525305
// // Tipping Cost in XRD                     :                         0
// // Storage Cost in XRD                     :             0.03738403256
// // Royalty Costs in XRD                    :                         0
