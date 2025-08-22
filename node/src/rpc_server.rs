use std::sync::{Arc, Mutex};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    tracing::{debug, info},
    types::ErrorObject,
};

use trove_core::{Address, ContractEnv, ContractInstance, Value, WorldState};
use trove_vm::VM;
use trovec::{codegen::compile, parser::parse_program};

use crate::serialization::*;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DeployContractRequest {
    #[serde(deserialize_with = "deserialize_bytecode_hash")]
    pub bytecode_hash: [u8; 32],
    #[serde(deserialize_with = "deserialize_address")]
    pub sender: Address,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct DeployContractResponse {
    pub instance_address: Address,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefineContractRequest {
    pub source_code: String,
    pub contract_name: String,
    #[serde(deserialize_with = "deserialize_address")]
    pub sender: Address,
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
    pub value: Value,
    pub method: String,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct CallContractResponse {
    pub value: Value,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct SendTransactionRequest {
    #[serde(deserialize_with = "deserialize_address")]
    pub from: Address,
    #[serde(deserialize_with = "deserialize_address")]
    pub to: Address,
    #[serde(deserialize_with = "deserialize_value")]
    pub value: Value,
    pub method: String,
    #[serde(deserialize_with = "deserialize_values")]
    pub args: Vec<Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct SendTransactionResponse {
    pub value: Value,
}

#[rpc(server)]
pub trait TroveRpc {
    // Read-only call of a contract no state modifications.
    #[method(name = "trove_call")]
    fn call(&self, request: CallContractRequest) -> RpcResult<CallContractResponse>;

    // Deploy a contract from an EOA. Returns the contract Address
    #[method(name = "trove_deployContract")]
    fn deploy_contract(&self, request: DeployContractRequest) -> RpcResult<DeployContractResponse>;

    // Compile and save the contract in the registry
    #[method(name = "trove_defineContract")]
    fn define_contract(&self, request: DefineContractRequest) -> RpcResult<DefineContractResponse>;

    #[method(name = "trove_sendTransaction")]
    fn send_transaction(
        &self,
        request: SendTransactionRequest,
    ) -> RpcResult<SendTransactionResponse>;
}

#[derive(Clone, Debug)]
pub struct TroveRcpServerImpl {
    world_state: Arc<Mutex<WorldState>>,
}

impl TroveRcpServerImpl {
    pub fn new() -> Self {
        Self {
            world_state: Arc::new(Mutex::new(WorldState::default())),
        }
    }

    fn make_env(&self, sender: Address, instance: ContractInstance, value: u64) -> ContractEnv {
        ContractEnv {
            sender,
            self_address: instance.address,
            instance,
            value,
            block_number: 0,
            timestamp: 0,
        }
    }
}

impl TroveRpcServer for TroveRcpServerImpl {
    fn deploy_contract(&self, request: DeployContractRequest) -> RpcResult<DeployContractResponse> {
        debug!("deploy_contrat {request:?}");
        let contract = self
            .world_state
            .lock()
            .unwrap()
            .get_contract_from_hash(request.bytecode_hash)
            .ok_or(rpc_err(format!(
                "Contract with bytecode hash '{:?}' not found",
                request.bytecode_hash
            )))?
            .clone();

        let mut vm = VM::new();
        let instance = vm
            .deploy(
                request.sender,
                contract,
                request.args,
                &mut self.world_state.lock().unwrap(),
            )
            .map_err(|err| {
                rpc_err(format!(
                    "Can't deploy contract '{:?}': {err}",
                    request.bytecode_hash
                ))
            })?;

        Ok(DeployContractResponse {
            instance_address: instance.address,
        })
    }

    fn call(&self, request: CallContractRequest) -> RpcResult<CallContractResponse> {
        debug!("call {request:?}");
        let value = request
            .value
            .as_number()
            .ok_or(rpc_err(String::from("Value must be a number")))? as u64;

        let target = self
            .world_state
            .lock()
            .unwrap()
            .get_instance_or_contract(request.callee)
            .ok_or(rpc_err(format!("Contract '{}' not found", request.callee)))?
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
            .map_err(|err| rpc_err(format!("{err:?}")))?;

        Ok(CallContractResponse { value })
    }

    fn define_contract(&self, request: DefineContractRequest) -> RpcResult<DefineContractResponse> {
        debug!("define_contract {request:?}");
        if self
            .world_state
            .lock()
            .unwrap()
            .name_index
            .contains_key(&request.contract_name)
        {
            return Err(rpc_err(format!(
                "Contract '{}' already defined",
                request.contract_name
            )));
        }

        let mut program = match parse_program("".to_string(), &request.source_code) {
            Ok(program) => program,
            Err(err) => {
                return Err(rpc_err(err.to_string()));
            }
        };

        let compiled_program = match compile(&mut program.statements) {
            Ok(compiled_program) => compiled_program,
            Err(err) => {
                return Err(rpc_err(err.to_string()));
            }
        };

        let contract = compiled_program
            .contracts
            .get(&request.contract_name)
            .ok_or(rpc_err(format!(
                "No contract '{}' defined in source",
                request.contract_name
            )))?
            .clone();

        let bytecode_hash = contract.code_hash;
        let bytecode_str = hex::encode(contract.code_hash);

        if self
            .world_state
            .lock()
            .unwrap()
            .code_hash_index
            .contains_key(&contract.code_hash)
        {
            return Err(rpc_err(format!(
                "Contract with hash '{}' already defined",
                bytecode_str
            )));
        }

        let contract_address = self.world_state.lock().unwrap().define_contract(contract);
        self.world_state
            .lock()
            .unwrap()
            .name_index
            .insert(request.contract_name, contract_address);
        self.world_state
            .lock()
            .unwrap()
            .code_hash_index
            .insert(bytecode_hash, contract_address);
        self.world_state
            .lock()
            .unwrap()
            .sender_index
            .entry(request.sender)
            .or_default()
            .push(contract_address);

        Ok(DefineContractResponse {
            bytecode_hash,
            contract_address,
        })
    }

    fn send_transaction(
        &self,
        request: SendTransactionRequest,
    ) -> RpcResult<SendTransactionResponse> {
        debug!("send_transaction {request:?}");
        let value = request
            .value
            .as_number()
            .ok_or(rpc_err(String::from("Value must be a number")))? as u64;

        let instance = self
            .world_state
            .lock()
            .unwrap()
            .get_contract_instance(request.to)
            .ok_or(rpc_err(format!(
                "Contract instance '{}' not found",
                request.to
            )))?
            .clone();

        let env = self.make_env(request.from, instance.clone(), value);

        let mut vm = VM::new();

        let value = vm
            .call_contract_method(&request.method, request.args, env)
            .map_err(err_vm)?;

        Ok(SendTransactionResponse { value })
    }
}

fn err_vm<E: std::fmt::Debug>(err: E) -> ErrorObject<'static> {
    rpc_err(format!("VM error: {:?}", err))
}

fn rpc_err(msg: String) -> ErrorObject<'static> {
    ErrorObject::owned(1000, msg, None::<()>)
}
