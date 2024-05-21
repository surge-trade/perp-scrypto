use scrypto::prelude::*;

#[blueprint]
#[types(
    String,
    MetadataValue,
)]
mod env_registry_mod {
    enable_method_auth!(
        methods {
            get_variables => PUBLIC;
            set_variables => restrict_to: [OWNER];
            remove_variables => restrict_to: [OWNER];
        }
    );

    pub struct EnvRegistry {
        variables: KeyValueStore<String, MetadataValue>,
    }

    impl EnvRegistry {
        pub fn new(owner_role: OwnerRole) -> Global<EnvRegistry> {
            Self {
                variables: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn get_variables(&self, keys: Vec<String>) -> HashMap<String, Option<MetadataValue>> {
            keys.into_iter().map(|key| (key.clone(), self.variables.get(&key).map(|value| value.clone()))).collect()
        }

        pub fn set_variables(&self, variables: Vec<(String, MetadataValue)>) {
            for (key, value) in variables {
                self.variables.insert(key, value);
            }
        }

        pub fn remove_variables(&self, keys: Vec<String>) {
            for key in keys {
                self.variables.remove(&key);
            }
        }
    }
}
