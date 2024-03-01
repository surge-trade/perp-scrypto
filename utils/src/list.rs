use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct List<T: ScryptoSbor + Clone> {
    pointer: u64,
    kvs: KeyValueStore<u64, T>,
}

impl<T: ScryptoSbor + Clone> List<T> {
    pub fn new() -> Self {
        Self { 
            pointer: 0,
            kvs: KeyValueStore::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.kvs.insert(self.pointer, item);
        self.pointer += 1;
    }

    pub fn get(&self, index: u64) -> Option<T> where T: Clone {
        self.kvs.get(&index).map(|item| item.clone())
    }

    pub fn get_mut(&mut self, index: u64) -> Option<KeyValueEntryRefMut<T>> {
        self.kvs.get_mut(&index)
    }

    pub fn range(&self, start: u64, end: u64) -> Vec<T> {
        let mut result = Vec::new();
        for i in start..end {
            if let Some(item) = self.get(i) {
                result.push(item);
            } else {
                break;
            }
        }
        result
    }

    pub fn len(&self) -> u64 {
        self.pointer
    }
}