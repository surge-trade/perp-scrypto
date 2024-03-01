use scrypto::prelude::*;
use account::*;
use super::errors::*;
// use super::margin_account::*;
use super::exchange::MarginAccount;
use super::requests::*;

pub struct VirtualMarginAccount {
    account: Global<MarginAccount>,
    account_info: MarginAccountInfo,
    account_updates: MarginAccountUpdates,
}

impl VirtualMarginAccount {
    pub fn new(account: ComponentAddress, collateral_resources: Vec<ResourceAddress>) -> Self {
        let account = Global::<MarginAccount>::try_from(account).expect(ERROR_INVALID_MARGIN_ACCOUNT);
        let account_info = account.get_info(collateral_resources);

        Self {
            account,
            account_updates: MarginAccountUpdates {
                position_updates: HashMap::new(),
                virtual_balance: account_info.virtual_balance,
            },
            account_info,
        }
    }

    pub fn realize(self) {
        self.account.update(self.account_updates);
    }

    pub fn positions(&self) -> &HashMap<u64, AccountPosition> {
        &self.account_info.positions
    }

    pub fn position(&self, pair_id: u64) -> AccountPosition {
        self.account_info.positions.get(&pair_id).cloned().unwrap_or_default()
    }

    pub fn position_amount(&self, pair_id: u64) -> Decimal {
        self.account_info.positions.get(&pair_id).map(|position| position.amount).unwrap_or_default()
    }

    pub fn collateral_balances(&self) -> &HashMap<ResourceAddress, Decimal> {
        &self.account_info.collateral_balances
    }

    pub fn collateral_amount(&self, resource: &ResourceAddress) -> Decimal {
        self.account_info.collateral_balances.get(resource).copied().unwrap_or_default()
    }

    pub fn virtual_balance(&self) -> Decimal {
        self.account_info.virtual_balance
    }

    pub fn process_request(&mut self, index: u64) -> Request {
        let keeper_request = self.account.process_request(index).expect(ERROR_MISSING_REQUEST);
        assert!(
            !keeper_request.processed,
            "{}", ERROR_REQUEST_ALREADY_PROCESSED
        );
        assert!(
            Clock::current_time_is_strictly_before(keeper_request.expiry, TimePrecision::Second),
            "{}", ERROR_REQUEST_EXPIRED
        );
        let request = Request::decode(&keeper_request.data);
        request
    }

    pub fn deposit_collateral(&mut self, token: Bucket) {
        let amount = token.amount();
        let resource = token.resource_address();
        self.account_info.collateral_balances
            .entry(resource)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);

        self.account.deposit_collateral(token);
    }

    pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
        for token in tokens.iter() {
            let amount = token.amount();
            let resource = token.resource_address();
            self.account_info.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);
        }

        self.account.deposit_collateral_batch(tokens);
    }

    pub fn withdraw_collateral(&mut self, resource: &ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        self.account_info.collateral_balances
            .entry(*resource)
            .and_modify(|balance| *balance -= amount)
            .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));

        self.account.withdraw_collateral(*resource, amount, withdraw_strategy)
    }

    pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket>  {
        for (resource, amount) in claims.iter() {
            self.account_info.collateral_balances
                .entry(*resource)
                .and_modify(|balance| *balance -= *amount)
                .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));
        }
        self.account_info.collateral_balances.retain(|_, &mut balance| balance != dec!(0));

        self.account.withdraw_collateral_batch(claims, withdraw_strategy)
    }

    pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.account_info.virtual_balance = virtual_balance;
        self.account_updates.virtual_balance = virtual_balance;
    }

    pub fn update_position(&mut self, pair_id: u64, position: AccountPosition) {
        self.account_info.positions.insert(pair_id, position.clone());
        self.account_updates.position_updates.insert(pair_id, position);
    }

    pub fn remove_position(&mut self, pair_id: u64) {
        self.account_info.positions.remove(&pair_id);
        self.account_updates.position_updates.insert(pair_id, AccountPosition::default());
    }
}