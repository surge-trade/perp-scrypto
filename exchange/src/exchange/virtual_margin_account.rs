use scrypto::prelude::*;
use std::cmp::Reverse;
use account::*;
use referral_generator::*;
use common::{PairId, ListIndex, _REFERRAL_RESOURCE};
use super::errors::*;
use super::events::*;
use super::exchange_mod::MarginAccount;
use super::requests::*;

const REFERRAL_RESOURCE: ResourceAddress = _REFERRAL_RESOURCE;

pub struct VirtualMarginAccount {
    account: Global<MarginAccount>,
    account_info: MarginAccountInfo,
    account_updates: MarginAccountUpdates,
    referral_data: Option<ReferralData>,
    referral_rewarded: bool,
}

impl VirtualMarginAccount {
    pub fn new(account: ComponentAddress, collateral_resources: Vec<ResourceAddress>) -> Self {
        let account = Global::<MarginAccount>::try_from(account).expect(ERROR_INVALID_MARGIN_ACCOUNT);
        let account_info = account.get_info(collateral_resources);
        let referral_data: Option<ReferralData> = if let Some(referral) = account_info.referral_id.clone() {
            let referral_data: ReferralData = NonFungible::from(NonFungibleGlobalId::new(REFERRAL_RESOURCE, referral)).data();
            Some(referral_data)
        } else {
            None
        };

        Self {
            account,
            account_updates: MarginAccountUpdates {
                position_updates: HashMap::new(),
                virtual_balance: account_info.virtual_balance,
                valid_requests_start: account_info.valid_requests_start,
                request_additions: vec![],
                request_updates: HashMap::new(),
                active_request_additions: vec![],
                active_request_removals: vec![],
            },
            account_info,
            referral_data,
            referral_rewarded: false,
        }
    }

