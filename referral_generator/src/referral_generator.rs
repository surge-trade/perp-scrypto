mod errors;

use scrypto::prelude::*;
use common::{Vaults, _AUTHORITY_RESOURCE, TO_ZERO};
use self::errors::*;

#[derive(ScryptoSbor, Clone)]
pub struct Referral {
    referrer: ComponentAddress,
    claims: Vec<(ResourceAddress, Decimal)>,
    claimed: bool,
}

#[blueprint]
#[types(
    ResourceAddress,
    Vault,
    Hash,
    Referral,
)]
mod referral_generator_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_referral => PUBLIC;

            generate_referrals => restrict_to: [authority];
            claim_referral => restrict_to: [authority];
        }
    );

    pub struct ReferralGenerator {
        vaults: Vaults,
        referrals: KeyValueStore<Hash, Referral>,
    }

    impl ReferralGenerator {
        pub fn new(owner_role: OwnerRole) -> Global<ReferralGenerator> {
            Self {
                vaults: Vaults::new(ReferralGeneratorKeyValueStore::new_with_registered_type),
                referrals: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_referral(&self, hash: Hash) -> Option<Referral> {
            self.referrals.get(&hash).map(|entry| entry.clone())
        }
        
        pub fn generate_referrals(&mut self, tokens: Vec<Bucket>, referrer: ComponentAddress, referrals: Vec<(Hash, Vec<(ResourceAddress, Decimal)>)>) {
            let mut amounts: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for bucket in tokens.iter() {
                let amount = amounts.entry(bucket.resource_address()).or_insert(Decimal::zero());
                *amount += bucket.amount();
            }

            let mut total_claims: HashMap<ResourceAddress, Decimal> = HashMap::new();
            for (_, claims) in referrals.iter() {
                for &(resource_address, amount) in claims {
                    let total_claim = total_claims.entry(resource_address).or_insert(Decimal::zero());
                    *total_claim += amount;
                }
            }

            for (resource_address, &total_claim) in total_claims.iter() {
                let amount = amounts.get(resource_address).copied().unwrap_or(Decimal::zero());
                assert!(
                    total_claim <= amount, 
                    "{}", ERROR_INSUFFICIENT_TOKENS
                );
            }

            for (hash, claims) in referrals {
                assert!(
                    self.referrals.get(&hash).is_none(),
                    "{}", ERROR_ALREADY_REFERRAL_EXISTS
                );
                self.referrals.insert(hash, Referral {
                    referrer,
                    claims,
                    claimed: false,
                });
            }

            self.vaults.put_batch(tokens);
        }

        pub fn claim_referral(&mut self, hash: Hash) -> (ComponentAddress, Vec<Bucket>) {
            let mut referral = self.referrals.get_mut(&hash).expect(ERROR_REFERRAL_NOT_FOUND);

            assert!(
                !referral.claimed,
                "{}", ERROR_REFERRAL_ALREADY_CLAIMED
            );

            referral.claimed = true;
            let tokens = self.vaults.take_advanced_batch(referral.claims.clone(), TO_ZERO);

            (referral.referrer, tokens)
        }
    }
}
