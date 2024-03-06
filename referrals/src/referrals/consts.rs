use scrypto::prelude::*;

// TODO: set this
pub const BASE_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic([
    93, 166, 99, 24, 198, 49, 140, 97, 245, 166, 27, 76, 99, 24, 198, 49, 140, 247, 148, 170, 141,
    41, 95, 20, 230, 49, 140, 99, 24, 198,
]);

pub const AUTHORITY_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic([
    93, 166, 99, 24, 198, 49, 140, 97, 245, 166, 27, 76, 99, 24, 198, 49, 140, 247, 148, 170, 141,
    41, 95, 20, 230, 49, 140, 99, 24, 198,
]);

pub const TO_INFINITY: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity);
pub const TO_ZERO: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToZero);
