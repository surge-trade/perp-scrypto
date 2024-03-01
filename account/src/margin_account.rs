mod errors;
mod consts;

use scrypto::prelude::*;
use utils::{List, Vaults};
use self::errors::*;
use self::consts::*;

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

#[derive(ScryptoSbor, Clone)]
pub struct KeeperRequest {
    pub data: Vec<u8>,
    pub expiry: Instant,
    pub processed: bool,
}

#[blueprint]
pub mod margin_account {
    // Set access rules
    enable_method_auth! { 
        roles {
            authority => updatable_by: [];
            trader => updatable_by: [OWNER];
            withdrawer => updatable_by: [OWNER];
            depositor => updatable_by: [OWNER];
        },
        methods { 
            get_info => PUBLIC;
            get_request => PUBLIC;
            get_requests => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            process_request => restrict_to: [authority];
            deposit_collateral => restrict_to: [authority];
            deposit_collateral_batch => restrict_to: [authority];
            withdraw_collateral => restrict_to: [authority];
            withdraw_collateral_batch => restrict_to: [authority];
        }
    }

    struct MarginAccount {
        collateral: Vaults,
        positions: HashMap<u64, AccountPosition>, // TODO: make kvs for efficient token movement
        collateral_balances: HashMap<ResourceAddress, Decimal>, // TODO: remove?
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
    }

    impl MarginAccount {
        pub fn new(owner_rule: AccessRule) -> Global<MarginAccount> {
            let (component_reservation, _this) = Runtime::allocate_component_address(MarginAccount::blueprint_id());

            Self {
                collateral: Vaults::new(),
                positions: HashMap::new(),
                collateral_balances: HashMap::new(),
                virtual_balance: dec!(0),
                requests: List::new(), // TODO: global ref list
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(owner_rule)) // TODO: set the owner role
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
                trader => OWNER;
                withdrawer => OWNER;
                depositor => OWNER;
            })
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

        pub fn get_request(&self, index: u64) -> Option<KeeperRequest> {
            self.requests.get(index)
        }

        pub fn get_requests(&self, start: u64, end: u64) -> Vec<KeeperRequest> {
            self.requests.range(start, end)
        }

        pub fn update(&mut self, update: MarginAccountUpdates) {
            for (pair_id, position) in update.position_updates {
                if position.amount != dec!(0) {
                    self.positions.insert(pair_id, position);
                } else {
                    self.positions.remove(&pair_id);
                }
            }
            self.virtual_balance = update.virtual_balance;
        }

        pub fn process_request(&mut self, index: u64) -> Option<KeeperRequest> {
            let mut request = self.requests.get_mut(index)?;
            let temp = request.clone();
            request.processed = true;
            Some(temp)
        }

        pub fn deposit_collateral(&mut self, token: Bucket) {
            let amount = token.amount();
            let resource = token.resource_address();
            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);

            self.collateral.put(token);
        }

        pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
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
            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance -= amount)
                .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));

            self.collateral.take_advanced(&resource, amount, withdraw_strategy)
        }

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
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
