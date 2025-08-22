use serde::{Deserialize, Deserializer, Serializer, de::Error as DeError};

use trove_core::{Address, Value};

pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(D::Error::custom)
}

pub fn deserialize_bytecode_hash<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.len() != 64 {
        return Err(D::Error::custom(format!(
            "Invalid bytecode hash length: expected 64, got {}",
            s.len()
        )));
    }

    let bytes = hex::decode(&s).map_err(|e| D::Error::custom(format!("Invalid hex: {}", e)))?;
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);

    Ok(hash)
}

pub fn deserialize_value<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Value::from(s))
}

pub fn deserialize_values<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    Ok(strings
        .iter()
        .cloned()
        .map(Value::from)
        .collect::<Vec<Value>>())
}

pub fn serialize_bytecode_hash<S>(hash: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = hex::encode(hash);
    serializer.serialize_str(&s)
}
