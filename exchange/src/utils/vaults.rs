use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct Vaults {
    vaults: KeyValueStore<ResourceAddress, Vault>,
}

impl Vaults {
    pub fn new() -> Self {
        Self {
            vaults: KeyValueStore::new(),
        }
    }

    pub fn amount(&self, resource: &ResourceAddress) -> Decimal {
        if let Some(vault) = self.vaults.get(resource) {
            vault.amount()
        } else {
            dec!(0)
        }
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

    pub fn take(&mut self, resource: ResourceAddress, amount: Decimal) -> Bucket {
        if self.vaults.get(&resource).is_none() {
            self.vaults.insert(resource, Vault::new(resource));
        }
        let mut vault = self.vaults.get_mut(&resource).unwrap();
        vault.take(amount)
    }

    pub fn take_advanced(&mut self, resource: ResourceAddress, amount: Decimal, withdraw_strategy: WithdrawStrategy) -> Bucket {
        if self.vaults.get(&resource).is_none() {
            self.vaults.insert(resource, Vault::new(resource));
        }
        let mut vault = self.vaults.get_mut(&resource).unwrap();
        vault.take_advanced(amount, withdraw_strategy)
    }
}
