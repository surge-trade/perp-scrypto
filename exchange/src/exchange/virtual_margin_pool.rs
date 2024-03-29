use scrypto::prelude::*;
use pool::*;
use utils::PairId;
use super::events::*;
use super::exchange::MarginPool;

pub struct VirtualLiquidityPool {
    pool: Global<MarginPool>,
    pool_info: MarginPoolInfo,
    pool_updates: MarginPoolUpdates,
}

impl VirtualLiquidityPool {
    pub fn new(pool: Global<MarginPool>) -> Self {
        let pool_info = pool.get_info();

        Self {
            pool,
            pool_updates: MarginPoolUpdates {
                position_updates: HashMap::new(),
                virtual_balance: pool_info.virtual_balance,
                unrealized_pool_funding: pool_info.unrealized_pool_funding,
                skew_abs_snap: pool_info.skew_abs_snap,
                pnl_snap: pool_info.pnl_snap,
            },
            pool_info,
        }
    }

    pub fn realize(self) {
        let current_time = Clock::current_time_rounded_to_seconds();
        let updates: Vec<(PairId, PoolPosition)> = self.pool_updates.position_updates.iter()
            .map(|(pair_id, position)| (*pair_id, position.clone())).collect();
        if !updates.is_empty() {
            let event_pair_updates = EventPairUpdates {
                time: current_time,
                updates,
            };
            Runtime::emit_event(event_pair_updates);
        }

        self.pool.update(self.pool_updates);
    }

    pub fn position(&self, pair_id: PairId) -> PoolPosition {
        if let Some(position) = self.pool_updates.position_updates.get(&pair_id) {
            position.clone()
        } else if let Some(position) = self.pool.get_position(pair_id) {
            position
        } else {
            PoolPosition {
                oi_long: dec!(0),
                oi_short: dec!(0),
                cost: dec!(0),
                skew_abs_snap: dec!(0),
                pnl_snap: dec!(0),
                funding_2_rate: dec!(0),
                funding_long_index: dec!(0),
                funding_short_index: dec!(0),
                last_update: Clock::current_time_rounded_to_seconds(),
                last_price: dec!(1),
            }
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

    pub fn update_position(&mut self, pair_id: PairId, position: PoolPosition) {
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