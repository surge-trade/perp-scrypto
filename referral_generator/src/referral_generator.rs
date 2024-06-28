mod errors;
mod structs;

use scrypto::prelude::*;
use common::{Vaults, _AUTHORITY_RESOURCE, TO_ZERO};
use self::errors::*;
pub use self::structs::*;

#[blueprint]
#[types(
    ResourceAddress,
    Vault,
    Hash,
    ReferralCode,
)]
mod referral_generator_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_referral_code => PUBLIC;

            create_referral_codes => restrict_to: [authority];
            claim_referral_code => restrict_to: [authority];
        }
    );

    pub struct ReferralGenerator {
        vaults: Vaults,
        referral_codes: KeyValueStore<Hash, ReferralCode>,
    }

    impl ReferralGenerator {
        pub fn new(owner_role: OwnerRole) -> Global<ReferralGenerator> {
            Self {
                vaults: Vaults::new(ReferralGeneratorKeyValueStore::new_with_registered_type),
                referral_codes: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_referral_code(&self, hash: Hash) -> Option<ReferralCode> {
            self.referral_codes.get(&hash).map(|entry| entry.clone())
        }
        
        pub fn create_referral_codes(&mut self, tokens: Vec<Bucket>, referral_id: NonFungibleLocalId, referrals: Vec<(Hash, Vec<(ResourceAddress, Decimal)>, u64)>) {
            let mut amounts: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for bucket in tokens.iter() {
                let amount = amounts.entry(bucket.resource_address()).or_insert(Decimal::zero());
                *amount += bucket.amount();
            }

            let mut total_claims: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for (_, claims, count) in referrals.iter() {
                for &(resource_address, amount) in claims {
                    let total_claim = total_claims.entry(resource_address).or_insert(Decimal::zero());
                    *total_claim += amount * Decimal::from(*count);
                }
            }

            for (resource_address, &total_claim) in total_claims.iter() {
                let amount = amounts.get(resource_address).copied().unwrap_or(Decimal::zero());
                assert!(
                    total_claim <= amount, 
                    "{}", ERROR_INSUFFICIENT_TOKENS
                );
            }

            for (hash, claims, count) in referrals {
                assert!(
                    self.referral_codes.get(&hash).is_none(),
                    "{}", ERROR_REFERRAL_CODE_ALREADY_EXISTS
                );
                self.referral_codes.insert(hash, ReferralCode {
                    referral_id: referral_id.clone(),
                    claims,
                    count: 0,
                    max_count: count,
                });
            }

            self.vaults.put_batch(tokens);
        }

        pub fn claim_referral_code(&mut self, hash: Hash) -> (NonFungibleLocalId, Vec<Bucket>) {
            let mut referral_code = self.referral_codes.get_mut(&hash).expect(ERROR_REFERRAL_CODE_NOT_FOUND);

            assert!(
                referral_code.count < referral_code.max_count,
                "{}", ERROR_REFERRAL_CODE_ALREADY_CLAIMED
            );

            referral_code.count += 1;
            let tokens = self.vaults.take_advanced_batch(referral_code.claims.clone(), TO_ZERO);

            (referral_code.referral_id.clone(), tokens)
        }
    }
}
