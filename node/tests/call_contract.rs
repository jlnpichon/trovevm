use std::{cell::RefCell, rc::Rc};

use trove_core::{
    ContractEnv, RuntimeError, Value, WorldState,
    contract::{CompiledContract, ContractOrInstance},
};
use trove_vm::VM;
use trovec::{codegen::compile, parser::parse_program};

fn compile_contract(source: &str, contract_name: &str) -> CompiledContract {
    let mut ast = parse_program("<test>".to_string(), source)
        .unwrap_or_else(|_| panic!("parsing '{contract_name}' contract failed"));
    let compiled_program = compile(&mut ast.statements)
        .unwrap_or_else(|_| panic!("compiling '{contract_name}' contract failed"));
    compiled_program
        .contracts
        .get(contract_name)
        .unwrap_or_else(|| panic!("get '{contract_name}' compiled contract failed"))
        .clone()
}

fn compile_and_sandbox(
    source: &str,
    contract_name: &str,
    method_name: &str,
) -> Result<Value, RuntimeError> {
    let compiled_contract = compile_contract(source, contract_name);
    let mut vm = VM::new();
    vm.sandbox_call(
        ContractOrInstance::Contract(compiled_contract),
        method_name,
        None,
        None,
        vec![],
        None,
    )
}

#[test]
fn test_call_contract_simple() {
    let source = r#"
    contract Counter {
        let count;

        fn init() {
            this.count = 42;
        }

        fn get() {
            return this.count;
        }
    }
    "#;

    let value = compile_and_sandbox(source, "Counter", "get").expect("compile_and_sandbox failed");
    pretty_assertions::assert_eq!(value, Value::Number(42.0));
}

