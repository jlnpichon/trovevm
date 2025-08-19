use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use jsonrpsee::{
    core::RpcResult, proc_macros::rpc, server::ServerBuilder, tracing::info, types::ErrorObject,
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use trove_core::{
    Address, CompiledFunction, ContractInstance, RuntimeError, Value, WorldState,
    contract::CompiledContract, storage::SharedStorage,
};
use trove_vm::VM;
use trovec::{codegen::compile, parser::parse_program};

#[derive(serde::Deserialize, Clone)]
pub struct DeployContractRequest {
    pub bytecode_hash: String,
    pub sender: String,
}

#[derive(serde::Serialize, Clone)]
pub struct DeployContractResponse {
    pub instance_address: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefineContractRequest {
    pub source_code: String,
    pub contract_name: String,
    pub sender: String,
}

#[derive(serde::Serialize, Clone)]
pub struct DefineContractResponse {
    pub bytecode_hash: String,
    pub contract_address: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct CallContractRequest {
    pub caller: String,
    pub callee: String,
    pub method: String,
    pub args: Vec<String>,
}

#[rpc(server)]
pub trait TroveRpc {
    #[method(name = "system_version")]
    fn version(&self) -> RpcResult<String>;

    // Read-only call of a contract no state modifications.
    #[method(name = "trove_call")]
    fn call(&self, request: CallContractRequest) -> RpcResult<String>;

    // Deploy a contract from an EOA. Returns the contract Address
    #[method(name = "trove_deployContract")]
    fn deploy_contract(&self, request: DeployContractRequest) -> RpcResult<DeployContractResponse>;

    // Compile and save the contract in the registry
    #[method(name = "trove_defineContract")]
    fn define_contract(&self, request: DefineContractRequest) -> RpcResult<DefineContractResponse>;

    #[method(name = "trove_sendTransaction")]
    fn send_transaction(
        &self,
        from: String,
        to: String,
        value: Option<String>,
        data: Option<String>,
    ) -> RpcResult<String>;
}

#[derive(Clone, Debug)]
pub struct TroveRcpServerImpl {
    world_state: Arc<Mutex<WorldState>>,
}

fn args_parse(args: Vec<String>) -> Vec<Value> {
    args.iter().cloned().map(Value::from).collect()
}

impl TroveRpcServer for TroveRcpServerImpl {
    fn version(&self) -> RpcResult<String> {
        Ok("2".to_string())
    }

    fn deploy_contract(&self, request: DeployContractRequest) -> RpcResult<DeployContractResponse> {
        // Create a new address
        // Initialize ContractInstance in the storage
        // Call constructor
        /*
        {
          "jsonrpc":"2.0",
          "method":"deploy_contract",
          "params": { "hash": "0xabc123..." },
          "id": 2
        }
        */
        todo!()
    }

    fn call(&self, request: CallContractRequest) -> RpcResult<String> {
        let caller = request.caller.parse::<u64>().map_err(|err| {
            ErrorObject::owned(1000, format!("Caller address error: '{err}'"), None::<()>)
        })?;
        let callee = request.callee.parse::<u64>().map_err(|err| {
            ErrorObject::owned(1000, format!("Callee address error: '{err}'"), None::<()>)
        })?;
        let args = args_parse(request.args);

        let contract = self
            .world_state
            .lock()
            .unwrap()
            .registry
            .get(&callee)
            .ok_or(ErrorObject::owned(
                1000,
                format!("Contract '{}' not found", request.callee),
                None::<()>,
            ))?
            .clone();

        /*
                let method = contract
                    .methods
                    .get(&request.method)
                    .ok_or(ErrorObject::owned(
                        1000,
                        format!("Contract method '{}' not found", request.method),
                        None::<()>,
                    ))?
                    .clone();

                let storage_snapshot = self.world_state.lock().unwrap().storage.clone();
                call_contract(contract, caller, method, args, storage_snapshot)
                    .map_err(|err| ErrorObject::owned(1000, format!("{err:?}"), None::<()>))?;
        */

        let mut vm = VM::new();
        // TODO: insert caller and other variables into the VM env
        vm.sandbox_call(contract, &request.method, args, None)
            .map_err(|err| ErrorObject::owned(1000, format!("{err:?}"), None::<()>))?;
        // if params as contract
        // -> exec read-only contract
        // if params as address
        // -> exec ContractInstance in read-only
        todo!()
    }

    fn define_contract(&self, request: DefineContractRequest) -> RpcResult<DefineContractResponse> {
        if self
            .world_state
            .lock()
            .unwrap()
            .name_index
            .contains_key(&request.contract_name)
        {
            return Err(ErrorObject::owned(
                1000,
                format!("Contract '{}' already defined", request.contract_name),
                None::<()>,
            ));
        }

        let program = match parse_program("".to_string(), &request.source_code) {
            Ok(program) => program,
            Err(err) => {
                return Err(ErrorObject::owned(1000, format!("{err}"), None::<()>));
            }
        };

        let compiled_program = match compile(&program.statements) {
            Ok(compiled_program) => compiled_program,
            Err(err) => {
                return Err(ErrorObject::owned(1000, format!("{err}"), None::<()>));
            }
        };

        let sender_address = u64::from_str_radix(request.sender.trim_start_matches("0x"), 16)
            .map_err(|err| {
                ErrorObject::owned(1000, format!("Parsing sender address: '{err}'"), None::<()>)
            })?;

        let contract = compiled_program
            .contracts
            .get(&request.contract_name)
            .ok_or(ErrorObject::owned(
                1000,
                format!("No contract '{}' defined in source", request.contract_name),
                None::<()>,
            ))?
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
            return Err(ErrorObject::owned(
                1000,
                format!("Contract with hash '{}' already defined", bytecode_str),
                None::<()>,
            ));
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
            .entry(sender_address)
            .or_default()
            .push(contract_address);

        Ok(DefineContractResponse {
            bytecode_hash: bytecode_str,
            contract_address: format!("{contract_address}"),
        })
    }

    fn send_transaction(
        &self,
        from: String,
        to: String,
        value: Option<String>,
        data: Option<String>,
    ) -> RpcResult<String> {
        // Fire up a VM with the WorldState
        // Initializer VM var (sender, value, callee...)
        // Run the VM
        // Update the storage
        todo!()
    }
}

pub fn init_logging(verbosity: u8) -> anyhow::Result<()> {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(fmt::layer())
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging(3)?;

    let server = ServerBuilder::default()
        .build("127.0.0.1:4321".parse::<SocketAddr>()?)
        .await?;
    let local_addr = server.local_addr();

    let server_impl = TroveRcpServerImpl {
        world_state: Arc::new(Mutex::new(WorldState::default())),
    };
    let handle = server.start(server_impl.into_rpc());
    info!("Server running on {local_addr:?}");

    handle.stopped().await;

    Ok(())
}

/*
pub fn call_contract(
    contract: CompiledContract,
    caller: Address,
    method: CompiledFunction,
    args: Vec<Value>,
    storage_snapshot: SharedStorage,
) -> Result<Value, RuntimeError> {
    // TODO: insert caller into a *special* variable in the VM
    let mut vm = VM::new();
    let fake_instance = ContractInstance::new(0, contract.clone(), storage_snapshot);
    vm.call_method(method, Value::ContractInstance(fake_instance), args)
}
*/
