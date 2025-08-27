# [TroveVM - A Minimalist Smart Contract VM Playground]

[![CI](https://github.com/jlnpichon/trovevm/actions/workflows/ci.yml/badge.svg)](https://github.com/jlnpichon/trovevm/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A lightweight and modular **toy Virtual Machine (VM)** designed for executing
smart contracts, implemented in Rust. This project aims to provide a simple
yet extensible playground to experiment with smart contract logic, opcode
design, and blockchain virtual machine concepts.

Inspired by the Ethereum Virtual Machine (EVM), it demonstrates how to:

- Parse a contract DSL,
- Compile it to custom bytecode,
- Run it in a deterministic, gas-metered VM,
- Interact with a mock blockchain runtime (accounts, balances, sender),
- Simulate a mini blockchain network via Docker Compose.

> ⚠️ This project is a toy. It's not safe for production

---

## Features

- 📦 **Stack-based VM** with custom opcodes
- 🧾 **Smart contract language** with functions, conditionals, variables
- 🧍 **Runtime with accounts & balances**
- 🐳 **Docker Compose** demo simulating a mini blockchain
- 🎯 Clean Rust architecture: modular, idiomatic, testable

---

## Goals

This project aims to provide a simple and modular smart contract
Virtual Machine (VM) that serves both as:

- An educational tool to help developers understand smart contract execution at
  a low level, with easy-to-follow code and concepts.
- A flexible foundation to build more advanced blockchain runtimes by enabling
  modular extensions and new features.
- The VM offers a sandbox environment for testing and experimentation.

---

## Architecture

```text
TroveLang Source (.trove)
 ↓
[Parser] (trove-parse) → Abstract Syntax Tree (AST)
 ↓
[Compiler] → (trove-vm/codegen) → Custom Bytecode
 ↓
[VM] (trove-vm) → Stack execution
 ↓
[Node] (trove-node) → Expose JSON-RPC over Websocket
```

---

## Getting Started

1. Clone the repo
2. Build with `cargo build`
3. Run example contracts in the sandbox environment
4. Extend with your own opcodes and contract logic

---

## 🐳 Run the full blockchain demo

### 1. Build and start TroveNode

This will build and start a TroveNode inside a Docker instance and allow you
to interact with it from your host via websocket/JSON-RPC.

```bash
docker-compose up --build
```

### 2. Run the Python client example

Make sure you have a virtual environment and `pytrove` installed

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e pytrove
```

Run the full lifecycle example:

```bash
python pytrove/example.py
```

### 3. Stop the node

```bash
docker-compose down
```

---

## 📂 Examples

See the examples/ folder for some .trove source files.

---

## ✨ Why this project?

This repo was created as a playground to:

- Learn how smart contract execution really works,
- Sharpen my Rust skills,
- Explore compilers, VMs, and secure execution models,

---

## Crates

- [`trove-core`](./core): Core types and definitions
- [`trove-parser`](./parser): TroveLang parser
- [`trove-vm`](./vm): Virtual Machine execution engine
- [`trove-cli`](./cli): Command-line interface
- [`trove-node`](./node): RPC server for TroveVM
- [`pytrove`](./pytrove): Python client for Trove Node
