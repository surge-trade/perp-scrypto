mod structs;

use scrypto::prelude::*;
use utils::{PairId, ListIndex, List, Vaults, _AUTHORITY_RESOURCE};
pub use self::structs::*;

#[blueprint]
#[types(
    ListIndex,
    KeeperRequest,
    ResourceAddress,
    Vault,
)]
pub mod margin_account {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

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
            // Public methods
            get_info => PUBLIC;
            get_request => PUBLIC;
            get_requests => PUBLIC;
            get_requests_tail => PUBLIC;
            get_requests_by_indexes => PUBLIC;
            get_requests_len => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            deposit_collateral_batch => restrict_to: [authority];
            withdraw_collateral_batch => restrict_to: [authority];
        }
    }
    
    struct MarginAccount {
        collateral: Vaults,
        positions: HashMap<PairId, AccountPosition>, // TODO: make kvs for efficient token movement
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
        last_liquidation_index: ListIndex,
    }

    impl MarginAccount {
        pub fn new(initial_rule: AccessRule, reservation: Option<GlobalAddressReservation>) -> Global<MarginAccount> {
            let component_reservation = match reservation {
                Some(reservation) => reservation,
                None => Runtime::allocate_component_address(MarginAccount::blueprint_id()).0
            };

            Self {
                collateral: Vaults::new(MarginAccountKeyValueStore::new_with_registered_type),
                positions: HashMap::new(), // TODO: make kvs for efficient token movement
                virtual_balance: dec!(0),
                requests: List::new(MarginAccountKeyValueStore::new_with_registered_type),
                last_liquidation_index: 0,
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
                requests_len: self.requests.len(),
                last_liquidation_index: self.last_liquidation_index,
            }
        }

        pub fn get_request(&self, index: ListIndex) -> Option<KeeperRequest> {
            self.requests.get(index).map(|request| request.clone())
        }

        pub fn get_requests(&self, start: ListIndex, end: ListIndex) -> Vec<KeeperRequest> {
            self.requests.range(start, end)
        }

        pub fn get_requests_tail(&self, num: ListIndex) -> Vec<KeeperRequest> {
            self.requests.range(self.requests.len() - num, self.requests.len())
        }

        pub fn get_requests_by_indexes(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, Option<KeeperRequest>> {
            indexes.into_iter().map(|index| (index, self.get_request(index))).collect()
        }

        pub fn get_requests_len(&self) -> ListIndex {
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
            self.last_liquidation_index = update.last_liquidation_index;

            for request in update.requests_new {
                self.requests.push(request);
            }

            for (index, updated_request) in update.request_updates {
                self.requests.update(index, updated_request);
            }
        }

        pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
            self.collateral.put_batch(tokens);
        }

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {            
            self.collateral.take_advanced_batch(claims, withdraw_strategy)
        }
    }
}
