use scrypto::prelude::*;
use account::*;
use super::errors::*;
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
                last_liquidation: account_info.last_liquidation,
            },
            account_info,
        }
    }

    pub fn realize(self) {
        self.account.update(self.account_updates);
    }

    pub fn address(&self) -> ComponentAddress {
        self.account.address()
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

    pub fn last_liquidation(&self) -> Instant {
        self.account_info.last_liquidation
    }

    pub fn verify_level_1_auth(&self) {
        let rule = self.account.get_role("level_1").expect(ERROR_MISSING_AUTH);
        Runtime::assert_access_rule(rule);
    }

    pub fn verify_level_2_auth(&self) {
        let rule = self.account.get_role("level_2").expect(ERROR_MISSING_AUTH);
        Runtime::assert_access_rule(rule);
    }

    pub fn verify_level_3_auth(&self) {
        let rule = self.account.get_role("level_3").expect(ERROR_MISSING_AUTH);
        Runtime::assert_access_rule(rule);
    }

    pub fn set_level_1_auth(&self, rule: AccessRule) {
        self.account.set_role("level_1", rule);
    }

    pub fn set_level_2_auth(&self, rule: AccessRule) {
        self.account.set_role("level_2", rule);
    }

    pub fn set_level_3_auth(&self, rule: AccessRule) {
        self.account.set_role("level_3", rule);
    }

    pub fn push_request(&mut self, request: Request, expiry_seconds: u64, status: u8) {
        assert!(
            status == STATUS_DORMANT || status == STATUS_ACTIVE,
            "{}", ERROR_INVALID_REQUEST_STATUS
        );

        let submission = Clock::current_time_rounded_to_seconds();
        let expiry = submission.add_seconds(expiry_seconds as i64).expect(ERROR_ARITHMETIC);

        let keeper_request = KeeperRequest {
            request: request.encode(), 
            submission,
            expiry,
            status,
        };

        self.account.push_request(keeper_request);
    }

    fn _assert_request_status_change(&self, status1: u8, status0: Option<u8>) {
        match status1 {
            STATUS_ACTIVE => assert!(status0 == Some(STATUS_DORMANT), "{}", ERROR_REQUEST_NOT_DORMANT),
            STATUS_EXECUTED => assert!(status0 == Some(STATUS_ACTIVE), "{}", ERROR_REQUEST_NOT_ACTIVE),
            STATUS_CANCELLED => assert!(status0 == Some(STATUS_ACTIVE), "{}", ERROR_REQUEST_NOT_ACTIVE),
            _ => panic!("{}", ERROR_INVALID_REQUEST_STATUS),
        }
    }

    pub fn set_request_status(&mut self, index: u64, status: u8) {
        let updated = self.account.set_request_status(index, status);
        self._assert_request_status_change(status, updated);
    }

    pub fn set_request_statuses(&mut self, updates: Vec<(u64, u8)>) {
        let updated = self.account.set_request_statuses(updates.clone());
        for ((_, status1), status0) in updates.iter().zip(updated.iter()) {
            self._assert_request_status_change(*status1, *status0);
        }
    }

    pub fn process_request(&mut self, index: u64) -> (Request, Instant) {
        let keeper_request = self.account.process_request(index, STATUS_EXECUTED).expect(ERROR_MISSING_REQUEST);
        let current_time = Clock::current_time_rounded_to_seconds();
        assert!(
            keeper_request.status == STATUS_ACTIVE,
            "{}", ERROR_REQUEST_NOT_ACTIVE
        );
        assert!(
            current_time.compare(keeper_request.expiry, TimeComparisonOperator::Lt),
            "{}", ERROR_REQUEST_EXPIRED
        );
        assert!(
            current_time.compare(self.last_liquidation(), TimeComparisonOperator::Gt),
            "{}", ERROR_REQUEST_BEFORE_LIQUIDATION
        );
        let request = Request::decode(&keeper_request.request);
        (request, keeper_request.submission)
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

    pub fn update_position(&mut self, pair_id: u64, position: AccountPosition) {
        self.account_info.positions.insert(pair_id, position.clone());
        self.account_updates.position_updates.insert(pair_id, position);
    }

    pub fn remove_position(&mut self, pair_id: u64) {
        self.account_info.positions.remove(&pair_id);
        self.account_updates.position_updates.insert(pair_id, AccountPosition::default());
    }

    pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.account_info.virtual_balance = virtual_balance;
        self.account_updates.virtual_balance = virtual_balance;
    }

    pub fn update_last_liquidation(&mut self) {
        let last_liquidation = Clock::current_time_rounded_to_seconds();
        self.account_info.last_liquidation = last_liquidation;
        self.account_updates.last_liquidation = last_liquidation;
    }
}