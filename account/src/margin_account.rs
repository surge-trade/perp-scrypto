mod consts;
pub mod structs;

use scrypto::prelude::*;
use utils::{List, Vaults};
use self::consts::*;
pub use self::structs::*;

#[blueprint]
pub mod margin_account {
    enable_function_auth! {
        new => rule!(require(AUTHORITY_RESOURCE));
    }
    enable_method_auth! { 
        roles {
            authority => updatable_by: [];
            level_1 => updatable_by: [authority];
            level_2 => updatable_by: [authority];
            level_3 => updatable_by: [authority];
        },
        methods { 
            get_info => PUBLIC;
            get_request => PUBLIC;
            get_requests => PUBLIC;
            get_requests_tail => PUBLIC;
            get_requests_len => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            push_request => restrict_to: [authority];
            set_request_status => restrict_to: [authority];
            set_request_statuses => restrict_to: [authority];
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
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
        last_liquidation: Instant,
    }

    impl MarginAccount {
        pub fn new(initial_rule: AccessRule) -> Global<MarginAccount> {
            let (component_reservation, _this) = Runtime::allocate_component_address(MarginAccount::blueprint_id());

            Self {
                collateral: Vaults::new(),
                positions: HashMap::new(), // TODO: make kvs for efficient token movement
                virtual_balance: dec!(0),
                requests: List::new(), // TODO: global ref list?
                last_liquidation: Instant::new(0),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
                level_1 => initial_rule.clone();
                level_2 => initial_rule.clone();
                level_3 => initial_rule.clone();
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self, collateral_resources: Vec<ResourceAddress>) -> MarginAccountInfo {
            MarginAccountInfo {
                positions: self.positions.clone(),
                collateral_balances: self.collateral.amounts(collateral_resources),
                virtual_balance: self.virtual_balance,
                last_liquidation: self.last_liquidation,
            }
        }

        pub fn get_request(&self, index: u64) -> Option<KeeperRequest> {
            self.requests.get(index).map(|request| request.clone())
        }

        pub fn get_requests(&self, start: u64, end: u64) -> Vec<KeeperRequest> {
            self.requests.range(start, end)
        }

        pub fn get_requests_tail(&self, num: u64) -> Vec<KeeperRequest> {
            self.requests.range(self.requests.len() - num, self.requests.len())
        }

        pub fn get_requests_len(&self) -> u64 {
            self.requests.len()
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
            self.last_liquidation = update.last_liquidation;
        }

        pub fn push_request(&mut self, request: KeeperRequest) {
            self.requests.push(request);
        }

        pub fn set_request_status(&mut self, index: u64, status: u8) -> Option<u8> {
            if let Some(mut request) = self.requests.get_mut(index) {
                let temp = request.status;
                request.status = status;
                Some(temp)
            } else {
                None
            }
        }

        pub fn set_request_statuses(&mut self, updates: Vec<(u64, u8)>) -> Vec<Option<u8>> {
            updates
                .into_iter()
                .map(|(index, status)| self.set_request_status(index, status))
                .collect()
        }

        pub fn process_request(&mut self, index: u64, status: u8) -> Option<KeeperRequest> {
            let mut request = self.requests.get_mut(index)?;
            let temp = request.clone();
            request.status = status;
            Some(temp)
        }

        pub fn deposit_collateral(&mut self, token: Bucket) {
            self.collateral.put(token);
        }

        pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
            self.collateral.put_batch(tokens);
        }

        pub fn withdraw_collateral(&mut self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            self.collateral.take_advanced(&resource, amount, withdraw_strategy)
        }

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {            
            self.collateral.take_advanced_batch(claims, withdraw_strategy)
        }
    }
}
