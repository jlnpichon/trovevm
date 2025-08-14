use trovevm::core::vm::{Opcode, RuntimeError, VM, Value, function::CompiledFunction};

fn run_program<const N: usize>(bytecode: &[Opcode; N], constants: Vec<Value>) -> Value {
    let function = CompiledFunction::from((bytecode, constants));
    let mut vm = VM::new();
    vm.run(function.clone()).expect("run the program")
}

#[test]
fn test_push() {
    let value = run_program(&[Opcode::Push(0), Opcode::Return], vec![Value::Number(1.0)]);
    assert_eq!(value, Value::Number(1.0));

    let value = run_program(
        &[Opcode::Push(0), Opcode::Return],
        vec![Value::String(String::from("a string"))],
    );
    assert_eq!(value, Value::String(String::from("a string")));
}

#[test]
fn test_add() {
    let value = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Add,
            Opcode::Return,
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    );
    assert_eq!(value, Value::Number(3.0));
}

#[test]
fn test_sub_mul_div() {
    let value = run_program(
        &[
            Opcode::Push(0), // 5
            Opcode::Push(1), // 2
            Opcode::Sub,
            Opcode::Push(2), // 3
            Opcode::Mul,
            Opcode::Push(2), // 3
            Opcode::Div,
            Opcode::Return,
        ],
        vec![Value::Number(5.0), Value::Number(2.0), Value::Number(3.0)],
    );
    assert_eq!(value, Value::Number(3.0));
}

#[test]
fn test_neg() {
    let value = run_program(
        &[Opcode::Push(0), Opcode::Neg, Opcode::Return],
        vec![Value::Number(5.0)],
    );
    assert_eq!(value, Value::Number(-5.0));
}

#[test]
fn test_comparisons() {
    let constants = vec![Value::Number(1.0), Value::Number(2.0)];
    let value = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Lt, // 1 < 2
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));

    let value = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Le, // 1 <= 2
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));

    let value = run_program(
        &[
            Opcode::Push(1),
            Opcode::Push(0),
            Opcode::Gt, // 2 > 1
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));

    let value = run_program(
        &[
            Opcode::Push(1),
            Opcode::Push(0),
            Opcode::Ge, // 2 >= 1
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));

    let value = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(0),
            Opcode::Eq, // 1 == 1
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));

    let value = run_program(
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Neq, // 1 != 2
            Opcode::Return,
        ],
        constants.clone(),
    );
    assert_eq!(value, Value::Bool(true));
}

#[test]
fn test_pop() {
    let value = run_program(
        &[Opcode::Push(0), Opcode::Pop, Opcode::Return],
        vec![Value::Number(42.0)],
    );
    assert_eq!(value, Value::Null);
}

#[test]
fn test_invalid_constant_index() {
    let function = CompiledFunction::from((
        &[Opcode::Push(100), Opcode::Return],
        vec![Value::Number(1.0)],
    ));
    let mut vm = VM::new();
    let result = vm.run(function);
    assert!(matches!(
        result,
        Err(RuntimeError::InvalidConstantIndex(100))
    ));
}

#[test]
fn test_division_by_zero() {
    let function = CompiledFunction::from((
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::Div,
            Opcode::Return,
        ],
        vec![Value::Number(1.0), Value::Number(0.0)],
    ));
    let mut vm = VM::new();
    let result = vm.run(function);
    assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
}

#[test]
fn test_global_var() {
    let value = run_program(
        &[
            Opcode::Push(2),         // Null
            Opcode::DefineGlobal(0), // foo = Null
            Opcode::Push(0),         // foo
            Opcode::Push(1),         // 42
            Opcode::SetGlobal(0),    // foo = 42
            Opcode::Return,
        ],
        vec![
            Value::String("foo".into()),
            Value::Number(42.0),
            Value::Null,
        ],
    );
    assert_eq!(value, Value::Number(42.0));
}
