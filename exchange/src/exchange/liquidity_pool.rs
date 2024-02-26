use scrypto::prelude::*;
use super::consts::BASE_RESOURCE;
use super::errors::*;
use self::liquidity_pool::LiquidityPool;

#[derive(ScryptoSbor, Clone)]
pub struct PoolPosition {
    pub oi_long: Decimal,
    pub oi_short: Decimal,
    pub cost: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
    pub funding_2_rate: Decimal,
    pub funding_long_index: Decimal,
    pub funding_short_index: Decimal,
    pub last_update: Instant,
}

#[derive(ScryptoSbor)]
pub struct LiquidityPoolInfo {
    pub base_tokens_amount: Decimal,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
    pub lp_token_manager: ResourceManager,
}

#[derive(ScryptoSbor)]
pub struct LiquidityPoolUpdates {
    pub position_updates: HashMap<u64, PoolPosition>,
    pub virtual_balance: Decimal,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
}

#[blueprint]
pub mod liquidity_pool {
    extern_blueprint! {
        "package_sim1pkyls09c258rasrvaee89dnapp2male6v6lmh7en5ynmtnavqdsvk9",
        Authority {
            fn get_rule(&self) -> AccessRule;
        }
    }

    const AUTHORITY: Global<Authority> = global_component!(
        Authority,
        "component_sim1czc0e8f9yhlvpv38s2ymrplu7q366y3k8zc53zf2srlm7qm604g029"
    );

    struct LiquidityPool {
        base_tokens: Vault,
        virtual_balance: Decimal,
        positions: KeyValueStore<u64, PoolPosition>,
        unrealized_pool_funding: Decimal,
        skew_abs_snap: Decimal,
        pnl_snap: Decimal,
        lp_token_manager: ResourceManager,
    }

    impl LiquidityPool {
        pub fn new(owner_role: OwnerRole) -> Global<LiquidityPool> {
            let (component_reservation, this) = Runtime::allocate_component_address(LiquidityPool::blueprint_id());

            let lp_token_manager = ResourceBuilder::new_fungible(owner_role.clone())
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
            .prepare_to_globalize(owner_role)
            .with_address(component_reservation)
            .globalize()
        }

        pub fn get_info(&self) -> LiquidityPoolInfo {
            LiquidityPoolInfo {
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

        pub fn update(&mut self, updates: LiquidityPoolUpdates) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for (position_id, position) in updates.position_updates {
                self.positions.insert(position_id, position);
            }

            self.virtual_balance = updates.virtual_balance;
            self.unrealized_pool_funding = updates.unrealized_pool_funding;
            self.skew_abs_snap = updates.skew_abs_snap;
            self.pnl_snap = updates.pnl_snap;
        }

        pub fn deposit(&mut self, token: Bucket) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.base_tokens.put(token);
        }

        pub fn withdraw(&mut self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.base_tokens.take_advanced(amount, withdraw_strategy)
        }

        pub fn mint_lp(&mut self, amount: Decimal) -> Bucket {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.lp_token_manager.mint(amount)
        }

        pub fn burn_lp(&mut self, token: Bucket) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            token.burn();
        }
    }
}