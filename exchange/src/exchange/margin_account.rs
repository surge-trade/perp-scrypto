use scrypto::prelude::*;
use crate::utils::{List, Vaults};
use super::errors::*;
use super::keeper_requests::KeeperRequest;
use self::margin_account::MarginAccount;

#[derive(ScryptoSbor, Clone, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub cost: Decimal,
    pub funding_index: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountInfo {
    positions: HashMap<u64, AccountPosition>,
    collateral_balances: HashMap<ResourceAddress, Decimal>,
    virtual_balance: Decimal,
}

#[derive(ScryptoSbor)]
pub struct MarginAccountUpdates {
    position_updates: HashMap<u64, AccountPosition>,
    virtual_balance: Decimal,
}

pub struct VirtualMarginAccount {
    account: Global<MarginAccount>,
    account_info: MarginAccountInfo,
    account_updates: MarginAccountUpdates,
}

impl VirtualMarginAccount {
    pub fn new(account: ComponentAddress) -> Self {
        let account = Global::<MarginAccount>::try_from(account).expect(ERROR_INVALID_ACCOUNT);
        let account_info = account.get_info();

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

    pub fn deposit_vaults(&mut self, tokens: Bucket) {
        self.account.deposit_vaults(tokens);
    }

    pub fn deposit_vaults_batch(&mut self, tokens: Vec<Bucket>) {
        self.account.deposit_vaults_batch(tokens);
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

    pub fn withdraw_vaults(&mut self, resource: &ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        self.account.withdraw_vaults(*resource, amount, withdraw_strategy)
    }

    pub fn withdraw_vaults_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
        self.account.withdraw_vaults_batch(claims, withdraw_strategy)
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

    pub fn transfer_from_vaults_to_collateral(&mut self, transfers: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) {
        for (resource, amount) in transfers.iter() {
            self.account_info.collateral_balances
                .entry(*resource)
                .and_modify(|balance| *balance += *amount)
                .or_insert(*amount);
        }

        self.account.transfer_from_vaults_to_collateral(transfers, withdraw_strategy);
    }

    pub fn transfer_from_collateral_to_vaults(&mut self, transfers: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) {
        for (resource, amount) in transfers.iter() {
            self.account_info.collateral_balances
                .entry(*resource)
                .and_modify(|balance| *balance -= *amount)
                .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));
        }

        self.account.transfer_from_collateral_to_vaults(transfers, withdraw_strategy);
    }

    pub fn update_virtual_balance(&mut self, virtual_balance: Decimal) {
        self.account_info.virtual_balance = virtual_balance;
        self.account_updates.virtual_balance = virtual_balance;
    }

    pub fn update_position(&mut self, pair_id: u64, position: AccountPosition) {
        self.account_info.positions.insert(pair_id, position.clone());
        self.account_updates.position_updates.insert(pair_id, position);
    }
}

#[blueprint]
pub mod margin_account {
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

    // TODO:  owner auth

    struct MarginAccount {
        vaults: Vaults,
        collateral: Vaults,
        positions: HashMap<u64, AccountPosition>,
        collateral_balances: HashMap<ResourceAddress, Decimal>,
        virtual_balance: Decimal,
        requests: List<KeeperRequest>,
    }

    impl MarginAccount {
        pub fn new() -> Global<MarginAccount> {
            Self {
                vaults: Vaults::new(),
                collateral: Vaults::new(),
                positions: HashMap::new(),
                collateral_balances: HashMap::new(),
                virtual_balance: dec!(0),
                requests: List::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn get_info(&self) -> MarginAccountInfo {
            MarginAccountInfo {
                positions: self.positions.clone(),
                collateral_balances: self.collateral_balances.clone(),
                virtual_balance: self.virtual_balance,
            }
        }

        pub fn update(&mut self, update: MarginAccountUpdates) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for (pair_id, position) in update.position_updates {
                if position.amount != dec!(0) {
                    self.positions.insert(pair_id, position);
                } else {
                    self.positions.remove(&pair_id);
                }
            }
            self.virtual_balance = update.virtual_balance;
        }

        pub fn deposit_vaults(&mut self, token: Bucket) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.vaults.put(token);
        }

        pub fn deposit_vaults_batch(&mut self, tokens: Vec<Bucket>) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.vaults.put_batch(tokens);
        }

        pub fn deposit_collateral(&mut self, token: Bucket) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            let amount = token.amount();
            let resource = token.resource_address();
            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);

            self.collateral.put(token);
        }

        pub fn deposit_collateral_batch(&mut self, tokens: Vec<Bucket>) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for token in tokens.iter() {
                let amount = token.amount();
                let resource = token.resource_address();
                self.collateral_balances
                    .entry(resource)
                    .and_modify(|balance| *balance += amount)
                    .or_insert(amount);
            }

            self.collateral.put_batch(tokens);
        }

        pub fn withdraw_vaults(&mut self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.vaults.take_advanced(&resource, amount, withdraw_strategy)
        }

        pub fn withdraw_vaults_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.vaults.take_advanced_batch(claims, withdraw_strategy)
        }

        pub fn withdraw_collateral(&mut self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            self.collateral_balances
                .entry(resource)
                .and_modify(|balance| *balance -= amount)
                .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));

            self.collateral.take_advanced(&resource, amount, withdraw_strategy)
        }

        pub fn withdraw_collateral_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
            Runtime::assert_access_rule(AUTHORITY.get_rule());

            for (resource, amount) in claims.iter() {
                self.collateral_balances
                    .entry(*resource)
                    .and_modify(|balance| *balance -= *amount)
                    .or_insert_with(|| panic!("{}", PANIC_NEGATIVE_COLLATERAL));
            }
            
            let tokens = self.collateral.take_advanced_batch(claims, withdraw_strategy);
            
            tokens
        }
        
        pub fn transfer_from_vaults_to_collateral(&mut self, transfers: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());
            
            let tokens = self.vaults.take_advanced_batch(transfers, withdraw_strategy);
            self.deposit_collateral_batch(tokens);
        }

        pub fn transfer_from_collateral_to_vaults(&mut self, transfers: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) {
            Runtime::assert_access_rule(AUTHORITY.get_rule());
    
            let tokens = self.withdraw_collateral_batch(transfers, withdraw_strategy);
            self.vaults.put_batch(tokens);
        }
    }
}