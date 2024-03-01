mod consts;
pub mod structs;

use scrypto::prelude::*;
use self::consts::*;
pub use self::structs::*;

#[blueprint]
pub mod margin_pool {
    // Set access rules
    enable_method_auth! { 
        roles {
            authority => updatable_by: [];
        },
        methods { 
            get_info => PUBLIC;
            get_position => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            deposit => restrict_to: [authority];
            withdraw => restrict_to: [authority];
            mint_lp => restrict_to: [authority];
            burn_lp => restrict_to: [authority];
        }
    }

    struct MarginPool {
        base_tokens: Vault,
        virtual_balance: Decimal,
        positions: KeyValueStore<u64, PoolPosition>,
        unrealized_pool_funding: Decimal,
        skew_abs_snap: Decimal,
        pnl_snap: Decimal,
        lp_token_manager: ResourceManager,
    }

    impl MarginPool {
        pub fn new(owner_rule: AccessRule) -> Global<MarginPool> {
            let (component_reservation, this) = Runtime::allocate_component_address(MarginPool::blueprint_id());

            let lp_token_manager = ResourceBuilder::new_fungible(OwnerRole::Updatable(owner_rule.clone()))
                .metadata(metadata!(
                    init {
                        "package" => GlobalAddress::from(Runtime::package_address()), locked;
                        "component" => GlobalAddress::from(this), locked;
                        "name" => format!("LP Token"), updatable;
                        "description" => format!("Liquidity provider token the represents a share of ownership of a pool."), updatable;
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
                base_tokens: Vault::new(BASE_RESOURCE),
                virtual_balance: dec!(0),
                positions: KeyValueStore::new(),
                unrealized_pool_funding: dec!(0),
                skew_abs_snap: dec!(0),
                pnl_snap: dec!(0),
                lp_token_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(owner_rule))
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self) -> MarginPoolInfo {
            MarginPoolInfo {
                base_tokens_amount: self.base_tokens.amount(),
                virtual_balance: self.virtual_balance,
                unrealized_pool_funding: self.unrealized_pool_funding,
                skew_abs_snap: self.skew_abs_snap,
                pnl_snap: self.pnl_snap,
                lp_token_manager: self.lp_token_manager,
            }
        }

        pub fn get_position(&self, position_id: u64) -> Option<PoolPosition> {
            self.positions.get(&position_id).map(|position| position.clone())
        }

        pub fn update(&mut self, update: MarginPoolUpdates) {
            for (position_id, position) in update.position_updates {
                self.positions.insert(position_id, position);
            }

            self.virtual_balance = update.virtual_balance;
            self.unrealized_pool_funding = update.unrealized_pool_funding;
            self.skew_abs_snap = update.skew_abs_snap;
            self.pnl_snap = update.pnl_snap;
        }

        pub fn deposit(&mut self, token: Bucket) {
            self.base_tokens.put(token);
        }

        pub fn withdraw(&mut self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            self.base_tokens.take_advanced(amount, withdraw_strategy)
        }

        pub fn mint_lp(&mut self, amount: Decimal) -> Bucket {
            self.lp_token_manager.mint(amount)
        }

        pub fn burn_lp(&mut self, token: Bucket) {
            token.burn();
        }
    }
}