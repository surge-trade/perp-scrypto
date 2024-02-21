use scrypto::prelude::*;

#[blueprint]
pub mod authority {
    enable_method_auth! {
        methods {
            set_rule => restrict_to: [OWNER];
            get_rule => PUBLIC;
        }
    }

    struct Authority {
        rule: AccessRule,
    }

    impl Authority {
        pub fn new(admin_badge: ResourceAddress, component: ComponentAddress) -> Global<Authority> {
            let rule = rule!(require(global_caller(component)));

            Self {
                rule
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(admin_badge))))
            .globalize()
        }

        pub fn set_rule(&mut self, component: ComponentAddress) {
            let rule = rule!(require(global_caller(component)));
            self.rule = rule;
        }

        pub fn get_rule(&self) -> AccessRule {
            self.rule.clone()
        }
    }
}

