use scrypto::prelude::*;

// Include the generated constants
include!(concat!(env!("OUT_DIR"), "/env_constants.rs"));

pub type PairId = String;

pub const TO_INFINITY: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity);
pub const TO_ZERO: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToZero);
