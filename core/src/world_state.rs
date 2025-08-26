use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ContractInstance, Value,
    contract::{CompiledContract, ContractOrInstance},
    storage::{InMemoryStorage, SharedStorage},
};

use super::address::Address;

#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub name_index: HashMap<String, Address>,
    pub code_hash_index: HashMap<[u8; 32], Address>,
    pub contracts: HashMap<Address, CompiledContract>,
    pub instances: HashMap<Address, ContractInstance>,
    pub storage: SharedStorage,
    next_address: u64,
}

impl WorldState {
    pub fn generate_address(&mut self) -> Address {
        let addr = self.next_address;
        self.next_address += 1;
        addr
    }

    pub fn define_contract(&mut self, contract: CompiledContract) -> Address {
        let address = self.generate_address();
        self.contracts.insert(address, contract.clone());
        self.storage.lock().set(
            address,
            "_contract",
            Rc::new(RefCell::new(Value::ContractDef(contract))),
        );
        address
    }

    pub fn get_instance_or_contract(&self, address: Address) -> Option<ContractOrInstance> {
        if let Some(instance) = self.instances.get(&address) {
            Some(ContractOrInstance::Instance(instance.clone()))
        } else {
            self.contracts
                .get(&address)
                .map(|contract| ContractOrInstance::Contract(contract.clone()))
        }
    }

    pub fn get_contract_from_hash(&self, bytecode_hash: [u8; 32]) -> Option<&CompiledContract> {
        let address = self.code_hash_index.get(&bytecode_hash)?;
        self.contracts.get(address)
    }

    pub fn get_contract_instance(&self, address: Address) -> Option<&ContractInstance> {
        self.instances.get(&address)
    }

    pub fn from_storage(storage: InMemoryStorage) -> Self {
        let mut name_index: HashMap<String, Address> = HashMap::new();
        let mut code_hash_index: HashMap<[u8; 32], Address> = HashMap::new();
        let mut contracts: HashMap<Address, CompiledContract> = HashMap::new();
        let mut instances: HashMap<Address, ContractInstance> = HashMap::new();

        for (addr, key, value) in storage.iter() {
            if key == "_instance" {
                if let Value::ContractInstance(instance) = value {
                    instances.insert(addr, instance);
                } else {
                    todo!()
                };
            } else if key == "_contract" {
                if let Value::ContractDef(contract) = value {
                    contracts.insert(addr, contract.clone());
                    name_index.insert(contract.name, addr);
                    code_hash_index.insert(contract.code_hash, addr);
                } else {
                    todo!()
                };
            }
        }

        Self {
            name_index,
            code_hash_index,
            contracts,
            instances,
            storage: SharedStorage::new(storage),
            next_address: 0,
        }
    }
}
