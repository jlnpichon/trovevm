# Trove Compiler — From TroveLang to Bytecode

`trove-compiler` is the **compiler crate** of the TroveVM project.  
It transforms **TroveLang** source code into **bytecode** executable by the
`trove-vm` stack-based virtual machine.

> ⚠️ TroveLang and its compiler are **toy implementations**, meant for
> learning and experimentation. They are **not safe for production**.

---

## Language Overview — TroveLang

TroveLang is a simple domain-specific language (DSL) for smart contracts.
Its design is inspired by educational languages like **clox** and minimalist
smart contract DSLs.

### Features

- **Functions**: define reusable logic with `fn`
- **Variables**: dynamically typed, stored in stack or global storage
- **Conditionals**: `if` / `else` statements
- **Loops**: basic `while` supported (no gas, can loop indefinitely)
- **Contracts**: groups of functions with their own storage

Example contract:

```trove
contract MyToken {
    let balances;

    fn init() {
        // Initialize balances map
        this.balances = map();
    }

    fn mint(to, amount) {
        this.balances[to] = this.balances[to] + amount;
    }

    fn get_balance(account) -> u64 {
        this.balances[account];
    }
}
```

> ⚠️ _Important_: Any contract storage (maps, counters, etc) **must
> be initialized** in `init`.

---

## Builtins

### Variables

The following variables are accessible inside contract's functions:

- `msg.sender`: The address of the caller/sender
- `msg.balance`: The balance of caller/sender
- `msg.value`: The transfer amount
- `owner`: The address of the contract's owner

### Functions

- `map()`: create a new dictionnary (HashMap<String, Value>)
- `error(String)`: throw an error to stop contract execution

## Current Limitations

- ❌ No type safety: values are dynamically typed
- ❌ No advanced optimizations
- ❌ No error recovery: parse / compile errors stop compilation
- ❌ No cryptography or security checks

## Quick Start

```rust
use trove_parser::parse_program;
use trove_compiler::codegen::compile;

// Parse TroveLang source
let program = parse_program("MyContract", source_code).unwrap();

// Compile to bytecode
let compiled = compile(&mut program.statements).unwrap();

// Get a specific contract
let contract = compiled.contracts.get("MyToken").unwrap();
let bytecode_hash = contract.code_hash;
```
