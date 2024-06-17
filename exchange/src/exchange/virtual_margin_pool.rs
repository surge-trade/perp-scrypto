use scrypto::prelude::*;
use pool::*;
use common::PairId;
use super::errors::*;
use super::events::*;
use super::exchange_mod::MarginPool;

pub struct VirtualLiquidityPool {
    pool: Global<MarginPool>,
    positions: HashMap<PairId, PoolPosition>,
    position_updates: HashSet<PairId>,
    base_tokens_amount: Decimal,
    virtual_balance: Decimal,
    unrealized_pool_funding: Decimal,
    skew_abs_snap: Decimal,
    pnl_snap: Decimal,
}

impl VirtualLiquidityPool {
    pub fn new(pool: Global<MarginPool>, pair_ids: HashSet<PairId>) -> Self {
        let pool_info = pool.get_info(pair_ids);

        Self {
            pool,
            positions: pool_info.positions,
            position_updates: HashSet::new(),
            base_tokens_amount: pool_info.base_tokens_amount,
            virtual_balance: pool_info.virtual_balance,
            unrealized_pool_funding: pool_info.unrealized_pool_funding,
            skew_abs_snap: pool_info.skew_abs_snap,
            pnl_snap: pool_info.pnl_snap,
        }
    }

    pub fn realize(self) {
        let pool_updates = MarginPoolUpdates {
            position_updates: self.positions.into_iter().filter(|(pair_id, _)| self.position_updates.contains(pair_id)).collect(),
            virtual_balance: self.virtual_balance,
            unrealized_pool_funding: self.unrealized_pool_funding,
            skew_abs_snap: self.skew_abs_snap,
            pnl_snap: self.pnl_snap,
        };

        let updates: Vec<(PairId, PoolPosition)> = pool_updates.position_updates.iter()
            .map(|(pair_id, position)| (pair_id.clone(), position.clone())).collect();
        if !updates.is_empty() {
            let event_pair_updates = EventPairUpdates {
                updates,
            };
            Runtime::emit_event(event_pair_updates);
        }

        self.pool.update(pool_updates);
    }

    pub fn position(&self, pair_id: &PairId) -> &PoolPosition {
        self.positions.get(pair_id).expect(ERROR_MISSING_POOL_POSITION)
    }

    pub fn position_mut(&mut self, pair_id: &PairId) -> &mut PoolPosition {
        self.position_updates.insert(pair_id.clone());
        self.positions.get_mut(pair_id).expect(ERROR_MISSING_POOL_POSITION)
    }

    pub fn base_tokens_amount(&self) -> Decimal {
        self.base_tokens_amount
    }

    pub fn virtual_balance(&self) -> Decimal {
        self.virtual_balance
    }

    pub fn unrealized_pool_funding(&self) -> Decimal {
        self.unrealized_pool_funding
    }

    pub fn skew_abs_snap(&self) -> Decimal {
        self.skew_abs_snap
    }

    pub fn pnl_snap(&self) -> Decimal {
        self.pnl_snap
    }

    pub fn deposit(&mut self, token: Bucket) {
        self.base_tokens_amount += token.amount();
        self.pool.deposit(token);
    }

    pub fn withdraw(&mut self, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        self.base_tokens_amount -= amount;
        self.pool.withdraw(amount, withdraw_strategy)
    }

    pub fn add_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.virtual_balance += virtual_balance;
    }

    pub fn add_unrealized_pool_funding(&mut self, unrealized_pool_funding: Decimal) {
        self.unrealized_pool_funding += unrealized_pool_funding;
    }

    pub fn add_skew_abs_snap(&mut self, skew_abs_snap: Decimal) {
        self.skew_abs_snap += skew_abs_snap;
    }

    pub fn add_pnl_snap(&mut self, pnl_snap: Decimal) {
        self.pnl_snap += pnl_snap;
    }
}