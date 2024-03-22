use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct Vaults {
    vaults: KeyValueStore<ResourceAddress, Vault>,
}

impl Vaults {
    pub fn new<F>(create_fn: F) -> Self 
    where
        F: Fn() -> KeyValueStore<ResourceAddress, Vault>,
    {
        Self { 
            vaults: create_fn(),
        }
    }

    pub fn amount(&self, resource: &ResourceAddress) -> Decimal {
        if let Some(vault) = self.vaults.get(resource) {
            vault.amount()
        } else {
            dec!(0)
        }
    }

    pub fn amounts(&self, resources: Vec<ResourceAddress>) -> HashMap<ResourceAddress, Decimal> {
        resources.into_iter().map(|resource| (resource, self.amount(&resource))).collect()
    }

    pub fn put(&mut self, tokens: Bucket) {
        let resource = tokens.resource_address();
        if self.vaults.get(&resource).is_some() {
            let mut vault = self.vaults.get_mut(&resource).unwrap();
            vault.put(tokens);
        } else {
            self.vaults.insert(resource, Vault::with_bucket(tokens));
        }
    }

    pub fn put_batch(&mut self, tokens: Vec<Bucket>) {
        for token in tokens {
            self.put(token);
        }
    }

    pub fn take(&mut self, resource: &ResourceAddress, amount: Decimal) -> Bucket {
        if self.vaults.get(&resource).is_none() {
            self.vaults.insert(*resource, Vault::new(*resource));
        }
        let mut vault = self.vaults.get_mut(&resource).unwrap();
        vault.take(amount)
    }

    pub fn take_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>) -> Vec<Bucket> {
        claims
            .into_iter()
            .map(|(resource, amount)| self.take(&resource, amount))
            .collect()
    }

    pub fn take_advanced(&mut self, resource: &ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        if self.vaults.get(&resource).is_none() {
            self.vaults.insert(*resource, Vault::new(*resource));
        }
        let mut vault = self.vaults.get_mut(&resource).unwrap();
        vault.take_advanced(amount, withdraw_strategy)
    }

    pub fn take_advanced_batch(&mut self, claims: Vec<(ResourceAddress, Decimal)>, withdraw_strategy: WithdrawStrategy) -> Vec<Bucket> {
        claims
            .into_iter()
            .map(|(resource, amount)| self.take_advanced(&resource, amount, withdraw_strategy))
            .collect()
    }
}
