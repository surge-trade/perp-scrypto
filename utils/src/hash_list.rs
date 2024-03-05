use scrypto::prelude::*;
use crate::list::*;

#[derive(ScryptoSbor)]
pub struct HashList<K: ScryptoSbor + Clone, V: ScryptoSbor + Clone> {
    list: List<K>,
    kvs: KeyValueStore<K, V>,
}

impl<K: ScryptoSbor + Clone, V: ScryptoSbor + Clone> HashList<K, V> {
    pub fn new() -> Self {
        Self { 
            kvs: KeyValueStore::new(),
            list: List::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.kvs.get(&key).is_none() {
            self.list.push(key.clone());
            self.kvs.insert(key, value);
        } else {
            self.kvs.insert(key, value);
        }
    }

    pub fn get(&self, key: &K) -> Option<KeyValueEntryRef<V>> {
        self.kvs.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<KeyValueEntryRefMut<V>> {
        self.kvs.get_mut(key)
    }

    pub fn range(&self, start: u64, end: u64) -> Vec<V> {
        let mut result = Vec::new();
        for i in start..end {
            if let Some(key) = self.list.get(i) {
                let item = self.get(&key).unwrap().clone();
                result.push(item);
            } else {
                break;
            }
        }
        result
    }

    pub fn len(&self) -> u64 {
        self.list.len()
    }
}
