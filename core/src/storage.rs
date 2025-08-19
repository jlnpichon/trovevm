use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use crate::contract::CompiledContract;
use crate::{address::Address, value::Value};

#[cfg(feature = "tokio-async")]
type MutexImpl<T> = tokio::sync::Mutex<T>;

#[cfg(not(feature = "tokio-async"))]
type MutexImpl<T> = std::sync::Mutex<T>;

#[derive(Debug, Clone)]
pub struct SharedStorage {
    inner: Arc<MutexImpl<Box<dyn StorageBackend + Send + Sync>>>,
}

impl Default for SharedStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(MutexImpl::new(Box::new(InMemoryStorage::new()))),
        }
    }
}

impl SharedStorage {
    pub fn new(backend: impl StorageBackend + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(MutexImpl::new(Box::new(backend))),
        }
    }

    #[cfg(feature = "tokio-async")]
    pub fn lock(&self) -> tokio::sync::MutexGuard<'_, Box<dyn StorageBackend + Send + Sync>> {
        self.inner.blocking_lock()
    }

    #[cfg(not(feature = "tokio-async"))]
    pub fn lock(&self) -> std::sync::MutexGuard<'_, Box<dyn StorageBackend + Send + Sync>> {
        self.inner.lock().unwrap()
    }
}

pub trait StorageBackend: Debug + Send + Sync {
    fn get(&self, address: Address, key: &str) -> Option<Value>;
    fn set(&mut self, address: Address, key: &str, value: Value);
    fn init_instance(&mut self, address: Address, contract: &CompiledContract);
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

    fn init_instance(&mut self, address: Address, contract: &CompiledContract) {
        let mut instance_map = HashMap::new();

        for var in &contract.vars {
            instance_map.insert(var.clone(), Value::Null);
        }

        self.memory.insert(address, instance_map);
    }
}
