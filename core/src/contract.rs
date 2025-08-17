use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    address::Address,
    function::CompiledFunction,
    storage::{InMemoryStorage, StorageBackend},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct SharedStorage(pub Rc<RefCell<dyn StorageBackend>>);

impl Default for SharedStorage {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(InMemoryStorage::new())))
    }
}

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

    pub fn get_field(&mut self, key: &str) -> Option<Value> {
        self.storage.0.borrow().get(self.address, key)
    }

    pub fn set_field(&mut self, key: &str, value: Value) {
        self.storage.0.borrow_mut().set(self.address, key, value)
    }

    pub fn get_method(&self, name: &str) -> Option<CompiledFunction> {
        self.contract.methods.get(name).cloned()
    }
}
