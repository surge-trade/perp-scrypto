pub mod errors;
pub mod structs;

use scrypto::prelude::*;
use common::{ListIndex, List, _BASE_AUTHORITY_RESOURCE, _BASE_RESOURCE, TO_ZERO, TO_INFINITY};
pub use self::errors::*;
pub use self::structs::*;

#[blueprint]
#[types(
    ListIndex,
    ResourceAddress,
    Input,
)]
mod token_wrapper_mod {
    const BASE_AUTHORITY_RESOURCE: ResourceAddress = _BASE_AUTHORITY_RESOURCE;
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;

    enable_method_auth!(
        roles {
            admin => updatable_by: [OWNER];
            wrap_user => updatable_by: [OWNER];
            unwrap_user => updatable_by: [OWNER];
            flash_user => updatable_by: [OWNER];
        },
        methods {
            deposit_authority => restrict_to: [OWNER];
            withdraw_authority => restrict_to: [OWNER];
            add_input => restrict_to: [OWNER];
            update_wrappable => restrict_to: [OWNER, admin];
            
            get_inputs => PUBLIC;
            
            wrap => restrict_to: [wrap_user];
            unwrap => restrict_to: [unwrap_user];
            flash_loan => restrict_to: [flash_user];
            repay_flash_loan => restrict_to: [flash_user];
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
        input_list: List<ResourceAddress>,
        inputs: KeyValueStore<ResourceAddress, Input>,
        flash_oath: ResourceManager,
    }

    impl TokenWrapper {
        pub fn new(owner_role: OwnerRole, authority_token: Bucket) -> Global<TokenWrapper> {
            let (component_reservation, this) = Runtime::allocate_component_address(TokenWrapper::blueprint_id());

            assert!(
                authority_token.resource_address() == BASE_AUTHORITY_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_AUTHORITY, Runtime::bech32_encode_address(authority_token.resource_address()), Runtime::bech32_encode_address(BASE_AUTHORITY_RESOURCE)
            );

            let flash_oath = ResourceBuilder::new_fungible(owner_role.clone())
                .metadata(metadata!(
                    init {
                        "package" => GlobalAddress::from(Runtime::package_address()), locked;
                        "component" => GlobalAddress::from(this), locked;
                        "name" => format!("Flash Oath"), updatable;
                        "symbol" => format!("FLASH"), updatable;
                        "description" => format!("Represents the obligation to repay a flash loan."), updatable;
                    }
                ))
                .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(this))); 
                        minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                        burner => rule!(require(global_caller(this))); 
                        burner_updater => rule!(deny_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor => rule!(deny_all); 
                    depositor_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            Self {
                authority_token: FungibleVault::with_bucket(authority_token.as_fungible()),
                input_list: List::new(TokenWrapperKeyValueStore::new_with_registered_type),
                inputs: KeyValueStore::new_with_registered_type(),
                flash_oath,
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                admin => OWNER;
                wrap_user => rule!(allow_all);
                unwrap_user => rule!(allow_all);
                flash_user => rule!(allow_all);
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn deposit_authority(&mut self, token: Bucket) {
            self.authority_token.put(token.as_fungible());
        }

        pub fn withdraw_authority(&mut self) -> Bucket {
            self.authority_token.take_all().into()
        }

        pub fn add_input(&mut self, resource: ResourceAddress) {
            if self.inputs.get(&resource).is_none() {
                self.input_list.push(resource);
                self.inputs.insert(
                    resource, 
                    Input {
                        vault: Vault::new(resource),
                        wrappable: true,
                    }
                );
            }
        }

        pub fn update_wrappable(&mut self, child_resource: ResourceAddress, wrappable: bool) {
            let mut child_vault = self.inputs.get_mut(&child_resource).expect(ERROR_INVALID_CHILD_TOKEN);
            child_vault.wrappable = wrappable;
        }

        pub fn get_inputs(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ResourceAddress, bool, Decimal)> {
            let start = start.unwrap_or(0);
            let end = (start + n).min(self.input_list.len());
            self.input_list.range(start, end).into_iter().map(|child_resource| {
                let child_vault = self.inputs.get(&child_resource).unwrap();
                (child_resource, child_vault.wrappable, child_vault.vault.amount())
            }).collect()
        }

        pub fn wrap(&mut self, child_token: Bucket) -> Bucket {
            let mut child_vault = self.inputs.get_mut(&child_token.resource_address()).expect(ERROR_INVALID_CHILD_TOKEN);

            assert!(
                child_vault.wrappable, 
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_WRAPPING_DISABLED, child_vault.wrappable, true
            );

            let parent_token = authorize!(self, {
                ResourceManager::from_address(BASE_RESOURCE).mint(child_token.amount())
            });
            child_vault.vault.put(child_token);

            parent_token
        }

        pub fn unwrap(&mut self, parent_token: Bucket, child_resource: ResourceAddress) -> Bucket {
            assert!(
                parent_token.resource_address() == BASE_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT_TOKEN, Runtime::bech32_encode_address(parent_token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );
            
            let mut child_vault = self.inputs.get_mut(&child_resource).expect(ERROR_INVALID_CHILD_TOKEN);

            let child_token = child_vault.vault.take_advanced(parent_token.amount(), TO_ZERO);
            parent_token.burn();

            child_token
        }

        pub fn flash_loan(&mut self, amount: Decimal) -> (Bucket, Bucket) {
            let flash_oath = self.flash_oath.mint(amount);
            let token = authorize!(self, {
                ResourceManager::from_address(BASE_RESOURCE).mint(amount)
            });

            (token, flash_oath)
        }

        pub fn repay_flash_loan(&mut self, flash_oath: Bucket, mut token: Bucket) -> Bucket {
            assert!(
                flash_oath.resource_address() == self.flash_oath.address(),
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_FLASH_OATH_TOKEN, Runtime::bech32_encode_address(flash_oath.resource_address()), Runtime::bech32_encode_address(self.flash_oath.address())
            );
            assert!(
                token.resource_address() == BASE_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT_TOKEN, Runtime::bech32_encode_address(token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );
            assert!(
                token.amount() >= flash_oath.amount(),
                "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_INSUFFICIENT_FLASH_LOAN_PAYMENT, token.amount(), flash_oath.amount()
            );

            token.take_advanced(flash_oath.amount(), TO_INFINITY).burn();
            flash_oath.burn();

            token
        }
    }
}
