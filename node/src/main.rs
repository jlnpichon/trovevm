use std::net::SocketAddr;

use jsonrpsee::{
    core::RpcResult, proc_macros::rpc, server::ServerBuilder, tracing::info, types::ErrorObject,
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use trove_core::WorldState;
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

#[derive(serde::Deserialize, Clone)]
pub struct DefineContractRequest {
    pub source_code: String,
    pub sender: String,
}

#[derive(serde::Serialize, Clone)]
pub struct DefineContractResponse {
    pub bytecode_hash: String,
}

#[rpc(server)]
pub trait TroveRpc {
    #[method(name = "system_version")]
    fn version(&self) -> RpcResult<String>;

    // Read-only call of a contract no state modifications.
    #[method(name = "trove_call")]
    fn call(&self, from: String, to: String, data: String) -> RpcResult<String>;

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
    world_state: WorldState,
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

    fn call(&self, from: String, to: String, data: String) -> RpcResult<String> {
        // Read contract state
        // Test code
        //
        /*
        {
            "method":"trove_call",
            "params": {
            "contract": "Counter",
            "function": "get",
            "args": []
        },
        {
            "method":"trove_call",
            "params": {
            "address": "0x1234567890abcdef",
            "function": "increment",
            "args": []
        }
        */

        // if params as contract
        // -> exec read-only contract
        // if params as address
        // -> exec ContractInstance in read-only
        todo!()
    }

    fn define_contract(&self, request: DefineContractRequest) -> RpcResult<DefineContractResponse> {
        let program = match parse_program("".to_string(), &request.source_code) {
            Ok(program) => program,
            Err(err) => {
                return Err(ErrorObject::owned(1000, format!("{err}"), None::<()>));
            }
        };

        let bytecode = match compile(&program.statements) {
            Ok(bytecode) => bytecode,
            Err(err) => {
                return Err(ErrorObject::owned(1000, format!("{err}"), None::<()>));
            }
        };

        let address = request
            .sender
            .parse::<u64>()
            .map_err(|err| ErrorObject::owned(1000, format!("{err}"), None::<()>))?;

        let contract = todo!();
        self.world_state.registry.insert(address, contract);

        Ok(DefineContractResponse {
            bytecode_hash: todo!(),
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
        world_state: WorldState::default(),
    };
    let handle = server.start(server_impl.into_rpc());
    info!("Server running on {local_addr:?}");

    handle.stopped().await;

    Ok(())
}
