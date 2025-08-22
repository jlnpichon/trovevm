use std::collections::HashMap;

use crate::{
    ContractInstance,
    contract::{CompiledContract, ContractOrInstance},
    storage::SharedStorage,
};

use super::address::Address;

#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub name_index: HashMap<String, Address>,
    pub code_hash_index: HashMap<[u8; 32], Address>,
    pub sender_index: HashMap<Address, Vec<Address>>, // list a defined contract
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
        self.contracts.insert(address, contract);
        address
    }

    pub fn get_instance_or_contract(&self, address: Address) -> Option<ContractOrInstance> {
        if let Some(instance) = self.instances.get(&address) {
            Some(ContractOrInstance::Instance(instance.clone()))
        } else if let Some(contract) = self.contracts.get(&address) {
            Some(ContractOrInstance::Contract(contract.clone()))
        } else {
            None
        }
    }

    pub fn get_contract_from_hash(&self, bytecode_hash: [u8; 32]) -> Option<&CompiledContract> {
        let address = self.code_hash_index.get(&bytecode_hash)?;
        self.contracts.get(address)
    }

    pub fn get_contract_instance(&self, address: Address) -> Option<&ContractInstance> {
        self.instances.get(&address)
    }
}
