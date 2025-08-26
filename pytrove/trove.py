#!/usr/bin/env python3
import json
import asyncio
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


async def main():
    client = TroveClient("ws://127.0.0.1:4321")

    # Define contract
    with open("MyToken.trove", "r") as f:
        source = f.read()

    define_res = await client.define_contract(
        sender="0x2a", source_code=source, contract_name="MyToken"
    )
    if "error" in define_res["result"]:
        print(define_res["result"]["error"])
        return

    print("Contract defined:", define_res)

    ## Deploy contract
    deploy_res = await client.deploy_contract(
        sender="0x2a", bytecode_hash=define_res["result"]["bytecode_hash"]
    )
    if "error" in define_res:
        print(define_res["error"]["message"])
        return

    instance_address = deploy_res["result"]["instance_address"]
    print("Contract deployed at:", instance_address)

    ## Call / read
    balance = await client.call(
        caller="0x2a", callee=instance_address, method="get_balance", args=["42"]
    )
    print("Balance of 42:", balance["result"])

    ## Send transaction
    tx_res = await client.send_transaction(
        sender="0x2a", to=instance_address, method="mint", args=["42", "50"]
    )
    if "error" in define_res:
        print(define_res["error"]["message"])
        return

    print("Mint result:", tx_res["result"])

    ## Read again
    balance = await client.call(
        caller="0x2a", callee=instance_address, method="get_balance", args=["42"]
    )
    if "error" in define_res:
        print(define_res["error"]["message"])
        return

    print("Balance of 0x2a after mint:", balance["result"])


asyncio.run(main())
