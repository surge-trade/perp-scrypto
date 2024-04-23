use scrypto::prelude::*;
use account::*;
use common::{PairId, ListIndex};
use super::errors::*;
use super::events::*;
use super::exchange_mod::MarginAccount;
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
                valid_requests_start: account_info.valid_requests_start,
                requests_new: vec![],
                request_updates: HashMap::new(),
            },
            account_info,
        }
    }

    pub fn realize(self) {
        let requests: Vec<(ListIndex, KeeperRequest)> = self.account_updates.requests_new.iter()
            .enumerate().map(|(i, request)| (i as ListIndex + self.account_info.requests_len, request.clone()))
            .chain(self.account_updates.request_updates.iter().map(|(index, request)| (*index, request.clone())))
            .collect();
        if !requests.is_empty() {
            let event_requests = EventRequests {
                account: self.account.address(),
                requests,
            };
            Runtime::emit_event(event_requests);
        }

        self.account.update(self.account_updates);
    }

    pub fn address(&self) -> ComponentAddress {
        self.account.address()
    }

    pub fn positions(&self) -> &HashMap<PairId, AccountPosition> {
        &self.account_info.positions
    }

    pub fn position_ids(&self) -> HashSet<PairId> {
        self.account_info.positions.keys().cloned().collect()
    }

    pub fn position(&self, pair_id: PairId) -> AccountPosition {
        self.account_info.positions.get(&pair_id).cloned().unwrap_or_default()
    }

    // pub fn position_amount(&self, pair_id: PairId) -> Decimal {
    //     self.account_info.positions.get(&pair_id).map(|position| position.amount).unwrap_or_default()
    // }

    pub fn keeper_request(&self, index: ListIndex) -> KeeperRequest {
        if let Some(request) = self.account_updates.request_updates.get(&index) {
            request.clone()
        } else {
            self.account.get_request(index).expect(ERROR_MISSING_REQUEST)
        }
    }

    pub fn keeper_requests(&self, indexes: Vec<ListIndex>) -> HashMap<ListIndex, KeeperRequest> {
        let mut requests = HashMap::new();
        let indexes = indexes.into_iter().filter(|&index| {
            if let Some(request) = self.account_updates.request_updates.get(&index) {
                requests.insert(index, request.clone());
                false
            } else {
                true
            }
        }).collect();

        let requests_fetched: HashMap<ListIndex, KeeperRequest> = self.account.get_requests_by_indexes(indexes)
            .into_iter().map(|(index, request)| (index, request.expect(ERROR_MISSING_REQUEST))).collect();
        requests.extend(requests_fetched);
        requests
    }

    // pub fn collateral_balances(&self) -> &HashMap<ResourceAddress, Decimal> {
    //     &self.account_info.collateral_balances
    // }

    pub fn collateral_amount(&self, resource: &ResourceAddress) -> Decimal {
        self.account_info.collateral_balances.get(resource).copied().unwrap_or_default()
    }

    pub fn virtual_balance(&self) -> Decimal {
        self.account_info.virtual_balance
    }

    // pub fn requests_len(&self) -> ListIndex {
    //     self.account_info.requests_len
    // }

    pub fn valid_requests_start(&self) -> ListIndex {
        self.account_info.valid_requests_start
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

    pub fn set_level_1_auth(&mut self, rule: AccessRule) {
        self.account.set_role("level_1", rule);
    }

    pub fn set_level_2_auth(&mut self, rule: AccessRule) {
        self.account.set_role("level_2", rule);
    }

    pub fn set_level_3_auth(&mut self, rule: AccessRule) {
        self.account.set_role("level_3", rule);
    }

    pub fn push_request(&mut self, request: Request, expiry_seconds: u64, status: Status) {
        assert!(
            status == STATUS_DORMANT || status == STATUS_ACTIVE,
            "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_INVALID_REQUEST_STATUS, status, vec![STATUS_DORMANT, STATUS_ACTIVE]
        );

        let submission = Clock::current_time_rounded_to_seconds();
        let expiry = submission.add_seconds(expiry_seconds as i64).expect(ERROR_ARITHMETIC);

        let keeper_request = KeeperRequest {
            request: request.encode(), 
            submission,
            expiry,
            status,
        };

        self.account_updates.requests_new.push(keeper_request);
    }

    fn _status_phases(&self, status: Status) -> Vec<Status> {
        match status {
            STATUS_ACTIVE => vec![STATUS_DORMANT],
            STATUS_EXECUTED => vec![STATUS_ACTIVE],
            STATUS_CANCELLED => vec![STATUS_ACTIVE, STATUS_DORMANT],
            _ => panic!("{}", ERROR_INVALID_REQUEST_STATUS),
        }
    }

    pub fn try_set_keeper_requests_status(&mut self, indexes: Vec<ListIndex>, status: Status) -> Vec<ListIndex> {
        let current_time = Clock::current_time_rounded_to_seconds();

        let status_phases = self._status_phases(status);
        let keeper_requests = self.keeper_requests(indexes);
        let mut updated = vec![];
        for (index, keeper_request) in keeper_requests.into_iter() {
            if !status_phases.contains(&keeper_request.status) {
                continue;
            } else {
                let mut keeper_request = keeper_request;
                keeper_request.status = status;
                keeper_request.submission = current_time;
                self.account_updates.request_updates.insert(index, keeper_request);
                updated.push(index);
            }
        }

        updated
    }

    pub fn cancel_request(&mut self, index: ListIndex) {
        let current_time = Clock::current_time_rounded_to_seconds();
        let mut keeper_request = self.keeper_request(index);
        let status_phases = self._status_phases(STATUS_CANCELLED);
        assert!(
            status_phases.contains(&keeper_request.status),
            "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_CANCEL_REQUEST_NOT_ACTIVE_OR_DORMANT, keeper_request.status, status_phases
        );
        keeper_request.status = STATUS_CANCELLED;
        keeper_request.submission = current_time;
        self.account_updates.request_updates.insert(index, keeper_request);
    }

    pub fn process_request(&mut self, index: ListIndex) -> (Request, Instant) {
        assert!(
            index >= self.valid_requests_start(),
            "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_PROCESS_REQUEST_BEFORE_VALID_START, index, self.valid_requests_start()
        );
        
        let current_time = Clock::current_time_rounded_to_seconds();
        let mut keeper_request = self.keeper_request(index);
        assert!(
            keeper_request.status == STATUS_ACTIVE,
            "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_PROCESS_REQUEST_NOT_ACTIVE, keeper_request.status, STATUS_ACTIVE
        );
        assert!(
            current_time.compare(keeper_request.expiry, TimeComparisonOperator::Lt),
            "{}, VALUE:{}, REQUIRED:{}, OP:< |", ERROR_PROCESS_REQUEST_EXPIRED, current_time.seconds_since_unix_epoch, keeper_request.expiry.seconds_since_unix_epoch
        );
        let submission = keeper_request.submission;
        let request = Request::decode(&keeper_request.request);

        keeper_request.status = STATUS_EXECUTED;
        keeper_request.submission = current_time;
        self.account_updates.request_updates.insert(index, keeper_request);

        (request, submission)
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

    pub fn update_position(&mut self, pair_id: PairId, position: AccountPosition) {
        self.account_info.positions.insert(pair_id, position.clone());
        self.account_updates.position_updates.insert(pair_id, position);
    }

    pub fn remove_position(&mut self, pair_id: PairId) {
        self.account_info.positions.remove(&pair_id);
        self.account_updates.position_updates.insert(pair_id, AccountPosition::default());
    }

    pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.account_info.virtual_balance = virtual_balance;
        self.account_updates.virtual_balance = virtual_balance;
    }

    pub fn update_valid_requests_start(&mut self) {
        self.account_info.valid_requests_start = self.account_info.requests_len;
        self.account_updates.valid_requests_start = self.account_info.requests_len;

        Runtime::emit_event(EventValidRequestsStart {
            account: self.account.address(),
            valid_requests_start: self.account_info.requests_len,
        });
    }
}