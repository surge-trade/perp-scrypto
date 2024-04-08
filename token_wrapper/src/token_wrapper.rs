mod errors;

use scrypto::prelude::*;
use utils::{ListIndex, List, _AUTHORITY_RESOURCE, _BASE_RESOURCE, TO_ZERO};
use self::errors::*;

#[derive(ScryptoSbor)]
pub struct ChildToken {
    pub vault: Vault,
    pub wrappable: bool,
}

#[blueprint]
#[types(
    ListIndex,
    ResourceAddress,
    ChildToken,
)]
mod token_wrapper {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;

    enable_method_auth!(
        roles {
            user => updatable_by: [OWNER];
        },
        methods {
            deposit_authority => restrict_to: [OWNER];
            withdraw_authority => restrict_to: [OWNER];
            add_child => restrict_to: [OWNER];
            update_wrappable => restrict_to: [OWNER];
            
            get_children => PUBLIC;
            
            wrap => restrict_to: [user];
            unwrap => restrict_to: [user];
        }
    );

    macro_rules! authorize {
        ($self:expr, $func:expr) => {{
            $self.authority_token.authorize_with_amount(dec!(0.000000000000000001),|| {
                $func
            })
        }};
    }

    struct TokenWrapper {
        authority_token: FungibleVault,
        child_list: List<ResourceAddress>,
        child_vaults: KeyValueStore<ResourceAddress, ChildToken>,
    }

    impl TokenWrapper {
        pub fn new(owner_role: OwnerRole, authority_token: Bucket) -> Global<TokenWrapper> {
            assert!(
                authority_token.resource_address() == AUTHORITY_RESOURCE,
                "{}", ERROR_INVALID_AUTHORITY
            );

            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                child_list: List::new(TokenWrapperKeyValueStore::new_with_registered_type),
                child_vaults: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .roles(roles!  {
                user => rule!(allow_all);
            })
            .globalize()
        }

        pub fn deposit_authority(&mut self, token: Bucket) {
            self.authority_token.put(token.as_fungible());
        }

        pub fn withdraw_authority(&mut self) -> Bucket {
            self.authority_token.take_all().into()
        }

        pub fn add_child(&mut self, child_resource: ResourceAddress) {
            self.child_list.push(child_resource);
            self.child_vaults.insert(
                child_resource, 
                ChildToken {
                    vault: Vault::new(child_resource),
                    wrappable: true,
                }
            );
        }

        pub fn update_wrappable(&mut self, child_resource: ResourceAddress, wrappable: bool) {
            let mut child_vault = self.child_vaults.get_mut(&child_resource).expect(ERROR_INVALID_CHILD_TOKEN);
            child_vault.wrappable = wrappable;
        }

        pub fn get_children(&self, start: ListIndex, end: ListIndex) -> Vec<(ResourceAddress, bool, Decimal)> {
            self.child_list.range(start, end).into_iter().map(|child_resource| {
                let child_vault = self.child_vaults.get(&child_resource).unwrap();
                (child_resource, child_vault.wrappable, child_vault.vault.amount())
            }).collect()
        }

        pub fn wrap(&mut self, child_token: Bucket) -> Bucket {
            let mut child_vault = self.child_vaults.get_mut(&child_token.resource_address()).expect(ERROR_INVALID_CHILD_TOKEN);

            assert!(
                child_vault.wrappable, 
                "{}", ERROR_WRAPPING_DISABLED
            );

            let parent_token = authorize!(self, {
                ResourceManager::from_address(BASE_RESOURCE).mint(child_token.amount())
            });
            child_vault.vault.put(child_token);

            parent_token
        }

        pub fn unwrap(&mut self, parent_token: Bucket, child_resource: ResourceAddress) -> Bucket {
            let mut child_vault = self.child_vaults.get_mut(&child_resource).expect(ERROR_INVALID_CHILD_TOKEN);

            let child_token = child_vault.vault.take_advanced(parent_token.amount(), TO_ZERO);
            parent_token.burn();

            child_token
        }
    }
}
