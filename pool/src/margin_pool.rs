mod structs;

use scrypto::prelude::*;
use common::{PairId, _AUTHORITY_RESOURCE, _BASE_RESOURCE};
pub use self::structs::*;

#[blueprint]
#[types(
    PairId,
    PoolPosition,
)]
pub mod margin_pool {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_RESOURCE: ResourceAddress = _BASE_RESOURCE;

    enable_method_auth! { 
        roles {
            authority => updatable_by: [];
        },
        methods { 
            // Get methods
            get_info => PUBLIC;
            get_position => PUBLIC;
            get_positions => PUBLIC;

            // Authority protected methods
            update => restrict_to: [authority];
            deposit => restrict_to: [authority];
            withdraw => restrict_to: [authority];
        }
    }

    struct MarginPool {
        positions: KeyValueStore<PairId, PoolPosition>,
        base_tokens: Vault,
        virtual_balance: Decimal,
        unrealized_pool_funding: Decimal,
        skew_abs_snap: Decimal,
        pnl_snap: Decimal,
        lp_token_manager: ResourceManager,
    }

    impl MarginPool {
        pub fn new(owner_role: OwnerRole) -> Global<MarginPool> {
            let (component_reservation, this) = Runtime::allocate_component_address(MarginPool::blueprint_id());

            let lp_token_manager = ResourceBuilder::new_fungible(owner_role.clone())
                .metadata(metadata!(
                    init {
                        "package" => GlobalAddress::from(Runtime::package_address()), locked;
                        "component" => GlobalAddress::from(this), locked;
                        "name" => format!("LP Token"), updatable;
                        "symbol" => format!("LPT"), updatable;
                        "description" => format!("Liquidity provider token the represents a share of ownership of a pool."), updatable;
                    }
                ))
                .mint_roles(mint_roles!{
                        minter => rule!(require(AUTHORITY_RESOURCE)); 
                        minter_updater => rule!(deny_all);
                    }
                )
                .burn_roles(burn_roles!{
                        burner => rule!(require(AUTHORITY_RESOURCE)); 
                        burner_updater => rule!(deny_all);
                    }
                )
                .create_with_no_initial_supply();

            Self {
                positions: KeyValueStore::new_with_registered_type(),
                base_tokens: Vault::new(BASE_RESOURCE),
                virtual_balance: dec!(0),
                unrealized_pool_funding: dec!(0),
                skew_abs_snap: dec!(0),
                pnl_snap: dec!(0),
                lp_token_manager,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
            })
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self, pair_ids: HashSet<PairId>) -> MarginPoolInfo {
            MarginPoolInfo {
                positions: self.get_positions(pair_ids),
                base_tokens_amount: self.base_tokens.amount(),
                virtual_balance: self.virtual_balance,
                unrealized_pool_funding: self.unrealized_pool_funding,
                skew_abs_snap: self.skew_abs_snap,
                pnl_snap: self.pnl_snap,
                lp_token_manager: self.lp_token_manager,
            }
        }

        pub fn get_position(&self, pair_id: PairId) -> Option<PoolPosition> {
            self.positions.get(&pair_id).map(|position| position.clone())
        }

        pub fn get_positions(&self, pair_ids: HashSet<PairId>) -> HashMap<PairId, Option<PoolPosition>> {
            pair_ids.into_iter().map(|k| (k, self.positions.get(&k).map(|v| v.clone()))).collect()
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
    }
}