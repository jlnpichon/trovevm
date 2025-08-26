# Trove Node

The `trove-node` crate is the RPC server for TroveVM.  
It allows deploying contracts, calling methods, and querying state via JSON-RPC.

## Usage

Run the node:

```bash
cargo run -p trove-node
```

By default, it listens on 127.0.0.1:8080.

## Example RPC call

Deploy a contract via Python client:

```python
from pytrove import TroveClient

client = TroveClient("http://127.0.0.1:4321")
addr = client.deploy_contract("my_contract.trove")
print("Contract deployed at:", addr)
```
