import json
import websockets


class TroveClient:
    def __init__(self, uri: str):
        self.uri = uri
        self.request_id = 1

    async def _rpc_call(self, method: str, params: list):
        self.request_id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        }
        async with websockets.connect(self.uri) as ws:
            await ws.send(json.dumps(request))
            response = await ws.recv()
            return json.loads(response)

    async def define_contract(self, sender: str, source_code: str, contract_name: str):
        req = {
            "sender": str(sender),
            "source_code": str(source_code),
            "contract_name": str(contract_name),
        }
        return await self._rpc_call("trove_defineContract", [req])

    async def deploy_contract(self, sender: str, bytecode_hash: str, args=None):
        if args is None:
            args = []
        req = {"sender": sender, "bytecode_hash": bytecode_hash, "args": args}
        return await self._rpc_call("trove_deployContract", [req])

    async def call(self, caller: str, callee: str, method: str, args=None, value=0):
        if args is None:
            args = []
        req = {
            "caller": str(caller),
            "callee": str(callee),
            "method": str(method),
            "args": args,
            "value": str(value),
        }
        return await self._rpc_call("trove_call", [req])

    async def send_transaction(
        self, sender: str, to: str, method: str, args=None, value=0
    ):
        if args is None:
            args = []
        req = {
            "from": str(sender),
            "to": str(to),
            "method": str(method),
            "args": args,
            "value": str(value),
        }
        return await self._rpc_call("trove_sendTransaction", [req])
