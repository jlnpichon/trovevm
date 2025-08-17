use std::collections::HashMap;

use crate::{address::Address, value::Value};

pub trait StorageBackend: std::fmt::Debug {
    fn get(&self, address: Address, key: &str) -> Option<Value>;
    fn set(&mut self, address: Address, key: &str, value: Value);
    fn init_instance(&mut self, address: Address);
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    memory: HashMap<Address, HashMap<String, Value>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
impl StorageBackend for InMemoryStorage {
    fn get(&self, address: Address, key: &str) -> Option<Value> {
        self.memory.get(&address)?.get(key).cloned()
    }

    fn set(&mut self, address: Address, key: &str, value: Value) {
        self.memory
            .entry(address)
            .or_default()
            .insert(key.to_string(), value);
    }

    fn init_instance(&mut self, address: Address) {
        self.memory.entry(address).or_default();
    }
}
