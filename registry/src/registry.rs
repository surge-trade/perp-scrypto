use scrypto::prelude::*;
use common::_AUTHORITY_RESOURCE;

#[derive(ScryptoSbor, Clone, Default)]
pub struct Permissions {
    pub level_1: HashSet<ComponentAddress>,
    pub level_2: HashSet<ComponentAddress>,
    pub level_3: HashSet<ComponentAddress>,
}

#[blueprint]
#[types(
    AccessRule,
    Permissions,
)]
mod registry_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_permissions => PUBLIC;

            register_permissions => restrict_to: [authority];
        }
    );

    pub struct Registry {
        permissions: KeyValueStore<AccessRule, Permissions>,
    }

    impl Registry {
        pub fn new(owner_role: OwnerRole) -> Global<Registry> {
            Self {
                permissions: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }

        pub fn get_permissions(&self, access_rule: AccessRule) -> Permissions {
            self.permissions.get(&access_rule).map(|p| p.clone()).unwrap_or_default()
        }

        pub fn register_permissions(&mut self, access_rule: AccessRule, permissions: Permissions) {
            self.permissions.insert(access_rule, permissions);
        }
    }
}