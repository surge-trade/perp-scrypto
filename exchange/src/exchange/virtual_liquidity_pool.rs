use scrypto::prelude::*;
use super::errors::*;
use super::liquidity_pool::*;
use super::liquidity_pool::liquidity_pool::LiquidityPool;

pub struct VirtualLiquidityPool {
    pool: Global<LiquidityPool>,
    pool_info: LiquidityPoolInfo,
    pool_updates: LiquidityPoolUpdates,
}

impl VirtualLiquidityPool {
    pub fn new(pool: ComponentAddress) -> Self {
        let pool = Global::<LiquidityPool>::try_from(pool).expect(ERROR_INVALID_POOL);
        let pool_info = pool.get_info();

        Self {
            pool,
            pool_updates: LiquidityPoolUpdates {
                position_updates: HashMap::new(),
                virtual_balance: pool_info.virtual_balance,
                unrealized_pool_funding: pool_info.unrealized_pool_funding,
                skew_abs_snap: pool_info.skew_abs_snap,
                pnl_snap: pool_info.pnl_snap,
            },
            pool_info,
        }
    }

    pub fn position(&self, pair_id: u64) -> PoolPosition {
        if let Some(position) = self.pool_updates.position_updates.get(&pair_id) {
            position.clone()
        } else {
            self.pool.get_position(pair_id).expect(ERROR_MISSING_POOL_POSITION)
        }
    }

    pub fn base_tokens_amount(&self) -> Decimal {
        self.pool_info.base_tokens_amount
    }

    pub fn virtual_balance(&self) -> Decimal {
        self.pool_info.virtual_balance
    }

    pub fn unrealized_pool_funding(&self) -> Decimal {
        self.pool_info.unrealized_pool_funding
    }

    pub fn skew_abs_snap(&self) -> Decimal {
        self.pool_info.skew_abs_snap
    }

    pub fn pnl_snap(&self) -> Decimal {
        self.pool_info.pnl_snap
    }

    pub fn lp_token_manager(&self) -> ResourceManager {
        self.pool_info.lp_token_manager
    }

    pub fn realize(self) {
        self.pool.update(self.pool_updates);
    }

    pub fn deposit(&mut self, token: Bucket) {
        self.pool_info.base_tokens_amount += token.amount();
        self.pool.deposit(token);
    }

    pub fn withdraw(&mut self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        self.pool_info.base_tokens_amount -= amount;
        self.pool.withdraw(amount, withdraw_strategy)
    }

    pub fn mint_lp(&mut self, amount: Decimal) -> Bucket {
        self.pool_info.virtual_balance += amount;
        self.pool.mint_lp(amount)
    }

    pub fn burn_lp(&mut self, token: Bucket) {
        self.pool_info.virtual_balance -= token.amount();
        self.pool.burn_lp(token);
    }

    pub fn update_position(&mut self, pair_id: u64, position: PoolPosition) {
        self.pool_updates.position_updates.insert(pair_id, position);
    }

    pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.pool_info.virtual_balance = virtual_balance;
        self.pool_updates.virtual_balance = virtual_balance;
    }

    pub fn update_unrealized_pool_funding(&mut self, unrealized_pool_funding: Decimal) {
        self.pool_info.unrealized_pool_funding = unrealized_pool_funding;
        self.pool_updates.unrealized_pool_funding = unrealized_pool_funding;
    }

    pub fn update_skew_abs_snap(&mut self, skew_abs_snap: Decimal) {
        self.pool_info.skew_abs_snap = skew_abs_snap;
        self.pool_updates.skew_abs_snap = skew_abs_snap;
    }

    pub fn update_pnl_snap(&mut self, pnl_snap: Decimal) {
        self.pool_info.pnl_snap = pnl_snap;
        self.pool_updates.pnl_snap = pnl_snap;
    }
}