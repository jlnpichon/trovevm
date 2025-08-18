use std::collections::HashMap;

use crate::{address::Address, function::CompiledFunction, storage::SharedStorage, value::Value};

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub methods: HashMap<String, CompiledFunction>,
}

#[derive(Debug, Clone)]
pub struct ContractInstance {
    pub contract: Contract,
    pub storage: SharedStorage,
    pub address: Address,
}

impl ContractInstance {
    pub fn new(address: Address, contract: Contract, storage: SharedStorage) -> Self {
        Self {
            contract,
            storage,
            address,
        }
    }

    #[cfg(feature = "tokio-async")]
    pub async fn get_field(&mut self, key: &str) -> Option<Value> {
        self.storage.lock().await.get(self.address, key)
    }

    #[cfg(not(feature = "tokio-async"))]
    pub fn get_field(&mut self, key: &str) -> Option<Value> {
        self.storage.lock().get(self.address, key)
    }

    #[cfg(feature = "tokio-async")]
    pub async fn set_field(&mut self, key: &str, value: Value) {
        self.storage.lock().await.set(self.address, key, value)
    }

    #[cfg(not(feature = "tokio-async"))]
    pub fn set_field(&mut self, key: &str, value: Value) {
        self.storage.lock().set(self.address, key, value)
    }

    pub fn get_method(&self, name: &str) -> Option<CompiledFunction> {
        self.contract.methods.get(name).cloned()
    }
}
