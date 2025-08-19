use trove_core::{RuntimeError, Value, WorldState, contract::CompiledContract, world_state};
use trove_vm::VM;
use trovec::{codegen::compile, parser::parse_program};

fn compile_contract(source: &str, contract_name: &str) -> CompiledContract {
    let ast = parse_program("<test>".to_string(), source)
        .unwrap_or_else(|_| panic!("compiling '{contract_name}' contract failed"));
    let compiled_program = compile(&ast.statements)
        .unwrap_or_else(|_| panic!("compiling '{contract_name}' contract failed"));
    compiled_program
        .contracts
        .get(contract_name)
        .expect("get 'Counter' compiled contract failed")
        .clone()
}

fn compile_and_sandbox(source: &str, contract_name: &str) -> Result<Value, RuntimeError> {
    let compiled_contract = compile_contract(source, contract_name);
    let mut vm = VM::new();
    vm.sandbox_call(compiled_contract, "get", vec![], None)
}

#[test]
fn call_contract_simple() {
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

    let value = compile_and_sandbox(source, "Counter").expect("compile_and_sandbox failed");
    pretty_assertions::assert_eq!(value, Value::Number(42.0));
}

#[test]
fn call_contract_multiple() {
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
        .deploy(compiled_contract, vec![], &mut world_state)
        .expect("deploy failed");

    for _ in 0..5 {
        vm.call_contract_method(instance.clone(), "increment", vec![])
            .expect("call_contract_method");
    }
    let value = vm
        .call_contract_method(instance.clone(), "get", vec![])
        .expect("call_contract_method");
    pretty_assertions::assert_eq!(value, Value::Number(15.0));
}
