use scrypto::prelude::*;
use utils::{_AUTHORITY_RESOURCE};

#[derive(ScryptoSbor)]
struct ReferralAccount {
    rewards: Decimal,
    referrer_account: Option<ComponentAddress>,
}

impl ReferralAccount {
    fn new(referrer_account: Option<ComponentAddress>) -> Self {
        Self {
            rewards: dec!(0),
            referrer_account,
        }
    }
}

#[blueprint]
#[types(
    ComponentAddress,
    ReferralAccount,
)]
mod referrals {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_referrer => PUBLIC;
            
            update_rebate => restrict_to: [authority];
            update_trickle_up => restrict_to: [authority];
            
            set_referrer => restrict_to: [authority];
            reward => restrict_to: [authority];
            collect => restrict_to: [authority];
        }
    );

    struct Referrals {
        referral_accounts: KeyValueStore<ComponentAddress, ReferralAccount>,
        undirected_rewards: Decimal,
        rebate: Decimal,
        trickle_up: Decimal,
    }

    impl Referrals {
        pub fn new(owner_role: OwnerRole) -> Global<Referrals> {
            Self {
                referral_accounts: KeyValueStore::new_with_registered_type(),
                undirected_rewards: dec!(0),
                rebate: dec!(0),
                trickle_up: dec!(0),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_referrer(&self, account: ComponentAddress) -> Option<ComponentAddress> {
            self.referral_accounts.get(&account)
                .map(|referral| referral.referrer_account)
                .flatten()
        }

        pub fn update_rebate(&mut self, rebate: Decimal) {
            assert!(rebate >= dec!(0));
            self.rebate = rebate;
        }

        pub fn update_trickle_up(&mut self, trickle_up: Decimal) {
            assert!(trickle_up >= dec!(0));
            self.trickle_up = trickle_up;
        }

        pub fn set_referrer(&mut self, account: ComponentAddress, referrer: Option<ComponentAddress>) {
            self.referral_accounts.get_mut(&account)
                .map(|mut referral| referral.referrer_account = referrer)
                .unwrap_or_else(|| {
                    self.referral_accounts.insert(account, ReferralAccount::new(referrer));
                });
        }

        pub fn reward(&mut self, referred_account: ComponentAddress, mut amount: Decimal) {
            assert!(amount >= dec!(0));

            let maybe_referrer = self.referral_accounts.get_mut(&referred_account)
                .and_then(|mut referral| {
                    referral.referrer_account.map(|referrer_account| {
                        let rebate = amount * self.rebate;
                        amount -= rebate;
                        referral.rewards += rebate;
                        referrer_account
                    })
                })
                .and_then(|referrer_account| {
                    self.referral_accounts.get_mut(&referrer_account)
                });

            match maybe_referrer {
                Some(mut referrer) => referrer.rewards += amount,
                None => self.undirected_rewards += amount,
            }
        }

        pub fn collect(&mut self, account: ComponentAddress) -> Decimal {
            let (mut amount, maybe_referrer_account) = self.referral_accounts.get_mut(&account)
                .map(|mut referral| {
                    let amount = referral.rewards;
                    referral.rewards = dec!(0);
                    (amount, referral.referrer_account)
                })
                .unwrap_or((dec!(0), None));

            let trickle_up = amount * self.trickle_up;
            amount -= trickle_up;
            match maybe_referrer_account.and_then(|referrer_account| self.referral_accounts.get_mut(&referrer_account)) {
                Some(mut referrer) => referrer.rewards += trickle_up,
                None => self.undirected_rewards += trickle_up,
            }

            amount
        }
    }
}