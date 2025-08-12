use trovevm::core::{
    ast::{BinaryOp, Expr, Literal, Statement, Var},
    codegen::Compiler,
    vm::{Opcode, Program, Value},
};

fn make_stmt_expr(expr: Expr) -> Statement {
    Statement::Expr(expr)
}

fn make_var_decl(name: &str, initializer: Option<Expr>) -> Statement {
    Statement::VarDecl(Var {
        name: name.into(),
        initializer,
    })
}

fn make_number_expr(n: f64) -> Expr {
    Expr::Literal(Literal::Number(n))
}

fn make_string_expr(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()))
}

fn make_null_expr() -> Expr {
    Expr::Literal(Literal::Null)
}

fn make_binary_op(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        op,
        lhs: lhs.into(),
        rhs: rhs.into(),
    }
}

fn assert_program(program: &Program, expected_bytecode: &[Opcode], expected_constants: &[Value]) {
    let (bytecode, constants) = program.as_slices();
    assert_eq!(
        bytecode.len(),
        expected_bytecode.len(),
        "Bytecode length mismatch"
    );
    assert_eq!(
        constants.len(),
        expected_constants.len(),
        "Constants length mismatch"
    );
    pretty_assertions::assert_eq!(bytecode, expected_bytecode);
    pretty_assertions::assert_eq!(constants, expected_constants, "Constants mismatch");
}

fn run_compiler(statements: &[Statement], bytecode: &[Opcode], constants: &[Value]) {
    let compiler = Compiler::new();
    let program = compiler.compile(statements).expect("compile ast");
    assert_program(&program, bytecode, constants);
}

#[test]
fn test_compile_literal() {
    let statement = make_stmt_expr(make_number_expr(42.0));
    run_compiler(&[statement], &[Opcode::Push(0)], &[Value::Number(42.0)]);

    let statement = make_stmt_expr(make_string_expr("a string"));
    run_compiler(
        &[statement],
        &[Opcode::Push(0)],
        &[Value::String(String::from("a string"))],
    );

    let statements = vec![
        make_stmt_expr(make_string_expr("a string")),
        make_stmt_expr(make_number_expr(42.42)),
        make_stmt_expr(make_null_expr()),
    ];
    run_compiler(
        &statements,
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Push(2)],
        &[
            Value::String(String::from("a string")),
            Value::Number(42.42),
            Value::Null,
        ],
    );
}

#[test]
fn test_compile_constant_reuse() {
    let statements = vec![
        make_stmt_expr(make_string_expr("a string")),
        make_stmt_expr(make_number_expr(42.0)),
        make_stmt_expr(make_string_expr("a string")),
    ];
    run_compiler(
        &statements,
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Push(0)],
        &[Value::String(String::from("a string")), Value::Number(42.0)],
    );
}

#[test]
fn test_compile_simple_addition() {
    let statement = make_stmt_expr(make_binary_op(
        BinaryOp::Add,
        make_number_expr(1.0),
        make_number_expr(2.0),
    ));

    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Add],
        &[Value::Number(1.0), Value::Number(2.0)],
    );
}

#[test]
fn test_var_decl() {
    let statement = make_var_decl("foo", Some(make_number_expr(42.0)));

    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::DefineGlobal(1)],
        &[Value::Number(42.0), Value::String("foo".into())],
    );
}
