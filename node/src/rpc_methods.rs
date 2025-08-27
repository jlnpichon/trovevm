use std::{cell::RefCell, rc::Rc};

use crate::serialization::*;
use jsonrpc_core::Value;
use serde::de::DeserializeOwned;
use tracing::debug;
use trove_core::{Address, ContractEnv, ContractInstance, WorldState};
use trove_vm::VM;
use trovec::{codegen::compile, parser::parse_program};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DeployContractRequest {
    #[serde(deserialize_with = "deserialize_bytecode_hash")]
    pub bytecode_hash: [u8; 32],
    #[serde(deserialize_with = "deserialize_address")]
    pub sender: Address,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<trove_core::Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct DeployContractResponse {
    pub instance_address: Address,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefineContractRequest {
    pub source_code: String,
    pub contract_name: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct DefineContractResponse {
    #[serde(serialize_with = "serialize_bytecode_hash")]
    pub bytecode_hash: [u8; 32],
    pub contract_address: Address,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CallContractRequest {
    #[serde(deserialize_with = "deserialize_address")]
    pub caller: Address,
    #[serde(deserialize_with = "deserialize_address")]
    pub callee: Address,
    #[serde(deserialize_with = "deserialize_value")]
    pub value: trove_core::Value,
    pub method: String,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<trove_core::Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct CallContractResponse {
    pub value: trove_core::Value,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct SendTransactionRequest {
    #[serde(deserialize_with = "deserialize_address")]
    pub from: Address,
    #[serde(deserialize_with = "deserialize_address")]
    pub to: Address,
    #[serde(deserialize_with = "deserialize_value")]
    pub value: trove_core::Value,
    pub method: String,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<trove_core::Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct SendTransactionResponse {
    pub value: trove_core::Value,
}

pub fn deploy_contract_handler(
    world_state: &Rc<RefCell<WorldState>>,
    params: serde_json::Value,
) -> Result<DeployContractResponse, String> {
    debug!("deploy_contrat {params:?}");

    let request: DeployContractRequest = parse_params(params)?;

    let contract = world_state
        .borrow()
        .get_contract_from_hash(request.bytecode_hash)
        .ok_or(format!(
            "Contract with bytecode hash '{:?}' not found",
            request.bytecode_hash
        ))?
        .clone();

    let mut vm = VM::new();
    let instance = vm
        .deploy(
            request.sender,
            contract,
            request.args,
            &mut world_state.borrow_mut(),
        )
        .map_err(|err| format!("Can't deploy contract '{:?}': {err}", request.bytecode_hash))?;

    Ok(DeployContractResponse {
        instance_address: instance.address,
    })
}

pub fn define_contract_handler(
    world_state: &Rc<RefCell<WorldState>>,
    params: serde_json::Value,
) -> Result<DefineContractResponse, String> {
    debug!("define_contract {params:?}");

    let request: DefineContractRequest = parse_params(params)?;

    if world_state
        .borrow()
        .name_index
        .contains_key(&request.contract_name)
    {
        return Err(format!(
            "Contract '{}' already defined",
            request.contract_name
        ));
    }

    let mut program = match parse_program("".to_string(), &request.source_code) {
        Ok(program) => program,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let compiled_program = match compile(&mut program.statements) {
        Ok(compiled_program) => compiled_program,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let contract = compiled_program
        .contracts
        .get(&request.contract_name)
        .ok_or(format!(
            "No contract '{}' defined in source",
            request.contract_name
        ))?
        .clone();

    let bytecode_hash = contract.code_hash;
    let bytecode_str = hex::encode(contract.code_hash);

    if world_state
        .borrow()
        .code_hash_index
        .contains_key(&contract.code_hash)
    {
        return Err(format!(
            "Contract with hash '{}' already defined",
            bytecode_str
        ));
    }

    let contract_address = world_state.borrow_mut().define_contract(contract);
    world_state
        .borrow_mut()
        .name_index
        .insert(request.contract_name, contract_address);
    world_state
        .borrow_mut()
        .code_hash_index
        .insert(bytecode_hash, contract_address);

    Ok(DefineContractResponse {
        bytecode_hash,
        contract_address,
    })
}

pub fn sandbox_call_handler(
    world_state: &Rc<RefCell<WorldState>>,
    params: serde_json::Value,
) -> Result<CallContractResponse, String> {
    let request: CallContractRequest = parse_params(params)?;

    let value = request.value.as_number().ok_or("Value must be a number")? as u64;

    let target = world_state
        .borrow()
        .get_instance_or_contract(request.callee)
        .ok_or_else(|| format!("Contract '{}' not found", request.callee))?
        .clone();

    let mut vm = VM::new();
    let value = vm
        .sandbox_call(
            target,
            &request.method,
            Some(request.caller),
            Some(value),
            request.args,
            None,
        )
        .map_err(|e| format!("{e}"))?;

    Ok(CallContractResponse { value })
}

pub fn send_transaction_handler(
    world_state: &Rc<RefCell<WorldState>>,
    params: serde_json::Value,
) -> Result<SendTransactionResponse, String> {
    debug!("send_transaction {params:?}");

    let request: SendTransactionRequest = parse_params(params)?;

    let value = request
        .value
        .as_number()
        .ok_or(String::from("Value must be a number"))? as u64;

    let instance = world_state
        .borrow()
        .get_contract_instance(request.to)
        .ok_or(format!("Contract instance '{}' not found", request.to))?
        .clone();

    let env = make_env(request.from, instance.clone(), value);

    let mut vm = VM::new();

    let value = vm
        .call_contract_method(&request.method, request.args, env)
        .map_err(|e| format!("{e}"))?;

    Ok(SendTransactionResponse { value })
}

fn make_env(sender: Address, instance: ContractInstance, value: u64) -> ContractEnv {
    ContractEnv {
        sender,
        self_address: instance.address,
        instance,
        value,
        block_number: 0,
        timestamp: 0,
    }
}

fn parse_params<T>(params: Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    match params {
        Value::Object(_) => {
            serde_json::from_value(params).map_err(|e| format!("Invalid params object: {}", e))
        }
        Value::Array(mut arr) => {
            if arr.is_empty() {
                return Err("Params array is empty".to_string());
            }
            serde_json::from_value(arr.remove(0))
                .map_err(|e| format!("Invalid params array element: {}", e))
        }
        _ => Err("Params must be an object or an array with one object".to_string()),
    }
}
