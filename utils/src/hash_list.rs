use scrypto::prelude::*;
use crate::list::*;
pub use crate::list::ListIndex;

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
        self.kvs.get_mut(&key)
            .map(|mut entry| {
                *entry = value.to_owned();
            })
            .unwrap_or_else(|| {
                self.list.push(key.clone());
                self.kvs.insert(key, value);
            });
    }

    pub fn get(&self, key: &K) -> Option<KeyValueEntryRef<V>> {
        self.kvs.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<KeyValueEntryRefMut<V>> {
        self.kvs.get_mut(key)
    }

    pub fn range(&self, start: ListIndex, end: ListIndex) -> Vec<V> {
        let mut result = Vec::new();
        for i in start..end {
            if let Some(key) = self.list.get(i) {
                let item = self.get(&key).unwrap().to_owned();
                result.push(item);
            } else {
                break;
            }
        }
        result
    }

    pub fn len(&self) -> ListIndex {
        self.list.len()
    }
}
