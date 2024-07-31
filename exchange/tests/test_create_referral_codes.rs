#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_create_referral_codes_normal() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => (vec![(base_resource, base_referral_0)], 1u64),
    );
    let base_balance_0 = interface.test_account_balance(base_resource);
    interface.create_referral_codes((referral_resource, referral_id), vec![(base_resource, base_input_0)], referral_hashes).expect_commit_success();
    
    let base_balance_1 = interface.test_account_balance(base_resource);
    let base_output_1 = base_balance_0 - base_balance_1;

    assert_eq!(base_output_1, base_input_0 - base_referral_0);
}

#[test]
fn test_create_referral_codes_invalid_resource() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;

    let (fake_referral_resource, referral_id) = interface.mint_test_nft();

    let base_input_0 = dec!(10);
    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => (vec![(base_resource, dec!(5))], 1u64),
    );
    interface.create_referral_codes((fake_referral_resource, referral_id), vec![(base_resource, base_input_0)], referral_hashes)
        .expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_REFERRAL));
}

#[test]
fn test_create_referral_codes_exceed_max_count() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => (vec![(base_resource, base_referral_0)], 2u64),
    );
    interface.create_referral_codes((referral_resource, referral_id), vec![(base_resource, base_input_0)], referral_hashes)
        .expect_specific_failure(|err| check_error_msg(err, ERROR_REFERRAL_LIMIT_REACHED));
}

#[test]
fn test_create_referral_codes_insufficient_token() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let base_referral_0 = dec!(10);
    let base_input_0 = dec!(5);
    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => (vec![(base_resource, base_referral_0)], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id), vec![(base_resource, base_input_0)], referral_hashes)
        .expect_specific_failure(|err| check_error_msg(err, ERROR_INSUFFICIENT_TOKEN));
}

#[test]
fn test_create_referral_codes_code_already_exists() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 2).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => (vec![(base_resource, base_referral_0)], 1u64),
    );
    interface.create_referral_codes((referral_resource, referral_id.clone()), vec![(base_resource, base_input_0)], referral_hashes.clone()).expect_commit_success();

    let base_input_1 = base_input_0;
    interface.create_referral_codes((referral_resource, referral_id), vec![(base_resource, base_input_1)], referral_hashes)
        .expect_specific_failure(|err| check_error_msg(err, ERROR_REFERRAL_CODE_ALREADY_EXISTS));
}
