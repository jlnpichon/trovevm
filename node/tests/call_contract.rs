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
        .expect("get 'Counter' compiled contract failed")
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

    world_state
        .storage
        .lock()
        .set(sender_address, "balance", Value::Number(1337.0));

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
fn token_minimal_test() {
    let source = r#"
    contract MyToken {
        let balance_owner;
        let balance_alice;
        let total_supply;

        fn init() {
            this.total_supply = 1000;
            this.balance_owner = 1000;
            this.balance_alice = 0;
        }

        fn transfer_to_alice(amount) {
            if (this.balance_owner < amount) {
                error("Insufficient balance");
            }
            this.balance_owner = this.balance_owner - amount;
            this.balance_alice = this.balance_alice + amount;
        }

        fn mint_owner(amount) {
            this.total_supply = this.total_supply + amount;
            this.balance_owner = this.balance_owner + amount;
        }

        fn burn_owner(amount) {
            if (this.balance_owner < amount) {
                error("Insufficient balance");
            }
            this.balance_owner = this.balance_owner - amount;
            this.total_supply = this.total_supply - amount;
        }

        fn get_owner_balance() {
            return this.balance_owner;
        }

        fn get_alice_balance() {
            return this.balance_alice;
        }
    }
    "#;

    let compiled_contract = compile_contract(source, "MyToken");

    let mut world_state = WorldState::default();
    let mut vm = VM::new();
    let instance = vm
        .deploy(0, compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    let sender_address = 42;

    // Init sender balance
    world_state
        .storage
        .lock()
        .set(sender_address, "balance", Value::Number(1000.0));

    let env_owner = ContractEnv {
        sender: sender_address,
        self_address: instance.address,
        instance: instance.clone(),
        value: 0,
        block_number: 0,
        timestamp: 0,
    };

    // Transfert 100 from owner to Alice
    vm.call_contract_method(
        "transfer_to_alice",
        vec![Value::Number(100.0)],
        env_owner.clone(),
    )
    .expect("transfer failed");

    // Check balances
    let owner_balance = vm
        .call_contract_method("get_owner_balance", vec![], env_owner.clone())
        .expect("get_owner_balance failed");
    let alice_balance = vm
        .call_contract_method("get_alice_balance", vec![], env_owner.clone())
        .expect("get_alice_balance failed");

    pretty_assertions::assert_eq!(owner_balance, Value::Number(900.0));
    pretty_assertions::assert_eq!(alice_balance, Value::Number(100.0));

    // Mint 50 for owner
    vm.call_contract_method("mint_owner", vec![Value::Number(50.0)], env_owner.clone())
        .expect("mint failed");

    let owner_balance = vm
        .call_contract_method("get_owner_balance", vec![], env_owner.clone())
        .expect("get_owner_balance failed");

    pretty_assertions::assert_eq!(owner_balance, Value::Number(950.0));

    // Burn 200 of owner
    vm.call_contract_method("burn_owner", vec![Value::Number(200.0)], env_owner.clone())
        .expect("burn failed");

    let owner_balance = vm
        .call_contract_method("get_owner_balance", vec![], env_owner)
        .expect("get_owner_balance failed");

    pretty_assertions::assert_eq!(owner_balance, Value::Number(750.0));
}
