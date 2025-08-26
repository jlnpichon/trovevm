#!/usr/bin/env python3
import asyncio
from pathlib import Path

from pytrove import TroveClient


def check_error(res):
    if "error" in res:
        print(res["error"]["message"])
        return True
    return False


async def main():
    client = TroveClient("ws://127.0.0.1:4321")

    examples_dir = Path(__file__).parent.parent / "examples"
    with open(examples_dir / "MyToken.trove", "r") as f:
        source = f.read()

    # Define contract
    define_res = await client.define_contract(
        sender="0x2a", source_code=source, contract_name="MyToken"
    )
    if check_error(define_res):
        return

    print("Contract defined:", define_res)

    # Deploy contract
    deploy_res = await client.deploy_contract(
        sender="0x2a", bytecode_hash=define_res["result"]["bytecode_hash"]
    )
    if check_error(deploy_res):
        return

    instance_address = deploy_res["result"]["instance_address"]
    print("Contract deployed at:", instance_address)

    # Call / read
    call_res = await client.call(
        caller="0x2a", callee=instance_address, method="get_balance", args=["42"]
    )
    if check_error(call_res):
        return

    print("Balance of 42:", call_res["result"])

    # Send transaction
    tx_res = await client.send_transaction(
        sender="0x2a", to=instance_address, method="mint", args=["42", "50"]
    )
    if check_error(tx_res):
        return

    print("Mint result:", tx_res["result"])

    # Read again
    call_res = await client.call(
        caller="0x2a", callee=instance_address, method="get_balance", args=["42"]
    )
    if check_error(call_res):
        return

    print("Balance of 0x2a after mint:", call_res["result"])


asyncio.run(main())
