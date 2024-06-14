use scrypto::prelude::*;

// Include the generated constants
include!(concat!(env!("OUT_DIR"), "/env_constants.rs"));

pub type PairId = String;

#[derive(ScryptoSbor, NonFungibleData, Clone, ManifestSbor)]
pub struct ReferralData {
    #[mutable] pub fee_referral: Decimal,
    #[mutable] pub fee_rebate: Decimal,
    #[mutable] pub referrals: u64,
    #[mutable] pub max_referrals: u64,
    #[mutable] pub balance: Decimal,
    #[mutable] pub total_rewarded: Decimal,
}

pub const TO_INFINITY: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity);
pub const TO_ZERO: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToZero);
