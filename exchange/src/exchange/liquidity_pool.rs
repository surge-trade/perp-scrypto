use scrypto::prelude::*;
use super::consts::BASE_RESOURCE;

#[derive(ScryptoSbor)]
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
pub struct LiquidityPool {
    pub base_tokens: Vault,
    pub virtual_balance: Decimal,
    pub positions: KeyValueStore<u64, PoolPosition>,
    pub unrealized_pool_funding: Decimal,
    pub skew_abs_snap: Decimal,
    pub pnl_snap: Decimal,
    pub lp_token_manager: ResourceManager,
}

impl LiquidityPool {
    pub fn new(this: ComponentAddress, owner_role: OwnerRole) -> Self {
        let lp_token_manager = ResourceBuilder::new_fungible(owner_role)
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
    }
}
