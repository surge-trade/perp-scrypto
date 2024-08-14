#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_create_referral_codes_from_allocation_normal() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let result = interface.mint_referral_with_allocation(
        dec!(0.05), 
        dec!(0.05), 
        1,
        vec![(base_resource, base_input_0)],
        vec![(base_resource, base_referral_0)],
        1
    ).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 1u64,
    );
    interface.create_referral_codes_from_allocation(
        (referral_resource, referral_id), 
        0, 
        referral_hashes
    ).expect_commit_success();
}

#[test]
fn test_create_referral_codes_from_allocation_invalid_resource() {
    let mut interface = get_setup();

    let (fake_referral_resource, referral_id) = interface.mint_test_nft();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 1u64,
    );
    interface.create_referral_codes_from_allocation(
        (fake_referral_resource, referral_id), 
        0, 
        referral_hashes
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_REFERRAL));
}

#[test]
fn test_create_referral_codes_from_allocation_not_found_0() {
    let mut interface = get_setup();
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 1u64,
    );
    interface.create_referral_codes_from_allocation(
        (referral_resource, referral_id), 
        0, 
        referral_hashes
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ALLOCATION_NOT_FOUND));
}

#[test]
fn test_create_referral_codes_from_allocation_not_found_1() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let result = interface.mint_referral_with_allocation(
        dec!(0.05), 
        dec!(0.05), 
        1,
        vec![(base_resource, base_input_0)],
        vec![(base_resource, base_referral_0)],
        1
    ).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 1u64,
    );
    interface.create_referral_codes_from_allocation(
        (referral_resource, referral_id), 
        1, 
        referral_hashes
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_ALLOCATION_NOT_FOUND));
}

#[test]
fn test_create_referral_codes_from_allocation_exceed_max_count() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let result = interface.mint_referral_with_allocation(
        dec!(0.05), 
        dec!(0.05), 
        1,
        vec![(base_resource, base_input_0)],
        vec![(base_resource, base_referral_0)],
        1
    ).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 2u64,
    );
    interface.create_referral_codes_from_allocation((referral_resource, referral_id), 0, referral_hashes)
        .expect_specific_failure(|err| check_error_msg(err, ERROR_ALLOCATION_LIMIT_REACHED));
}

#[test]
fn test_create_referral_codes_from_allocation_code_already_exists() {
    let mut interface = get_setup();
    let base_resource = interface.resources.base_resource;
    let referral_resource = interface.resources.referral_resource;

    let base_referral_0 = dec!(5);
    let base_input_0 = dec!(10);
    let result = interface.mint_referral_with_allocation(
        dec!(0.05), 
        dec!(0.05), 
        2,
        vec![(base_resource, base_input_0)],
        vec![(base_resource, base_referral_0)],
        2
    ).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    let referral_hashes = hashmap!(
        Hash([0; Hash::LENGTH]) => 1u64,
    );
    interface.create_referral_codes_from_allocation(
        (referral_resource, referral_id.clone()), 
        0, 
        referral_hashes.clone()
    ).expect_commit_success();

    interface.create_referral_codes_from_allocation(
        (referral_resource, referral_id), 
        0, 
        referral_hashes
    ).expect_specific_failure(|err| check_error_msg(err, ERROR_REFERRAL_CODE_ALREADY_EXISTS));
}
