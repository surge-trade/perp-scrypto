pub mod errors;

use scrypto::prelude::*;
use common::{ListIndex, _BASE_RESOURCE, TOKEN_WRAPPER_PACKAGE, _TOKEN_WRAPPER_COMPONENT};
pub use self::errors::*;

#[blueprint]
mod token_wrapper_2_mod {
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;
    const TOKEN_WRAPPER_COMPONENT: ComponentAddress = _TOKEN_WRAPPER_COMPONENT;

    extern_blueprint! {
        TOKEN_WRAPPER_PACKAGE,
        TokenWrapper {
            // Constructor
            // fn new(owner_role: OwnerRole, authority_token: Bucket) -> Global<TokenWrapper>;

            // Owner protected methods
            // fn deposit_authority(&self, token: Bucket);
            // fn withdraw_authority(&mut self) -> Bucket;
            // fn add_input(&mut self, resource: ResourceAddress);
            // fn update_wrappable(&mut self, child_resource: ResourceAddress, wrappable: bool);

            // Getter methods
            fn get_inputs(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ResourceAddress, bool, Decimal)>;

            // Authority protected methods
            fn wrap(&self, child_token: Bucket) -> Bucket;
            fn unwrap(&self, parent_token: Bucket, child_resource: ResourceAddress) -> Bucket;
            fn flash_loan(&self, amount: Decimal) -> (Bucket, Bucket);
            fn repay_flash_loan(&self, flash_oath: Bucket, token: Bucket) -> Bucket;
        }
    }

    enable_method_auth!(
        roles {
            admin => updatable_by: [OWNER];
            wrap_user => updatable_by: [OWNER];
            unwrap_user => updatable_by: [OWNER];
            flash_user => updatable_by: [OWNER];
        },
        methods {
            get_inputs => PUBLIC;
            
            wrap => restrict_to: [wrap_user];
            unwrap => restrict_to: [unwrap_user];
            flash_loan => restrict_to: [flash_user];
            repay_flash_loan => restrict_to: [flash_user];
        }
    );

    struct TokenWrapper2 {}

    impl TokenWrapper2 {
        pub fn new(owner_role: OwnerRole) -> Global<TokenWrapper2> {
            let (component_reservation, _this) = Runtime::allocate_component_address(TokenWrapper2::blueprint_id());

            Global::<TokenWrapper>::try_from(TOKEN_WRAPPER_COMPONENT).expect("TokenWrapper component is not a valid TokenWrapper");

            Self {}
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

        pub fn get_inputs(&self, n: ListIndex, start: Option<ListIndex>) -> Vec<(ResourceAddress, bool, Decimal)> {
            Global::<TokenWrapper>::from(TOKEN_WRAPPER_COMPONENT).get_inputs(n, start)
        }

        pub fn wrap(&mut self, child_token: Bucket) -> Bucket {
            Global::<TokenWrapper>::from(TOKEN_WRAPPER_COMPONENT).wrap(child_token)
        }

        pub fn unwrap(&mut self, parent_token: Bucket, child_resource: ResourceAddress) -> Bucket {
            assert!(
                parent_token.resource_address() == BASE_RESOURCE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_INVALID_PAYMENT_TOKEN, Runtime::bech32_encode_address(parent_token.resource_address()), Runtime::bech32_encode_address(BASE_RESOURCE)
            );

            Global::<TokenWrapper>::from(TOKEN_WRAPPER_COMPONENT).unwrap(parent_token, child_resource)
        }

        pub fn flash_loan(&mut self, amount: Decimal) -> (Bucket, Bucket) {
            Global::<TokenWrapper>::from(TOKEN_WRAPPER_COMPONENT).flash_loan(amount)
        }

        pub fn repay_flash_loan(&mut self, flash_oath: Bucket, token: Bucket) -> Bucket {
            Global::<TokenWrapper>::from(TOKEN_WRAPPER_COMPONENT).repay_flash_loan(flash_oath, token)
        }
    }
}
