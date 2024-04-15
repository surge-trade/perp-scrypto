use scrypto::prelude::*;
use common::_AUTHORITY_RESOURCE;

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
mod fee_distributor {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_referrer => PUBLIC;
            get_rebate => PUBLIC;
            get_trickle_up => PUBLIC;
            get_virtual_balance => PUBLIC;

            update_rebate => restrict_to: [authority];
            update_trickle_up => restrict_to: [authority];
            update_virtual_balance => restrict_to: [authority];

            set_referrer => restrict_to: [authority];
            reward => restrict_to: [authority];
            collect => restrict_to: [authority];
        }
    );

    struct FeeDistributor {
        referral_accounts: KeyValueStore<ComponentAddress, ReferralAccount>,
        virtual_balance: Decimal,
        rebate: Decimal,
        trickle_up: Decimal,
    }

    impl FeeDistributor {
        pub fn new(owner_role: OwnerRole) -> Global<FeeDistributor> {
            Self {
                referral_accounts: KeyValueStore::new_with_registered_type(),
                virtual_balance: dec!(0),
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

        pub fn get_rebate(&self) -> Decimal {
            self.rebate
        }

        pub fn get_trickle_up(&self) -> Decimal {
            self.trickle_up
        }

        pub fn get_virtual_balance(&self) -> Decimal {
            self.virtual_balance
        }

        pub fn update_rebate(&mut self, rebate: Decimal) {
            assert!(rebate >= dec!(0));
            self.rebate = rebate;
        }

        pub fn update_trickle_up(&mut self, trickle_up: Decimal) {
            assert!(trickle_up >= dec!(0));
            self.trickle_up = trickle_up;
        }

        pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
            assert!(virtual_balance >= dec!(0));
            self.virtual_balance = virtual_balance;
        }

        pub fn set_referrer(&mut self, account: ComponentAddress, referrer: Option<ComponentAddress>) {
            self.referral_accounts.get_mut(&account)
                .map(|mut referral| referral.referrer_account = referrer)
                .unwrap_or_else(|| {
                    self.referral_accounts.insert(account, ReferralAccount::new(referrer));
                });
        }

        pub fn reward(&mut self, amount_protocol: Decimal, mut amount_referral: Decimal, referred_account: ComponentAddress) {
            assert!(amount_protocol >= dec!(0));
            assert!(amount_referral >= dec!(0));

            self.virtual_balance += amount_protocol;

            let maybe_referrer = self.referral_accounts.get_mut(&referred_account)
                .and_then(|mut referral| {
                    referral.referrer_account.map(|referrer_account| {
                        let rebate = amount_referral * self.rebate;
                        amount_referral -= rebate;
                        referral.rewards += rebate;
                        referrer_account
                    })
                })
                .and_then(|referrer_account| {
                    self.referral_accounts.get_mut(&referrer_account)
                });

            match maybe_referrer {
                Some(mut referrer) => referrer.rewards += amount_referral,
                None => self.virtual_balance += amount_referral,
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
                None => self.virtual_balance += trickle_up,
            }

            amount
        }
    }
}