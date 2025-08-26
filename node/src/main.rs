use std::cell::RefCell;
use std::fs;
use std::net::TcpListener;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use tungstenite::{Message, accept};

use logging::init_logging;
use trove_core::storage::SerializableStorage;
use trove_core::{WorldState, storage::InMemoryStorage};

use crate::args::CliArgs;
use crate::rpc_methods::{
    define_contract_handler, deploy_contract_handler, sandbox_call_handler,
    send_transaction_handler,
};

mod args;
mod logging;
mod rpc_methods;
mod serialization;

#[derive(Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: serde_json::Value,
}

#[derive(Serialize, Debug)]
struct RpcResponse {
    jsonrpc: &'static str,
    result: serde_json::Value,
    id: serde_json::Value,
}

fn call_handler<T, F>(handler: F, params: serde_json::Value) -> serde_json::Value
where
    T: serde::Serialize,
    F: FnOnce(serde_json::Value) -> Result<T, String>,
{
    match handler(params) {
        Ok(resp_struct) => serde_json::to_value(resp_struct)
            .unwrap_or_else(|e| serde_json::json!({ "error": format!("Serialize error: {}", e) })),
        Err(err) => serde_json::json!({ "error": err }),
    }
}

fn handle_request(ws: &Rc<RefCell<WorldState>>, req: RpcRequest) -> RpcResponse {
    println!("req {req:#?}");
    let result = match req.method.as_str() {
        "trove_call" => call_handler(|p| sandbox_call_handler(ws, p), req.params),
        "trove_defineContract" => call_handler(|p| define_contract_handler(ws, p), req.params),
        "trove_deployContract" => call_handler(|p| deploy_contract_handler(ws, p), req.params),
        "trove_sendTransaction" => call_handler(|p| send_transaction_handler(ws, p), req.params),
        _ => json!({"error": "method not found"}),
    };

    RpcResponse {
        jsonrpc: "2.0",
        result,
        id: req.id,
    }
}

fn handle_client(
    ws: &Rc<RefCell<WorldState>>,
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
) -> anyhow::Result<()> {
    while let Ok(msg) = websocket.read() {
        if msg.is_text() {
            let txt = msg.into_text()?;
            let req: RpcRequest = serde_json::from_str(&txt)?;
            let resp = handle_request(ws, req);
            let resp_text = serde_json::to_string(&resp)?;
            websocket.send(Message::Text(resp_text.into()))?;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    init_logging(3)?;
    let cli = CliArgs::parse_args();

    let world_state = if let Some(path) = &cli.state_file {
        if path.exists() {
            let json = fs::read_to_string(path)?;
            let ss: SerializableStorage = serde_json::from_str(&json).expect("from_str");
            let storage: InMemoryStorage = ss.into();
            WorldState::from_storage(storage)
        } else {
            WorldState::default()
        }
    } else {
        WorldState::default()
    };

    let listener = TcpListener::bind("127.0.0.1:4321")?;
    let world_state = Rc::new(RefCell::new(world_state));
    info!("Trove JSON-RPC node running on 127.0.0.1:4321");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut websocket = accept(stream)?;
                handle_client(&world_state, &mut websocket)?;
            }
            Err(err) => error!("Connection failed: {err}"),
        }
    }

    Ok(())
}
