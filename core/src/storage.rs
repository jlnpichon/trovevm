use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::contract::CompiledContract;
use crate::value::SharedValue;
use crate::{address::Address, value::Value};

#[derive(Debug, Clone)]
pub struct SharedStorage {
    inner: Rc<Mutex<InMemoryStorage>>,
}

impl Default for SharedStorage {
    fn default() -> Self {
        Self {
            inner: Rc::new(Mutex::new(InMemoryStorage::new())),
        }
    }
}

impl SharedStorage {
    pub fn new(inner: InMemoryStorage) -> Self {
        Self {
            inner: Rc::new(Mutex::new(inner)),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, InMemoryStorage> {
        self.inner.lock().unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    memory: HashMap<Address, HashMap<String, SharedValue>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, address: Address, key: &str) -> Option<SharedValue> {
        self.memory.get(&address)?.get(key).cloned()
    }

    pub fn set(&mut self, address: Address, key: &str, value: SharedValue) {
        self.memory
            .entry(address)
            .or_default()
            .insert(key.to_string(), value);
    }

    pub fn init_instance(
        &mut self,
        sender: Address,
        address: Address,
        contract: &CompiledContract,
    ) {
        let mut instance_map = HashMap::new();

        for var in &contract.vars {
            instance_map.insert(var.clone(), SharedValue::from_null());
        }

        instance_map.insert("balance".to_string(), SharedValue::from_number(0.));
        instance_map.insert("owner".to_string(), SharedValue::from_number(sender as f64));

        self.memory.insert(address, instance_map);
    }

    pub fn iter(&self) -> impl Iterator<Item = (Address, String, Value)> + '_
    where
        Address: Clone,
        Value: Clone,
    {
        self.memory.iter().flat_map(|(addr, map)| {
            map.iter()
                .map(move |(key, value)| (*addr, key.clone(), value.borrow().clone()))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableStorage {
    memory: HashMap<Address, HashMap<String, Value>>,
}

impl From<&InMemoryStorage> for SerializableStorage {
    fn from(storage: &InMemoryStorage) -> Self {
        let mut memory = HashMap::new();

        for (addr, slots) in &storage.memory {
            let mut slots_plain = HashMap::new();
            for (k, v) in slots {
                slots_plain.insert(k.clone(), v.borrow().clone());
            }
            memory.insert(*addr, slots_plain);
        }

        SerializableStorage { memory }
    }
}

impl From<SerializableStorage> for InMemoryStorage {
    fn from(ss: SerializableStorage) -> Self {
        let mut memory = HashMap::new();

        for (addr, slots_plain) in ss.memory {
            let mut slots = HashMap::new();
            for (k, v) in slots_plain {
                slots.insert(k, SharedValue::from_value(v));
            }
            memory.insert(addr, slots);
        }

        InMemoryStorage { memory }
    }
}
