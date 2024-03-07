mod errors;
mod consts;

use scrypto::prelude::*;
use self::consts::*;
use self::errors::*;

#[derive(ScryptoSbor)]
pub struct ChildToken {
    pub vault: Vault,
    pub wrappable: bool,
}

#[blueprint]
mod token_wrapper {
    enable_method_auth!(
        roles {
            user => updatable_by: [OWNER];
        },
        methods {
            add_child => restrict_to: [OWNER];
            update_wrappable => restrict_to: [OWNER];
            
            wrap => restrict_to: [user];
            unwrap => restrict_to: [user];
        }
    );

    struct TokenWrapper {
        parent_token: ResourceManager,
        child_vaults: KeyValueStore<ResourceAddress, ChildToken>,
    }

    impl TokenWrapper {
        pub fn new(owner_rule: AccessRule) -> Global<TokenWrapper> {
            let (component_reservation, _this) = Runtime::allocate_component_address(TokenWrapper::blueprint_id());
            
            let parent_token = ResourceBuilder::new_fungible(OwnerRole::Updatable(owner_rule.clone()))
                .create_with_no_initial_supply();

            Self {
                parent_token,
                child_vaults: KeyValueStore::new(),
            }
            .instantiate()  
            .prepare_to_globalize(OwnerRole::Updatable(owner_rule))
            .roles(roles!  {
                user => rule!(allow_all);
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn add_child(&mut self, child_resource: ResourceAddress) {
            let child_vault = ChildToken {
                vault: Vault::new(child_resource),
                wrappable: true,
            };

            self.child_vaults.insert(child_resource, child_vault);
        }

        pub fn update_wrappable(&mut self, child_resource: ResourceAddress, wrappable: bool) {
            let mut child_vault = self.child_vaults.get_mut(&child_resource).expect(ERROR_INVALID_CHILD_TOKEN);
            child_vault.wrappable = wrappable;
        }

        pub fn wrap(&mut self, child_token: Bucket) -> Bucket {
            let mut child_vault = self.child_vaults.get_mut(&child_token.resource_address()).expect(ERROR_INVALID_CHILD_TOKEN);

            assert!(
                child_vault.wrappable, 
                "{}", ERROR_WRAPPING_DISABLED
            );

            let parent_token = self.parent_token.mint(child_token.amount());
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
