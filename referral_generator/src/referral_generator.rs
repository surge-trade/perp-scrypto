pub mod errors;
pub mod structs;

use scrypto::prelude::*;
use common::{ListIndex, Vaults, _AUTHORITY_RESOURCE, TO_ZERO, TO_INFINITY};
pub use self::errors::*;
pub use self::structs::*;

#[blueprint]
#[types(
    ResourceAddress,
    Vault,
    Hash,
    NonFungibleLocalId,
    Vec<ReferralAllocation>,
    ReferralCode,
)]
mod referral_generator_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_allocations => PUBLIC;
            get_referral_code => PUBLIC;

            add_allocation => restrict_to: [authority];
            create_referral_codes => restrict_to: [authority];
            create_referral_codes_from_allocation => restrict_to: [authority];
            claim_referral_code => restrict_to: [authority];
        }
    );

    pub struct ReferralGenerator {
        vaults: Vaults,
        referral_allocations: KeyValueStore<NonFungibleLocalId, Vec<ReferralAllocation>>,
        referral_codes: KeyValueStore<Hash, ReferralCode>,
    }

    impl ReferralGenerator {
        pub fn new(owner_role: OwnerRole) -> Global<ReferralGenerator> {
            Self {
                vaults: Vaults::new(ReferralGeneratorKeyValueStore::new_with_registered_type),
                referral_allocations: KeyValueStore::new_with_registered_type(),
                referral_codes: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_allocations(&self, referral_id: NonFungibleLocalId) -> Vec<ReferralAllocation> {
            if let Some(referral_allocation_list) = self.referral_allocations.get(&referral_id) {
                referral_allocation_list.clone()
            } else {
                Vec::new()
            }
        }

        pub fn get_referral_code(&self, hash: Hash) -> Option<ReferralCode> {
            self.referral_codes.get(&hash).map(|entry| entry.clone())
        }

        pub fn add_allocation(
            &mut self, 
            tokens: Vec<Bucket>, 
            referral_id: NonFungibleLocalId, 
            claims: Vec<(ResourceAddress, Decimal)>, 
            count: u64
        ) -> (Vec<Bucket>, ListIndex) {
            let mut mapped_tokens: HashMap<ResourceAddress, Bucket> = HashMap::new();
            for token in tokens.into_iter() {
                let resource = token.resource_address();
                if let Some(mapped_token) = mapped_tokens.get_mut(&resource) {
                    mapped_token.put(token);
                } else {
                    mapped_tokens.insert(resource, token);
                }
            }

            let mut total_claims: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for &(resource, amount) in claims.iter() {
                assert!(
                    amount.is_positive(),
                    "{}, VALUE:{}, REQUIRED:0, OP:> |", ERROR_INVALID_CLAIM_AMOUNT, amount,
                );
                let total_claim = total_claims.entry(resource).or_insert(Decimal::zero());
                *total_claim += amount * Decimal::from(count);
            }

            let mut referral_tokens: Vec<Bucket> = vec![];
            let mut remainder_tokens: Vec<Bucket> = vec![];
            for (&resource, &total_claim) in total_claims.iter() {
                if let Some(token) = mapped_tokens.get_mut(&resource) {
                    assert!(
                        total_claim <= token.amount(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_INSUFFICIENT_TOKEN, total_claim, token.amount()
                    );
                    let referral_token = token.take_advanced(total_claim, TO_INFINITY);
                    referral_tokens.push(referral_token);
                } else {
                    assert!(
                        total_claim.is_zero(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INSUFFICIENT_TOKEN, total_claim, dec!(0)
                    );
                }
            }
            remainder_tokens.extend(mapped_tokens.into_iter().map(|(_, token)| token));

            if self.referral_allocations.get(&referral_id).is_none() {
                self.referral_allocations.insert(referral_id.clone(), Vec::new());
            }
            let mut referral_allocation_list = self.referral_allocations.get_mut(&referral_id).unwrap();

            let index = referral_allocation_list.len() as ListIndex;
            referral_allocation_list.push(ReferralAllocation {
                claims,
                count: 0,
                max_count: count,
            });

            self.vaults.put_batch(referral_tokens);
            
            (remainder_tokens, index)
        }
        
        pub fn create_referral_codes(
            &mut self, 
            tokens: Vec<Bucket>, 
            referral_id: NonFungibleLocalId, 
            referral_hashes: HashMap<Hash, (Vec<(ResourceAddress, Decimal)>, u64)>
        ) -> Vec<Bucket> {
            let mut mapped_tokens: HashMap<ResourceAddress, Bucket> = HashMap::new();
            for token in tokens.into_iter() {
                let resource = token.resource_address();
                if let Some(mapped_token) = mapped_tokens.get_mut(&resource) {
                    mapped_token.put(token);
                } else {
                    mapped_tokens.insert(resource, token);
                }
            }

            let mut total_claims: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for (_, (claims, count)) in referral_hashes.iter() {
                if *count == 1 {
                    for &(resource, amount) in claims {
                        assert!(
                            amount.is_positive(),
                            "{}, VALUE:{}, REQUIRED:0, OP:> |", ERROR_INVALID_CLAIM_AMOUNT, amount,
                        );
                        let total_claim = total_claims.entry(resource).or_insert(Decimal::zero());
                        *total_claim += amount;
                    }
                } else {
                    assert!(
                        claims.len() == 0,
                        "{}, VALUE:{}, REQUIRED:0, OP:== |", ERROR_REFERRAL_MULTIPLE_USE_CODE_WITH_CLAIMS, claims.len()
                    );
                }
            }

            let mut referral_tokens: Vec<Bucket> = vec![];
            let mut remainder_tokens: Vec<Bucket> = vec![];
            for (&resource, &total_claim) in total_claims.iter() {
                if let Some(token) = mapped_tokens.get_mut(&resource) {
                    assert!(
                        total_claim <= token.amount(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_INSUFFICIENT_TOKEN, total_claim, token.amount()
                    );
                    let referral_token = token.take_advanced(total_claim, TO_INFINITY);
                    referral_tokens.push(referral_token);
                } else {
                    assert!(
                        total_claim.is_zero(),
                        "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INSUFFICIENT_TOKEN, total_claim, dec!(0)
                    );
                }
            }
            remainder_tokens.extend(mapped_tokens.into_iter().map(|(_, token)| token));

            for (hash, (claims, count)) in referral_hashes.into_iter() {
                assert!(
                    self.referral_codes.get(&hash).is_none(),
                    "{}, VALUE:{:?}, REQUIRED:None, OP:== |", ERROR_REFERRAL_CODE_ALREADY_EXISTS, Some(hash)
                );
                self.referral_codes.insert(hash, ReferralCode {
                    referral_id: referral_id.clone(),
                    claims,
                    count: 0,
                    max_count: count,
                });
            }

            self.vaults.put_batch(referral_tokens);

            remainder_tokens
        }

        pub fn create_referral_codes_from_allocation(
            &mut self, 
            referral_id: NonFungibleLocalId, 
            allocation_index: ListIndex, 
            referral_hashes: HashSet<Hash>
        ) {
            let mut referral_allocation_list = self.referral_allocations.get_mut(&referral_id).expect(ERROR_ALLOCATION_NOT_FOUND);
            let referral_allocation = referral_allocation_list.get_mut(allocation_index as usize).expect(ERROR_ALLOCATION_NOT_FOUND);
                
            let total_count: u64 = referral_hashes.len() as u64;
            referral_allocation.count += total_count;

            assert!(
                referral_allocation.count <= referral_allocation.max_count,
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", ERROR_ALLOCATION_LIMIT_REACHED, referral_allocation.count, referral_allocation.max_count
            );

            let claims = &referral_allocation.claims;
            for hash in referral_hashes.into_iter() {
                assert!(
                    self.referral_codes.get(&hash).is_none(),
                    "{}, VALUE:{:?}, REQUIRED:None, OP:== |", ERROR_REFERRAL_CODE_ALREADY_EXISTS, Some(hash)
                );
                self.referral_codes.insert(hash, ReferralCode {
                    referral_id: referral_id.clone(),
                    claims: claims.clone(),
                    count: 0,
                    max_count: 1,
                });
            }
        }

        pub fn claim_referral_code(&mut self, hash: Hash) -> (NonFungibleLocalId, Vec<Bucket>) {
            let mut referral_code = self.referral_codes.get_mut(&hash).expect(ERROR_REFERRAL_CODE_NOT_FOUND);

            assert!(
                referral_code.count < referral_code.max_count,
                "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_REFERRAL_CODE_ALREADY_CLAIMED, referral_code.count, referral_code.max_count
            );

            referral_code.count += 1;
            let tokens = self.vaults.take_advanced_batch(referral_code.claims.clone(), TO_ZERO);

            (referral_code.referral_id.clone(), tokens)
        }
    }
}
