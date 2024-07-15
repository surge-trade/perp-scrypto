use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone, ManifestSbor)]
pub struct ReferralData {
    #[mutable] pub fee_referral: Decimal,
    #[mutable] pub fee_rebate: Decimal,
    #[mutable] pub referrals: u64,
    #[mutable] pub max_referrals: u64,
    #[mutable] pub balance: Decimal,
    #[mutable] pub total_rewarded: Decimal,
}

#[derive(ScryptoSbor, Clone)]
pub struct ReferralCode {
    pub referral_id: NonFungibleLocalId,
    pub claims: Vec<(ResourceAddress, Decimal)>,
    pub count: u64,
    pub max_count: u64,
}

#[derive(ScryptoSbor, Clone)]
pub struct ReferralAllocation {
    pub claims: Vec<(ResourceAddress, Decimal)>,
    pub count: u64,
    pub max_count: u64,
}