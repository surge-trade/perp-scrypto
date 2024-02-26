use scrypto::prelude::*;
use crate::utils::{List, Vaults};
use super::errors::*;
use super::keeper_requests::KeeperRequest;
use super::margin_account::margin_account::MarginAccount;

#[derive(ScryptoSbor, Clone, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub cost: Decimal,
    pub funding_index: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountInfo {
    pub positions: HashMap<u64, AccountPosition>,
    pub collateral_balances: HashMap<ResourceAddress, Decimal>,
    pub virtual_balance: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountUpdates {
    pub position_updates: HashMap<u64, AccountPosition>,
    pub virtual_balance: Decimal,
}

#[blueprint]
pub mod margin_account {
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Authority {
            fn get_rule(&self) -> AccessRule;
        }
    }

    const AUTHORITY: Global<Authority> = global_component!(
        Authority,
        "component_sim1czc0e8f9yhlvpv38s2ymrplu7q366y3k8zc53zf2srlm7qm604g029"
    );

    // Set access rules
    enable_method_auth! { 
        roles {
            trader => updatable_by: [OWNER];
            withdrawer => updatable_by: [OWNER];
            depositor => updatable_by: [OWNER];
        },
        methods { 
            get_info => PUBLIC;

            // Authority protected methods
            update => PUBLIC;
            deposit_collateral => PUBLIC;
            deposit_collateral_batch => PUBLIC;
            withdraw_collateral => PUBLIC;
            withdraw_collateral_batch => PUBLIC;
        }
    }

    struct MarginAccount {
        collateral: Vaults,
        positions: HashMap<u64, AccountPosition>,
        collateral_balances: HashMap<ResourceAddress, Decimal>,
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
    }

    impl MarginAccount {
        pub fn new() -> Global<MarginAccount> {
            let (component_reservation, _this) = Runtime::allocate_component_address(MarginAccount::blueprint_id());

            Self {
                collateral: Vaults::new(),
                positions: HashMap::new(),
                collateral_balances: HashMap::new(),
                virtual_balance: dec!(0),
                requests: List::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None) // TODO: set the owner role
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self) -> MarginAccountInfo {
            MarginAccountInfo {
                positions: self.positions.clone(),
                collateral_balances: self.collateral_balances.clone(),
                virtual_balance: self.virtual_balance,
            }
        }

        pub fn update(&mut self, update: MarginAccountUpdates) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for (pair_id, position) in update.position_updates {
                if position.amount != dec!(0) {
                    self.positions.insert(pair_id, position);
                } else {
                    self.positions.remove(&pair_id);
                }
            }
            self.virtual_balance = update.virtual_balance;
        }

        pub fn deposit_collateral(&mut self, token: Bucket) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            let amount = token.amount();
            let resource = token.resource_address();
            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);

            self.collateral.put(token);
        }

        pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for token in tokens.iter() {
                let amount = token.amount();
                let resource = token.resource_address();
                self.collateral_balances
                    .entry(resource)
                    .and_modify(|balance| *balance += amount)
                    .or_insert(amount);
            }

            self.collateral.put_batch(tokens);
        }

        pub fn withdraw_collateral(&mut self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance -= amount)
                .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));

            self.collateral.take_advanced(&resource, amount, withdraw_strategy)
        }

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for (resource, amount) in claims.iter() {
                self.collateral_balances
                    .entry(*resource)
                    .and_modify(|balance| *balance -= *amount)
                    .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));
            }
            
            let tokens = self.collateral.take_advanced_batch(claims, withdraw_strategy);
            
            tokens
        }
    }
}
