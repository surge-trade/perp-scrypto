use scrypto::prelude::*;
use common::_AUTHORITY_RESOURCE;

#[blueprint]
mod fee_distributor_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
        },
        methods {
            get_protocol_virtual_balance => PUBLIC;
            get_treasury_virtual_balance => PUBLIC;

            update_protocol_virtual_balance => restrict_to: [authority];
            update_treasury_virtual_balance => restrict_to: [authority];
            distribute => restrict_to: [authority];
        }
    );

    struct FeeDistributor {
        treasury_virtual_balance: Decimal,
        protocol_virtual_balance: Decimal,
    }

    impl FeeDistributor {
        pub fn new(owner_role: OwnerRole) -> Global<FeeDistributor> {
            Self {
                treasury_virtual_balance: dec!(0),
                protocol_virtual_balance: dec!(0),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .globalize()
        }
        
        pub fn get_protocol_virtual_balance(&self) -> Decimal {
            self.protocol_virtual_balance
        }
        
        pub fn get_treasury_virtual_balance(&self) -> Decimal {
            self.treasury_virtual_balance
        }

        pub fn update_protocol_virtual_balance(&mut self, protocol_virtual_balance: Decimal) {
            assert!(protocol_virtual_balance >= dec!(0));
            self.protocol_virtual_balance = protocol_virtual_balance;
        }
        
        pub fn update_treasury_virtual_balance(&mut self, treasury_virtual_balance: Decimal) {
            assert!(treasury_virtual_balance >= dec!(0));
            self.treasury_virtual_balance = treasury_virtual_balance;
        }

        pub fn distribute(&mut self, amount_protocol: Decimal, amount_treasury: Decimal) {
            self.protocol_virtual_balance += amount_protocol;
            self.treasury_virtual_balance += amount_treasury;
        }
    }
}