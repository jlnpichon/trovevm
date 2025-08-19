use std::collections::HashMap;

use bincode::error::{DecodeError, EncodeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{address::Address, function::CompiledFunction, storage::SharedStorage, value::Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContract {
    pub name: String,
    pub methods: HashMap<String, CompiledFunction>,
    pub vars: Vec<String>,
    pub code_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub methods: HashMap<String, CompiledFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInstance {
    pub contract: CompiledContract,
    #[serde(skip)]
    pub storage: SharedStorage,
    pub address: Address,
}

impl ContractInstance {
    pub fn new(address: Address, contract: CompiledContract, storage: SharedStorage) -> Self {
        Self {
            contract,
            storage,
            address,
        }
    }

    pub fn get_field(&mut self, key: &str) -> Option<Value> {
        self.storage.lock().get(self.address, key)
    }

    pub fn set_field(&mut self, key: &str, value: Value) {
        self.storage.lock().set(self.address, key, value)
    }

    pub fn get_method(&self, name: &str) -> Option<CompiledFunction> {
        self.contract.methods.get(name).cloned()
    }

    pub fn var_exists(&self, name: &String) -> bool {
        self.contract.var_exists(name)
    }
}

impl CompiledContract {
    pub fn into_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<CompiledContract, DecodeError> {
        let (compiled_contract, _) =
            bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(compiled_contract)
    }

    pub fn compute_hash(&mut self) -> Result<(), EncodeError> {
        let serialized_bytes = self.into_bytes()?;
        let hash: [u8; 32] = Sha256::digest(&serialized_bytes).into();
        self.code_hash = hash;
        Ok(())
    }

    pub fn var_exists(&self, name: &String) -> bool {
        self.vars.contains(name)
    }

    pub fn default_storage(&self) -> SharedStorage {
        let storage = SharedStorage::default();
        let instance_address = 0;
        storage.lock().init_instance(instance_address, self);
        storage
    }
}

impl ContractInstance {
    fn into_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    fn from_bytes(bytes: &[u8], storage: SharedStorage) -> Result<ContractInstance, DecodeError> {
        let (mut instance, _): (ContractInstance, _) =
            bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        instance.storage = storage;
        Ok(instance)
    }
}
