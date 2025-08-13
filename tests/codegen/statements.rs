use trovevm::core::{
    ast::{BinaryOp, Expr, Literal, Statement, Var},
    codegen::compile,
    vm::{Opcode, Program, Value},
};

fn make_stmt_expr(expr: Expr) -> Statement {
    Statement::Expr(expr)
}

fn make_block(statements: Vec<Statement>) -> Statement {
    Statement::Block(statements)
}

fn make_var_decl(name: &str, initializer: Option<Expr>) -> Statement {
    Statement::VarDecl(Var {
        name: name.into(),
        initializer,
    })
}

fn make_assignment(target: Expr, value: Expr) -> Expr {
    Expr::Assign {
        target: target.into(),
        value: value.into(),
    }
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
    let program = compile(statements).expect("compile ast");
    assert_program(&program, bytecode, constants);
}

#[test]
fn test_compile_literal() {
    // 42.0;
    let statement = make_stmt_expr(make_number_expr(42.0));
    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::Pop],
        &[Value::Number(42.0)],
    );

    // "a string";
    let statement = make_stmt_expr(make_string_expr("a string"));
    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::Pop],
        &[Value::String(String::from("a string"))],
    );

    // "a string";
    // 42.42;
    // Null;
    let statements = vec![
        make_stmt_expr(make_string_expr("a string")),
        make_stmt_expr(make_number_expr(42.42)),
        make_stmt_expr(make_null_expr()),
    ];
    run_compiler(
        &statements,
        &[
            Opcode::Push(0),
            Opcode::Pop,
            Opcode::Push(1),
            Opcode::Pop,
            Opcode::Push(2),
            Opcode::Pop,
        ],
        &[
            Value::String(String::from("a string")),
            Value::Number(42.42),
            Value::Null,
        ],
    );
}

#[test]
fn test_compile_constant_reuse() {
    // "a string";
    // 42.0;
    // "a string";
    let statements = vec![
        make_stmt_expr(make_string_expr("a string")),
        make_stmt_expr(make_number_expr(42.0)),
        make_stmt_expr(make_string_expr("a string")),
    ];
    run_compiler(
        &statements,
        &[
            Opcode::Push(0),
            Opcode::Pop,
            Opcode::Push(1),
            Opcode::Pop,
            Opcode::Push(0),
            Opcode::Pop,
        ],
        &[Value::String(String::from("a string")), Value::Number(42.0)],
    );
}

#[test]
fn test_compile_simple_addition() {
    // 1.0 + 2.0;
    let statement = make_stmt_expr(make_binary_op(
        BinaryOp::Add,
        make_number_expr(1.0),
        make_number_expr(2.0),
    ));

    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::Push(1), Opcode::Add, Opcode::Pop],
        &[Value::Number(1.0), Value::Number(2.0)],
    );
}

#[test]
fn test_global_var() {
    let statement = make_var_decl("foo", Some(make_number_expr(42.0)));

    run_compiler(
        &[statement],
        &[Opcode::Push(0), Opcode::DefineGlobal(1)],
        &[Value::Number(42.0), Value::String("foo".into())],
    );
}

#[test]
fn test_local_var() {
    // {
    //   let foo;
    //   {
    //     let bar = 42.0;
    //     foo = bar;
    //   }
    // }
    let statement = make_block(vec![
        make_var_decl("foo", None),
        make_block(vec![
            make_var_decl("bar", Some(make_number_expr(42.0))),
            make_stmt_expr(make_assignment(
                Expr::Variable("foo".to_string()),
                Expr::Variable("bar".to_string()),
            )),
        ]),
    ]);

    run_compiler(
        &[statement],
        &[
            Opcode::Push(0),
            Opcode::Push(1),
            Opcode::GetLocal(1),
            Opcode::SetLocal(0),
            Opcode::Pop, // expr statement pushed 42.0 (foo = bar)
            Opcode::Pop, // foo
            Opcode::Pop, // foo
        ],
        &[Value::Null, Value::Number(42.0)],
    );
}