#[test]
fn test_call_contract_multiple() {
    let source = r#"
    contract Counter {
        let count;

        fn init() {
            this.count = 10;
        }

        fn increment() {
            this.count = this.count + 1;
        }

        fn get() {
            return this.count;
        }
    }
    "#;

    let compiled_contract = compile_contract(source, "Counter");

    let mut world_state = WorldState::default();

    let mut vm = VM::new();
    let instance = vm
        .deploy(0, compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    let env = ContractEnv {
        sender: 0,
        self_address: instance.address,
        instance,
        value: 0,
        block_number: 0,
        timestamp: 0,
    };

    for _ in 0..5 {
        vm.call_contract_method("increment", vec![], env.clone())
            .expect("call_contract_method");
    }
    let value = vm
        .call_contract_method("get", vec![], env)
        .expect("call_contract_method");
    pretty_assertions::assert_eq!(value, Value::Number(15.0));
}

#[test]
fn test_balance() {
    let source = r#"
    contract Wallet {
        fn get_sender_balance() {
            return msg.balance;
        }

        fn get_balance() {
            return this.balance;
        }
    }
    "#;

    let compiled_contract = compile_contract(source, "Wallet");

    let mut world_state = WorldState::default();

    let mut vm = VM::new();
    let instance = vm
        .deploy(0, compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    let sender_address = 42;

    world_state.storage.lock().set(
        sender_address,
        "balance",
        Rc::new(RefCell::new(Value::Number(1337.0))),
    );

    let env = ContractEnv {
        sender: sender_address,
        self_address: instance.address,
        instance,
        value: 0,
        block_number: 0,
        timestamp: 0,
    };

    let value = vm
        .call_contract_method("get_sender_balance", vec![], env.clone())
        .expect("call_contract_method");
    pretty_assertions::assert_eq!(value, Value::Number(1337.0));

    let value = vm
        .call_contract_method("get_balance", vec![], env.clone())
        .expect("call_contract_method");
    pretty_assertions::assert_eq!(value, Value::Number(0.0));
}

#[test]
fn token_map_test() {
    let source = r#"
    contract MyToken {
        let balances;

        fn init() {
            this.balances = map();
            this.balances[owner] = 1000;
        }

        fn transfer_to(from, to, amount) {
            let from_balance = this.balances[from];
            if (from_balance < amount) {
                error("Insufficient funds");
            }

            this.balances[from] = from_balance - amount;
            let to_balance = this.balances[to];
            this.balances[to] = to_balance + amount;
        }

        fn mint(to,  amount) {
            if (msg.sender != owner) {
                error("Only owner can mint");
            }

            let current = this.balances[to];
            this.balances[to] = current + amount;
            return amount;
        }

        fn burn(who, amount) {
            if (msg.sender != owner) {
                error("Only owner can mint");
            }

            let balance = this.balances[who];
            if (balance < amount) {
                error("Not enough funds for burning");
            }

            this.balances[who] = balance - amount;
            return amount;
        }

        fn get_balance(user) {
            return this.balances[user];
        }
    }
    "#;

    let compiled_contract = compile_contract(source, "MyToken");

    let bob = 42;
    let alice = 84;

    let mut world_state = WorldState::default();

    // Init sender balance
    world_state
        .storage
        .lock()
        .set(bob, "balance", Rc::new(RefCell::new(Value::Number(1000.0))));

    let mut vm = VM::new();
    let instance = vm
        .deploy(bob, compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    let env_owner = ContractEnv {
        sender: bob,
        self_address: instance.address,
        instance: instance.clone(),
        value: 0,
        block_number: 0,
        timestamp: 0,
    };

    // Transfer 100 from Bob to Alice
    vm.call_contract_method(
        "transfer_to",
        vec![
            Value::Number(bob as f64),
            Value::Number(alice as f64),
            Value::Number(100.0),
        ],
        env_owner.clone(),
    )
    .expect("transfer failed");

    // Check balances
    let owner_balance = vm
        .call_contract_method(
            "get_balance",
            vec![Value::Number(bob as f64)],
            env_owner.clone(),
        )
        .expect("get_balance failed");
    let alice_balance = vm
        .call_contract_method(
            "get_balance",
            vec![Value::Number(alice as f64)],
            env_owner.clone(),
        )
        .expect("get_balance failed");

    pretty_assertions::assert_eq!(owner_balance, Value::Number(900.0));
    pretty_assertions::assert_eq!(alice_balance, Value::Number(100.0));

    // Mint 50 for owner
    vm.call_contract_method(
        "mint",
        vec![Value::Number(bob as f64), Value::Number(50.0)],
        env_owner.clone(),
    )
    .expect("mint failed");

    // Check owner balacne
    let owner_balance = vm
        .call_contract_method(
            "get_balance",
            vec![Value::Number(bob as f64)],
            env_owner.clone(),
        )
        .expect("get_balance failed");

    pretty_assertions::assert_eq!(owner_balance, Value::Number(950.0));

    // Burn 10 of Alice
    vm.call_contract_method(
        "burn",
        vec![Value::Number(alice as f64), Value::Number(10.0)],
        env_owner.clone(),
    )
    .expect("burn failed");

    // Check Alice balance
    let alice_balance = vm
        .call_contract_method("get_balance", vec![Value::Number(alice as f64)], env_owner)
        .expect("get_balance failed");

    pretty_assertions::assert_eq!(alice_balance, Value::Number(90.0));
}

#[test]
fn test_transaction_failure_does_not_change_balances() {
    let source = r#"
    contract FailContract {
        fn fail_method() {
            error("Fail");
        }
    }
    "#;

    let compiled_contract = compile_contract(source, "FailContract");

    let bob = 42;

    let mut world_state = WorldState::default();

    // Init sender balance
    world_state
        .storage
        .lock()
        .set(bob, "balance", Rc::new(RefCell::new(Value::Number(1000.0))));

    let mut vm = VM::new();
    let instance = vm
        .deploy(bob, compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    // Init contract balance
    world_state.storage.lock().set(
        instance.address,
        "balance",
        Rc::new(RefCell::new(Value::Number(50.0))),
    );

    let env_owner = ContractEnv {
        sender: bob,
        self_address: instance.address,
        instance: instance.clone(),
        value: 200,
        block_number: 0,
        timestamp: 0,
    };

    let result = vm.call_contract_method("fail_method", vec![], env_owner.clone());

    assert!(
        matches!(result, Err(RuntimeError::ContractError(_))),
        "Expected call to fail with ContractError"
    );

    let storage = instance.storage.lock();
    let sender_balance = storage.get(env_owner.sender, "balance").unwrap();
    let contract_balance = storage.get(env_owner.instance.address, "balance").unwrap();

    assert_eq!(sender_balance, Rc::new(RefCell::new(Value::Number(1000.0))));
    assert_eq!(contract_balance, Rc::new(RefCell::new(Value::Number(50.0))));
}
