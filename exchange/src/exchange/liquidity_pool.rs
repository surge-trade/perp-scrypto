pub mod pool_position;
pub use self::pool_position::PoolPosition;

use scrypto::prelude::*;
use crate::utils::Vaults;

#[derive(ScryptoSbor)]
pub struct LiquidityPool {
    pub vaults: Vaults,
    pub positions: HashMap<ResourceAddress, PoolPosition>,
    pub unrealized_borrowing: Decimal,
    pub last_update: Instant,
    pub lp_token_manager: ResourceManager,
}

impl LiquidityPool {
    pub fn new(resources: Vec<ResourceAddress>, this: ComponentAddress, owner_role: OwnerRole) -> Self {
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
            vaults: Vaults::new(),
            positions: resources.into_iter().map(|r| (r, PoolPosition::default())).collect(),
            unrealized_borrowing: dec!(0),
            last_update: Clock::current_time_rounded_to_minutes(),
            lp_token_manager,
        }
    }
}
