use std::collections::HashMap;

use crate::{Value, contract::CompiledContract, storage::SharedStorage};

use super::address::Address;

#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub name_index: HashMap<String, Address>,
    pub code_hash_index: HashMap<[u8; 32], Address>,
    pub sender_index: HashMap<Address, Vec<Address>>, // list a defined contract
    pub registry: HashMap<Address, CompiledContract>,
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
        self.registry.insert(address, contract);
        address
    }
}
