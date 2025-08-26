# TroveVM — Stack-Based Smart Contract Virtual Machine

`trove-vm` is the **execution engine** of the TroveVM project,
implementing a minimalistic **stack-based virtual machine** for smart contracts.

This crate provides a simple and modular environment to run bytecode produced
by `trove-parser` + `trove-compiler`.

> ⚠️ This VM is a **toy implementation**. It is **not secure**, does
> **not implement gas metering**, and is intended for learning, experimentation,
> and prototyping only.

---

## Design Overview

The VM is inspired by educational VMs such as [clox](https://craftinginterpreters.com), with a focus on simplicity and clarity.

### Stack-Based Execution

- Uses a **value stack** (`Vec<Value>`) to evaluate expressions.
- Supports **basic operations**: arithmetic, comparisons, conditionals,
  function calls.
- Variables are currently stored in **global or local environments**.
- Bytecode is executed sequentially, with a program counter (`ip`) pointing to
  the current instruction.

### Contracts

- Contracts are represented as `ContractInstance`, containing:
  - `address`: unique identifier
  - `code`: compiled bytecode
  - `storage`: key-value store (simulated blockchain storage)
- Contracts can be deployed, called (read-only), or invoked via transactions (state-modifying).

### Environment

Each VM execution receives a `ContractEnv`:

- `sender`: the origin of the call
- `self_address`: the current contract address
- `value`: attached numeric value (like ETH)
- `block_number`, `timestamp`: placeholder fields for blockchain simulation
- `instance`: reference to the contract instance

---

## Current Features

- Stack-based VM with basic opcodes
- Local and global variable support
- Contract deployment and instantiation
- Read-only `call` and state-modifying `send_transaction`
- Integration with `trove-core` for addresses, storage, and world state

---

## Limitations / Warnings

- ❌ **No gas or resource metering**: contracts can loop indefinitely
- ❌ **No cryptography**: all addresses, balances, and storage are naive
- ❌ **No type safety**: values are dynamically typed (`Value` enum)
- ❌ **No concurrency**: single-threaded execution
- ❌ **Not safe for production**: this VM is for learning and experimentation only

---

## Quick Start

```rust
use trove_core::{WorldState, Address};
use trove_vm::VM;

// Create a world state
let mut world_state = WorldState::default();

// Deploy and call contracts
let mut vm = VM::new();
let contract = world_state.get_contract_from_hash(bytecode_hash).unwrap();
let instance = vm.deploy(sender_address, contract, args, &mut world_state).unwrap();

let result = vm.call_contract_method("method_name", vec![], env).unwrap();

```

---
