use trovevm::core::vm::{Opcode, Program, RuntimeError, VM, Value, function::CompiledFunction};

fn run_program<const N: usize>(
    bytecode: &[Opcode; N],
    constants: Vec<Value>,
) -> (VM, CompiledFunction) {
    let function = CompiledFunction::from((bytecode, constants));
    let mut vm = VM::new();
    vm.run(&function).expect("run the program");
    (vm, function)
}

fn assert_stack_top(vm: &VM, expected: &Value) {
    assert_eq!(vm.stack_top(), Some(expected));
}

#[test]
fn test_push() {
    let (vm, _) = run_program(&[Opcode::Push(0)], vec![Value::Number(1.0)]);
    assert_stack_top(&vm, &Value::Number(1.0));
    assert_eq!(vm.ip(), 1);

    let (vm, _) = run_program(
        &[Opcode::Push(0)],
        vec![Value::String(String::from("a string"))],
    );
    assert_stack_top(&vm, &Value::String(String::from("a string")));
    assert_eq!(vm.ip(), 1);
}

#[test]
fn test_add() {
    let (vm, _) = run_program(
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Add],
        vec![Value::Number(1.0), Value::Number(2.0)],
    );
    assert_stack_top(&vm, &Value::Number(3.0));
    assert_eq!(vm.ip(), 3);
}

#[test]
fn test_sub_mul_div() {
    let (vm, _) = run_program(
        &[
            Opcode::Push(0), // 5
            Opcode::Push(1), // 2
            Opcode::Sub,
            Opcode::Push(2), // 3
            Opcode::Mul,
            Opcode::Push(2), // 3
            Opcode::Div,
        ],
        vec![Value::Number(5.0), Value::Number(2.0), Value::Number(3.0)],
    );
    assert_stack_top(&vm, &Value::Number(3.0));
    assert_eq!(vm.ip(), 7);
}

#[test]
fn test_neg() {
    let (vm, _) = run_program(&[Opcode::Push(0), Opcode::Neg], vec![Value::Number(5.0)]);
    assert_stack_top(&vm, &Value::Number(-5.0));
    assert_eq!(vm.ip(), 2);
}

#[test]
fn test_comparisons() {
    let (mut vm, _) = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Lt, // 1 < 2
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Le, // 1 <= 2
            Opcode::Push(1),
            Opcode::Push(0),
            Opcode::Gt, // 2 > 1
            Opcode::Push(1),
            Opcode::Push(0),
            Opcode::Ge, // 2 >= 1
            Opcode::Push(0),
            Opcode::Push(0),
            Opcode::Eq, // 1 == 1
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Neq, // 1 != 2
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    );
    assert_eq!(vm.ip(), 18);

    let expected_results = [
        Value::Bool(true),
        Value::Bool(true),
        Value::Bool(true),
        Value::Bool(true),
        Value::Bool(true),
        Value::Bool(true),
    ];

    for expected in expected_results.iter().rev() {
        assert_eq!(vm.pop().unwrap(), *expected);
    }
}

#[test]
fn test_pop() {
    let (vm, _) = run_program(&[Opcode::Push(0), Opcode::Pop], vec![Value::Number(42.0)]);
    assert_eq!(vm.stack_top(), None);
}

#[test]
fn test_invalid_constant_index() {
    let function = CompiledFunction::from((&[Opcode::Push(100)], vec![Value::Number(1.0)]));
    let mut vm = VM::new();
    let result = vm.run(&function);
    assert!(matches!(
        result,
        Err(RuntimeError::InvalidConstantIndex(100))
    ));
}

#[test]
fn test_division_by_zero() {
    let function = CompiledFunction::from((
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Div],
        vec![Value::Number(1.0), Value::Number(0.0)],
    ));
    let mut vm = VM::new();
    let result = vm.run(&function);
    assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
}

#[test]
fn test_global_var() {
    let (vm, _) = run_program(
        &[
            Opcode::Push(2),         // Null
            Opcode::DefineGlobal(0), // foo = Null
            Opcode::Push(0),         // foo
            Opcode::Push(1),         // 42
            Opcode::SetGlobal(0),    // foo = 42
        ],
        vec![
            Value::String("foo".into()),
            Value::Number(42.0),
            Value::Null,
        ],
    );
    assert_stack_top(&vm, &Value::Number(42.0));
    assert_eq!(vm.ip(), 5);
}
