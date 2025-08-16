use std::collections::HashMap;

use super::{
    contract::{Contract, SharedStorage},
    storage::Address,
};

#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub registry: HashMap<Address, Contract>,
    pub storage: SharedStorage,
}

impl WorldState {
    pub fn generate_address(&self, sender: Address) -> Address {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sender.hash(&mut hasher);
        self.registry.len().hash(&mut hasher);
        hasher.finish()
    }

    pub fn define_contract(&mut self, sender: Address, contract: Contract) {
        let address = self.generate_address(sender);
        self.registry.insert(address, contract);
    }
}
