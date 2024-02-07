pub mod account_position;
pub use self::account_position::AccountPosition;

use scrypto::prelude::*;
use crate::utils::{List, Vaults, Pair};
use super::keeper_requests::KeeperRequest;

#[derive(ScryptoSbor)]
pub struct MarginAccount {
    pub vaults: Vaults,
    pub positions: HashMap<Pair, AccountPosition>,
    pub collateral: Vaults,
    pub virtual_balance: Decimal,
    pub requests: List<KeeperRequest>,
}

impl MarginAccount {
    pub fn new() -> Self {
        Self {
            vaults: Vaults::new(),
            positions: HashMap::new(),
            collateral: Vaults::new(),
            virtual_balance: dec!(0),
            requests: List::new(),
        }
    }
}

#[derive(ScryptoSbor)]
pub struct MarginAccountManager {
    pub account_ids: List<NonFungibleLocalId>,
    accounts: KeyValueStore<NonFungibleLocalId, MarginAccount>,
    account_badge_manager: ResourceManager,
}

impl MarginAccountManager {
    pub fn new(this: ComponentAddress, owner_role: OwnerRole) -> Self {
        let account_badge_manager = ResourceBuilder::new_ruid_non_fungible::<()>(owner_role)
            .metadata(metadata!(
                init {
                    "package" => GlobalAddress::from(Runtime::package_address()), locked;
                    "component" => GlobalAddress::from(this), locked;
                    "name" => format!("Margin Account Badge"), updatable;
                    "description" => format!("Used to control a margin account."), updatable;
                }
            ))
            .mint_roles(mint_roles!{
                    minter => rule!(require(global_caller(this))); 
                    minter_updater => rule!(deny_all);
                }
            )
            .burn_roles(burn_roles!{
                    burner => rule!(require(global_caller(this))); 
                    burner_updater => rule!(deny_all);
                }
            )
            .create_with_no_initial_supply();

        Self {
            account_ids: List::new(),
            accounts: KeyValueStore::new(),
            account_badge_manager,
        }
    }

    pub fn get(&self, id: &NonFungibleLocalId) -> Option<KeyValueEntryRef<MarginAccount>> {
        self.accounts.get(id)
    }

    pub fn get_mut(&mut self, id: &NonFungibleLocalId) -> Option<KeyValueEntryRefMut<MarginAccount>> {
        self.accounts.get_mut(id)
    }

    pub fn create_account(&mut self) -> Bucket {
        let badge = self.account_badge_manager.mint_ruid_non_fungible(());
        let id = badge.as_non_fungible().non_fungible_local_id();
        self.account_ids.push(id.clone());
        self.accounts.insert(id, MarginAccount::new());

        badge
    }

    pub fn check_ownership(&self, proof: Proof) -> NonFungibleLocalId {
        let checked_proof = proof.check(self.account_badge_manager.address());
        checked_proof.as_non_fungible().non_fungible_local_id()
    }
}
