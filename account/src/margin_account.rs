pub mod structs;

use scrypto::prelude::*;
use common::{PairId, ListIndex, List, Vaults, _AUTHORITY_RESOURCE};
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
            get_active_requests => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            update_referral_id => restrict_to: [authority];
            deposit_collateral_batch => restrict_to: [authority];
            withdraw_collateral_batch => restrict_to: [authority];
        }
    }
    
    struct MarginAccount {
        collateral: Vaults,
        collateral_balances: HashMap<ResourceAddress, Decimal>,
        positions: HashMap<PairId, AccountPosition>,
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
        active_requests: HashSet<ListIndex>,
        valid_requests_start: ListIndex,
        referral_id: Option<NonFungibleLocalId>,
    }

    impl MarginAccount {
        pub fn new(
            level_1: AccessRule, 
            level_2: AccessRule, 
            level_3: AccessRule, 
            referral_id: Option<NonFungibleLocalId>,
            dapp_definition: GlobalAddress,
            reservation: Option<GlobalAddressReservation>,
        ) -> Global<MarginAccount> {
            let component_reservation = match reservation {
                Some(reservation) => reservation,
                None => Runtime::allocate_component_address(MarginAccount::blueprint_id()).0
            };

            Self {
                collateral: Vaults::new(MarginAccountKeyValueStore::new_with_registered_type),
                collateral_balances: HashMap::new(),
                positions: HashMap::new(),
                virtual_balance: dec!(0),
                requests: List::new(MarginAccountKeyValueStore::new_with_registered_type),
                active_requests: HashSet::new(),
                valid_requests_start: 0,
                referral_id,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(AUTHORITY_RESOURCE))))
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
                level_1 => level_1;
                level_2 => level_2;
                level_3 => level_3;
            })
            .metadata(metadata! {
                init {
                    "dapp_definition" => dapp_definition, updatable;
                }
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self) -> MarginAccountInfo {
            MarginAccountInfo {
                positions: self.positions.clone(),
                collateral_balances: self.collateral_balances.clone(),
                virtual_balance: self.virtual_balance,
                requests_len: self.requests.len(),
                active_requests_len: self.active_requests.len(),
                valid_requests_start: self.valid_requests_start,
                referral_id: self.referral_id.clone(),
            }
        }

        pub fn get_request(&self, index: ListIndex) -> Option<KeeperRequest> {
            self.requests.get(index).map(|request| request.clone())
        }

        pub fn get_requests(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)> {
            let start = start.unwrap_or(0);
            let end = (start + n).min(self.requests.len());
            (start..end).zip(self.requests.range(start, end).into_iter()).collect()
        }

        pub fn get_requests_tail(&self, n: ListIndex, end: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)> {
            let len = self.requests.len();
            let end = end.map(|end| (end + 1).min(len)).unwrap_or(len);
            let start = end - n.min(end);
            (start..end).rev().zip(self.requests.range(start, end).into_iter().rev()).collect()
        }

        pub fn get_requests_by_indexes(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, Option<KeeperRequest>> {
            indexes.into_iter().map(|index| (index, self.get_request(index))).collect()
        }

        pub fn get_requests_len(&self) -> ListIndex {
            self.requests.len()
        }

        pub fn get_active_requests(&self) -> HashMap<ListIndex, KeeperRequest> {
            self.active_requests.iter().map(|index| (*index, self.get_request(*index).unwrap())).collect()
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
            if update.valid_requests_start > self.valid_requests_start {
                self.valid_requests_start = update.valid_requests_start;
                self.active_requests = self.active_requests.iter().filter(|index| **index >= self.valid_requests_start).cloned().collect();
            }
            for request in update.request_additions {
                self.requests.push(request);
            }
            for (index, updated_request) in update.request_updates {
                self.requests.update(index, updated_request);
            }
            self.active_requests.extend(update.active_request_additions);
            for removal in update.active_request_removals {
                self.active_requests.remove(&removal);
            }
        }

        pub fn update_referral_id(&mut self, referral_id: Option<NonFungibleLocalId>) {
            self.referral_id = referral_id;
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

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {            
            let tokens = self.collateral.take_advanced_batch(claims, withdraw_strategy);
            for token in tokens.iter() {
                let amount = token.amount();
                let resource = token.resource_address();
                self.collateral_balances
                    .entry(resource)
                    .and_modify(|balance| *balance -= amount);
                if self.collateral_balances[&resource].is_zero() {
                    self.collateral_balances.remove(&resource);
                }
            }

            tokens
        }
    }
}
