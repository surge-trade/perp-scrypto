#![allow(dead_code)]

#[path = "tests_common/mod.rs"]
mod tests_common;
use tests_common::*;

#[test]
fn test_collect_referral_rewards_normal() {
    let mut interface = get_setup();
    let referral_resource = interface.resources.referral_resource;

    let result = interface.mint_referral(dec!(0.05), dec!(0.05), 1).expect_commit_success().clone();
    let referral_id = parse_added_nft_ids(&result, referral_resource).first().unwrap().clone();

    interface.collect_referral_rewards((referral_resource, referral_id)).expect_commit_success();
}

#[test]
fn test_collect_referral_rewards_invalid_resource() {
    let mut interface = get_setup();

    let (fake_referral_resource, referral_id) = interface.mint_test_nft();

    interface.collect_referral_rewards((fake_referral_resource, referral_id))
        .expect_specific_failure(|err| check_error_msg(err, ERROR_INVALID_REFERRAL));
}

// TODO: not enough tokens in pool