    pub fn realize(self) {
        let requests: Vec<(ListIndex, KeeperRequest)> = self.account_updates.request_additions.iter()
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

        if self.referral_rewarded {
            let referral_id = self.account_info.referral_id.unwrap();
            let referral_data = self.referral_data.unwrap();
            let referral_manager = ResourceManager::from_address(REFERRAL_RESOURCE);
            referral_manager.update_non_fungible_data(&referral_id, "balance", referral_data.balance);
            referral_manager.update_non_fungible_data(&referral_id, "total_rewarded", referral_data.total_rewarded);
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

    pub fn position(&self, pair_id: &PairId) -> AccountPosition {
        self.account_info.positions.get(pair_id).cloned().unwrap_or_default()
    }

    // pub fn position_amount(&self, pair_id: PairId) -> Decimal {
    //     self.account_info.positions.get(&pair_id).map(|position| position.amount).unwrap_or_default()
    // }

    pub fn referral(&self) -> Option<(NonFungibleGlobalId, ReferralData)> {
        if let Some(referral_id) = self.account_info.referral_id.clone() {
            let referral_id = NonFungibleGlobalId::new(REFERRAL_RESOURCE, referral_id);
            let referral_data = self.referral_data.clone().unwrap();
            Some((referral_id, referral_data))
        } else {
            None
        }
    }

    pub fn fee_share_referral(&self) -> Decimal {
        self.referral_data.as_ref().map_or(dec!(0), |referral_data| referral_data.fee_referral)
    }

    pub fn fee_rebate(&self) -> Decimal {
        self.referral_data.as_ref().map_or(dec!(0), |referral_data| (dec!(1) - referral_data.fee_rebate))
    }

    pub fn reward_referral(&mut self, amount: Decimal) {
        if let Some(referral_data) = self.referral_data.as_mut() {
            referral_data.balance += amount;
            referral_data.total_rewarded += amount;
            self.referral_rewarded = true;
        }
    }

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

    pub fn requests_tail(&self, n: ListIndex, end: Option<ListIndex>) -> Vec<(ListIndex, KeeperRequest)> {
        self.account.get_requests_tail(n, end)
    }

    pub fn active_requests(&self) -> Vec<(ListIndex, KeeperRequest)> {
        let mut requests = self.account.get_active_requests().into_iter()
            .collect::<Vec<_>>();
        requests.sort_by_key(|(index, _)| Reverse(*index));
        requests
    }

    pub fn active_requests_len(&self) -> usize {
        self.account_info.active_requests_len
    }

    pub fn collateral_amount(&self, resource: &ResourceAddress) -> Decimal {
        self.account_info.collateral_balances.get(resource).copied().unwrap_or_default()
    }

    pub fn collateral_amounts(&self) -> &HashMap<ResourceAddress, Decimal> {
        &self.account_info.collateral_balances
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

    pub fn get_level_1_auth(&self) -> AccessRule {
        self.account.get_role("level_1").expect(ERROR_MISSING_AUTH)
    }

    pub fn get_level_2_auth(&self) -> AccessRule {
        self.account.get_role("level_2").expect(ERROR_MISSING_AUTH)
    }

    pub fn get_level_3_auth(&self) -> AccessRule {
        self.account.get_role("level_3").expect(ERROR_MISSING_AUTH)
    }

    pub fn verify_level_1_auth(&self) {
        let rule = self.get_level_1_auth();
        Runtime::assert_access_rule(rule);
    }

    pub fn verify_level_2_auth(&self) {
        let rule = self.get_level_2_auth();
        Runtime::assert_access_rule(rule);
    }

    pub fn verify_level_3_auth(&self) {
        let rule = self.get_level_3_auth();
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

    pub fn push_request(&mut self, request: Request, expiry_seconds: u64, status: Status, effected_components: Vec<ComponentAddress>) {
        assert!(
            status == STATUS_ACTIVE || status == STATUS_DORMANT,
            "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_INVALID_REQUEST_STATUS, status, vec![STATUS_ACTIVE, STATUS_DORMANT]
        );

        let submission = Clock::current_time_rounded_to_seconds();
        let expiry = submission.add_seconds(expiry_seconds as i64).expect(ERROR_ARITHMETIC);

        let keeper_request = KeeperRequest {
            request: request.encode(), 
            submission,
            expiry,
            status,
            effected_components,
        };
        self._add_active_request(self.account_info.requests_len);

        self.account_updates.request_additions.push(keeper_request);
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
                if status == STATUS_ACTIVE || status == STATUS_DORMANT {
                    if keeper_request.status != STATUS_ACTIVE && keeper_request.status != STATUS_DORMANT {
                        self._add_active_request(index);
                    }
                } else if keeper_request.status == STATUS_ACTIVE || keeper_request.status == STATUS_DORMANT {
                    self._remove_active_request(index);
                }
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
        if keeper_request.status == STATUS_ACTIVE || keeper_request.status == STATUS_DORMANT {
            self._remove_active_request(index);
        }
        keeper_request.status = STATUS_CANCELLED;
        keeper_request.submission = current_time;
        self.account_updates.request_updates.insert(index, keeper_request);
    }

    pub fn process_request(&mut self, index: ListIndex) -> (Request, Instant, bool) {
        assert!(
            index >= self.valid_requests_start(),
            "{}, VALUE:{}, REQUIRED:{}, OP:>= |", ERROR_PROCESS_REQUEST_BEFORE_VALID_START, index, self.valid_requests_start()
        );
        
        let current_time = Clock::current_time_rounded_to_seconds();
        let mut keeper_request = self.keeper_request(index);
        
        let expired = current_time.compare(keeper_request.expiry, TimeComparisonOperator::Gte);
        if expired {
            assert!(
                keeper_request.status == STATUS_ACTIVE || keeper_request.status == STATUS_DORMANT,
                "{}, VALUE:{}, REQUIRED:{:?}, OP:contains |", ERROR_PROCESS_REQUEST_NOT_ACTIVE, keeper_request.status, vec![STATUS_ACTIVE, STATUS_DORMANT]
            );

            keeper_request.status = STATUS_EXPIRED;
        } else {
            assert!(
                keeper_request.status == STATUS_ACTIVE,
                "{}, VALUE:{}, REQUIRED:{}, OP:== |", ERROR_PROCESS_REQUEST_NOT_ACTIVE, keeper_request.status, STATUS_ACTIVE
            );
            
            keeper_request.status = STATUS_EXECUTED;
        }

        let submission = keeper_request.submission;
        let request = Request::decode(&keeper_request.request);

        self._remove_active_request(index);
        keeper_request.submission = current_time;
        self.account_updates.request_updates.insert(index, keeper_request);

        (request, submission, expired)
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

    pub fn update_position(&mut self, pair_id: &PairId, position: AccountPosition) {
        self.account_info.positions.insert(pair_id.clone(), position.clone());
        self.account_updates.position_updates.insert(pair_id.clone(), position);
    }

    pub fn remove_position(&mut self, pair_id: &PairId) {
        self.account_info.positions.remove(pair_id);
        self.account_updates.position_updates.insert(pair_id.clone(), AccountPosition::default());
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

    fn _add_active_request(&mut self, index: ListIndex) {
        self.account_info.active_requests_len += 1;
        self.account_updates.active_request_additions.push(index);
    }

    fn _remove_active_request(&mut self, index: ListIndex) {
        self.account_info.active_requests_len -= 1;
        self.account_updates.active_request_removals.push(index);
    }

    fn _status_phases(&self, status: Status) -> Vec<Status> {
        match status {
            STATUS_ACTIVE => vec![STATUS_DORMANT],
            STATUS_EXECUTED => vec![STATUS_ACTIVE],
            STATUS_CANCELLED => vec![STATUS_ACTIVE, STATUS_DORMANT],
            _ => panic!("{}", ERROR_INVALID_REQUEST_STATUS),
        }
    }
}