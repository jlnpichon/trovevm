# pytrove - Python client for TroveVM Node

`pytrove` is a Python client for interacting with a TroveVM JSON-RPC node via WebSocket.

It allows you to:

- Define and deploy smart contracts
- Call contract methods (read-only)
- Send transactions (state-changing calls)

---

## Installation

### 1. Create a virtual environment

```bash
python -m venv .venv
source .venv/bin/active
```

### 2. Install `pytrove` in editable mode

```bash
pip install -e .
```

This allows you to edit the source code and immediately see changes in your scripts.

---

## Usage

```python
from pytrove import TroveClient

# Connect to the local Trove node
client = TroveClient("ws://127.0.0.1:4321")

# Example: define a contract
source_code = open("examples/MyToken.trove").read()
define_res = await client.define_contract(
  sender="42",
  contract_name="MyToken",
  source_code=source_code
)

print("Contract defined:", define_res)
```

---

## Example Script

See [example.py](./example.py) for a complete lifecycle example:

1. Define a contract
2. Deploy a contract
3. Read state with a call
4. Send transactions

---

## Notes

- Ensure your Trove node is up and running
- Paths to contract files are correct
- Requires Python >= 3.10 for `asyncio.run`
