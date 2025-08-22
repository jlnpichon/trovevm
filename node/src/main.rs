use std::net::SocketAddr;

use jsonrpsee::{server::ServerBuilder, tracing::info};

use logging::init_logging;
use rpc_server::{TroveRcpServerImpl, TroveRpcServer};

mod logging;
mod rpc_server;
mod serialization;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging(3)?;

    let server = ServerBuilder::default()
        .build("127.0.0.1:4321".parse::<SocketAddr>()?)
        .await?;
    let local_addr = server.local_addr();

    let server_impl = TroveRcpServerImpl::new();
    let handle = server.start(server_impl.into_rpc());
    info!("Server running on {local_addr:?}");

    handle.stopped().await;

    Ok(())
}
