mod errors;

use scrypto::prelude::*;
use common::{_AUTHORITY_RESOURCE, TO_ZERO};
use self::errors::*;

#[blueprint]
mod fee_delegator_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth! { 
        roles {
            user => updatable_by: [OWNER];
            depositor => updatable_by: [OWNER];
            withdrawer => updatable_by: [OWNER];
        },
        methods { 
            get_fee_oath_resource => PUBLIC;
            get_max_lock => PUBLIC;
            get_is_contingent => PUBLIC;
            get_vault_amount => PUBLIC;
            get_virtual_balance => PUBLIC;

            update_max_lock => restrict_to: [OWNER];
            update_is_contingent => restrict_to: [OWNER];
            update_virtual_balance => restrict_to: [OWNER];
            deposit => restrict_to: [depositor];
            withdraw => restrict_to: [withdrawer];

            lock_fee => restrict_to: [user];
        }
    }

    struct FeeDelegator {
        fee_oath: ResourceManager,
        vault: FungibleVault,
        virtual_balance: Decimal,
        max_lock: Decimal,
        is_contingent: bool,
    }

    impl FeeDelegator {
        pub fn new(owner_role: OwnerRole) -> Global<FeeDelegator> {
            let (component_reservation, this) = Runtime::allocate_component_address(FeeDelegator::blueprint_id());

            let fee_oath = ResourceBuilder::new_fungible(owner_role.clone())
                .metadata(metadata!(
                    init {
                        "package" => GlobalAddress::from(Runtime::package_address()), locked;
                        "component" => GlobalAddress::from(this), locked;
                        "name" => format!("Fee Oath"), updatable;
                        "symbol" => format!("FEE"), updatable;
                        "description" => format!("Represents the obligation to pay fees for a transaction."), updatable;
                    }
                ))
                .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(this))); 
                        minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                        burner => rule!(require(AUTHORITY_RESOURCE)); 
                        burner_updater => rule!(deny_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor => rule!(deny_all); 
                    depositor_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();
                
            Self {
                max_lock: dec!(3),
                is_contingent: false,
                vault: FungibleVault::new(XRD),
                virtual_balance: dec!(0),
                fee_oath,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                user => rule!(allow_all);
                depositor => OWNER;
                withdrawer => OWNER;
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_fee_oath_resource(&self) -> ResourceAddress {
            self.fee_oath.address()
        }

        pub fn get_max_lock(&self) -> Decimal {
            self.max_lock
        }

        pub fn get_is_contingent(&self) -> bool {
            self.is_contingent
        }

        pub fn get_virtual_balance(&self) -> Decimal {
            self.virtual_balance
        }

        pub fn get_vault_amount(&self) -> Decimal {
            self.vault.amount()
        }

        pub fn update_max_lock(&mut self, max_lock: Decimal) {
            assert!(max_lock >= dec!(0));
            self.max_lock = max_lock;
        }

        pub fn update_is_contingent(&mut self, is_contingent: bool) {
            self.is_contingent = is_contingent;
        }

        pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
            assert!(virtual_balance >= dec!(0));
            self.virtual_balance = virtual_balance;
        }

        pub fn deposit(&mut self, token: Bucket) {
            self.vault.put(token.as_fungible());
        }

        pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
            let token = self.vault.take_advanced(amount, TO_ZERO);
            token.into()
        }

        pub fn lock_fee(&mut self, amount: Decimal) -> Bucket {
            if self.is_contingent {
                self.vault.lock_contingent_fee(amount);
            } else {
                self.vault.lock_fee(amount);
            }

            let price = Runtime::get_usd_price();
            let value = price * amount;
            self.virtual_balance += value;

            let fee_oath = self.fee_oath.mint(value);
            let total_supply = self.fee_oath.total_supply().unwrap();
            assert!(
                total_supply <= self.max_lock, 
                "{}, VALUE:{}, REQUIRED:{}, OP:<= |", MAX_LOCK_EXCEEDED, total_supply, self.max_lock
            );
            
            fee_oath
        }
    }
}