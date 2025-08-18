use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Contract, Expr, Function, Identifier, Literal, Statement, UnaryOp, Var, Visitor,
};
use trove_core::{CompiledFunction, Opcode, Value};

const MAX_LOCAL_VARS: usize = 255;

#[derive(Debug, thiserror::Error, Clone)]
#[error("compile error")]
pub enum CompileError {
    #[error("Too many local variables")]
    TooManyLocalVars,
    #[error("A variable with the same name already exists in this scope")]
    VariableRedeclaration,
    #[error("Variable '{0}' not found")]
    VariableNotFound(String),
    #[error("Variable used in its own initializer")]
    VariableInOwnInitializer,
}

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    function: CompiledFunction,
    functions: HashMap<String, CompiledFunction>,
    contracts: HashMap<String, CompiledContract>,
    locals: Vec<Local>,
    scope_depth: usize,
}

#[derive(Debug, Clone)]
pub struct CompiledContract {
    name: String,
    functions: HashMap<String, CompiledFunction>,
    vars: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub contracts: HashMap<String, CompiledContract>,
    pub script: CompiledFunction,
}

impl Compiler {
    pub fn new(name: &str) -> Self {
        Self {
            function: CompiledFunction::new(name),
            ..Default::default()
        }
    }
}

pub fn compile(statements: &[Statement]) -> Result<CompiledProgram, CompileError> {
    let mut compiler = Compiler::new("script");
    compiler.compile(statements)?;
    Ok(compiler.end_program())
}

impl Compiler {
    pub fn compile(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            self.visit_statement(statement);
        }
        Ok(())
    }

    pub fn end_function(mut self) -> CompiledFunction {
        self.emit_return();
        self.function
    }

    pub fn end_program(self) -> CompiledProgram {
        let contracts = self.contracts.clone();
        let script = self.end_function();
        CompiledProgram { contracts, script }
    }

    pub fn end_contract(mut self, name: &str, vars: Vec<String>) -> CompiledContract {
        self.emit_return();
        CompiledContract {
            name: name.to_string(),
            functions: std::mem::take(&mut self.functions),
            vars,
        }
    }

    pub fn function_map(&self) -> HashMap<String, CompiledFunction> {
        self.functions.clone()
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn pop_locals(&mut self) {
        while let Some(local) = self.locals.last() {
            match local.depth {
                Some(d) if d >= self.scope_depth => {
                    self.emit_opcode(Opcode::Pop);
                    self.locals.pop();
                }
                _ => break,
            }
        }
    }

    pub fn end_scope(&mut self) {
        self.pop_locals();
        self.scope_depth -= 1;
    }

    fn add_local(&mut self, name: &str) -> Result<usize, CompileError> {
        if self.locals.len() > MAX_LOCAL_VARS {
            return Err(CompileError::TooManyLocalVars);
        }

        for local in self.locals.iter().rev() {
            match local.depth {
                Some(d) if d < self.scope_depth => break,
                _ => {}
            }

            if local.name == name {
                return Err(CompileError::VariableRedeclaration);
            }
        }

        self.locals.push(Local {
            name: name.to_string(),
            depth: None,
        });

        Ok(self.locals.len() - 1)
    }

    fn resolve_local(&self, name: &str) -> Result<Option<usize>, CompileError> {
        for (i_rev, local) in self.locals.iter().rev().enumerate() {
            if name == local.name {
                if local.depth.is_some() {
                    let index = self.locals.len() - 1 - i_rev;
                    return Ok(Some(index));
                } else {
                    return Err(CompileError::VariableInOwnInitializer);
                }
            }
        }
        Ok(None)
    }

    fn define_constant(&mut self, value: Value) -> usize {
        self.function.program_mut().define_constant(value)
    }

    fn define_global(&mut self, name: &Identifier) -> usize {
        self.function.program_mut().define_global(name)
    }

    fn emit_opcode(&mut self, opcode: Opcode) {
        self.function.program_mut().emit_opcode(opcode);
    }

    fn emit_return(&mut self) {
        if self.scope_depth > 0 {
            let index = self.define_constant(Value::Null);
            self.emit_opcode(Opcode::Push(index));
        }
        self.emit_opcode(Opcode::Return);
    }
    fn emit_jump(&mut self, jump: Opcode) -> usize {
        self.function.program_mut().emit_jump(jump)
    }

    fn current_opcode_index(&self) -> usize {
        self.function.program().current_opcode_index()
    }

    fn patch_jump(&mut self, index: usize, offset: usize) {
        self.function.program_mut().patch_jump(index, offset);
    }

    fn emit_constant(&mut self, value: Value) {
        self.function.program_mut().emit_constant(value);
    }

    fn emit_null(&mut self) {
        self.function.program_mut().emit_null();
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        if let Some(last) = self.locals.last_mut() {
            last.depth = Some(self.scope_depth);
        }
    }
}

impl Visitor for Compiler {
    fn visit_statement(&mut self, s: &Statement) {
        match s {
            Statement::ContractDecl(contract) => {
                self.visit_contract_decl(contract);
            }
            Statement::FnDecl(function) => {
                self.visit_fn_decl(function);
            }
            Statement::VarDecl(var) => {
                self.visit_var_decl(var);
            }
            Statement::Block(statements) => {
                self.begin_scope();
                for statement in statements {
                    statement.accept(self);
                }
                self.end_scope();
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    expr.accept(self);
                    self.pop_locals();
                    self.emit_opcode(Opcode::Return);
                } else {
                    self.pop_locals();
                    self.emit_return();
                }
            }
            Statement::Expr(expr) => {
                expr.accept(self);
                self.emit_opcode(Opcode::Pop);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.accept(self);
                let jump_to_else = self.emit_jump(Opcode::JumpIfFalse(0));
                self.emit_opcode(Opcode::Pop); // Pop the condition

                then_branch.accept(self);
                let mut jump_to_end = 0;
                if else_branch.is_some() {
                    jump_to_end = self.emit_jump(Opcode::Jump(0));
                }

                let else_offset = self.current_opcode_index();
                self.patch_jump(jump_to_else, else_offset - jump_to_else);

                if let Some(else_branch) = &else_branch {
                    self.emit_opcode(Opcode::Pop); // Pop the condition
                    else_branch.accept(self)
                }

                if else_branch.is_some() {
                    let end_offset = self.current_opcode_index();
                    self.patch_jump(jump_to_end, end_offset - jump_to_end);
                }
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                let mut start = self.current_opcode_index();
                let mut exit_jump = None;

                if let Some(initializer) = initializer {
                    initializer.accept(self);
                }

                if let Some(condition) = condition {
                    condition.accept(self);

                    exit_jump = Some(self.emit_jump(Opcode::JumpIfFalse(0)));
                    self.emit_opcode(Opcode::Pop); // condition
                }

                if let Some(increment) = increment {
                    let body_jump = self.emit_jump(Opcode::Jump(0));
                    let increment_start = self.current_opcode_index();

                    increment.accept(self);
                    self.emit_opcode(Opcode::Pop);

                    self.emit_opcode(Opcode::JumpBack(self.current_opcode_index() - start - 1)); // to condition

                    start = increment_start;
                    self.patch_jump(body_jump, self.current_opcode_index() - body_jump);
                }

                body.accept(self);

                self.emit_opcode(Opcode::JumpBack(self.current_opcode_index() - start)); // to increment

                if let Some(exit_jump) = exit_jump {
                    let current = self.current_opcode_index();
                    self.patch_jump(exit_jump, current - exit_jump);
                }

                self.emit_opcode(Opcode::Pop); // condition
                self.end_scope();
            }
            Statement::While { condition, body } => {
                let start = self.current_opcode_index();
                condition.accept(self);

                let jump_end = self.emit_jump(Opcode::JumpIfFalse(0));
                self.emit_opcode(Opcode::Pop); // condition

                body.accept(self);

                self.emit_jump(Opcode::JumpBack(self.current_opcode_index() - start));

                let end = self.current_opcode_index();
                self.patch_jump(jump_end, end - jump_end);

                self.emit_opcode(Opcode::Pop); // condition
            }
        }
    }

    fn visit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Variable(var) => {
                if let Some(index) = self.resolve_local(var).expect("resolve_local") {
                    self.emit_opcode(Opcode::GetLocal(index));
                } else {
                    let index = self.define_constant(Value::String(var.clone()));
                    self.emit_opcode(Opcode::GetGlobal(index));
                }
            }
            Expr::Get { object, name } => {
                self.visit_expr(object);
                let index = self.define_constant(Value::String(name.clone()));
                self.emit_opcode(Opcode::GetField(index));
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                value.accept(self);
                object.accept(self);

                let index = self.define_constant(Value::String(name.clone()));
                self.emit_opcode(Opcode::SetField(index));
            }
            Expr::Assign { target, value } => {
                value.accept(self);

                if let Expr::Variable(name) = target.as_ref() {
                    if let Some(index) = self.resolve_local(name).expect("resolve_local") {
                        self.emit_opcode(Opcode::SetLocal(index));
                    } else {
                        let index = self.define_constant(Value::String(name.clone()));
                        self.emit_opcode(Opcode::SetGlobal(index));
                    }
                } else {
                    panic!("Unsupported assignment target {target:?}");
                }
            }
            Expr::FnCall { callee, args } => {
                callee.accept(self);

                for arg in args {
                    arg.accept(self);
                }

                self.emit_opcode(Opcode::Call(args.len()));
            }
            Expr::Unary { op, expr } => {
                expr.accept(self);
                match op {
                    UnaryOp::Not => todo!(),
                    UnaryOp::Minus => self.emit_opcode(Opcode::Neg),
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                lhs.accept(self);

                match op {
                    BinaryOp::And => {
                        let jump_end = self.emit_jump(Opcode::JumpIfFalse(0));
                        self.emit_opcode(Opcode::Pop);
                        rhs.accept(self);

                        let end = self.current_opcode_index();
                        self.patch_jump(jump_end, end - jump_end - 1);
                    }
                    BinaryOp::Or => {
                        let jump_else = self.emit_jump(Opcode::JumpIfFalse(0));
                        let jump_end = self.emit_jump(Opcode::Jump(0));
                        self.emit_opcode(Opcode::Pop);

                        let current = self.current_opcode_index();
                        self.patch_jump(jump_else, current - jump_else - 1);

                        rhs.accept(self);
                        let end = self.current_opcode_index();
                        self.patch_jump(jump_end, end - jump_end);
                    }
                    _ => {
                        rhs.accept(self);
                        let opcode = match op {
                            BinaryOp::Add => Opcode::Add,
                            BinaryOp::Substract => Opcode::Sub,
                            BinaryOp::Multiply => Opcode::Mul,
                            BinaryOp::Divide => Opcode::Div,
                            BinaryOp::Modulo => Opcode::Mod,
                            BinaryOp::Gt => Opcode::Gt,
                            BinaryOp::Ge => Opcode::Ge,
                            BinaryOp::Lt => Opcode::Lt,
                            BinaryOp::Le => Opcode::Le,
                            BinaryOp::Eq => Opcode::Eq,
                            BinaryOp::Neq => Opcode::Neq,
                            _ => unreachable!(),
                        };
                        self.emit_opcode(opcode);
                    }
                }
            }
        }
    }

    fn visit_literal(&mut self, l: &Literal) {
        let value = Value::from(l);
        self.emit_constant(value);
    }

    fn visit_var_decl(&mut self, v: &Var) {
        if self.scope_depth > 0 {
            self.add_local(&v.name).expect("add_local");
        }

        if let Some(initializer) = &v.initializer {
            initializer.accept(self);
        } else {
            self.emit_null();
        }

        if self.scope_depth > 0 {
            self.mark_initialized();
        } else {
            self.define_global(&v.name);
        }
    }

    fn visit_contract_decl(&mut self, c: &Contract) {
        let mut contract_compiler = Compiler::new(&c.name);

        contract_compiler.begin_scope();
        // Initializers are forbidden
        for var in &c.vars {
            self.add_local(&var.name).expect("add_local");
            self.mark_initialized();
        }
        for func in &c.funcs {
            contract_compiler.visit_fn_decl(func);
        }
        contract_compiler.end_scope();

        let compiled_contract = contract_compiler
            .end_contract(&c.name, c.vars.iter().map(|v| v.name.to_string()).collect());

        self.contracts
            .insert(compiled_contract.name.clone(), compiled_contract);
    }

    fn visit_fn_decl(&mut self, f: &Function) {
        let mut compiler = Compiler::new(&f.name);

        compiler.begin_scope();
        for param in &f.params {
            compiler.add_local(param).expect("add_local");
            compiler.mark_initialized();
        }

        compiler
            .compile(std::slice::from_ref(&*f.body))
            .expect("compile");

        let mut func_obj = compiler.end_function();
        func_obj.arity = f.params.len();

        self.functions.insert(f.name.to_string(), func_obj.clone());
        let index = self.define_constant(Value::Function(func_obj));

        if self.scope_depth == 0 {
            self.emit_opcode(Opcode::Push(index));
            let name_index = self.define_constant(Value::String(f.name.to_string()));
            self.emit_opcode(Opcode::DefineGlobal(name_index));
        } else {
            self.add_local(&f.name).expect("add_local");
            self.mark_initialized();
            self.emit_opcode(Opcode::Push(index));
            self.emit_opcode(Opcode::SetLocal(index));
        }
    }
}

impl From<&Literal> for Value {
    fn from(literal: &Literal) -> Self {
        match literal {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.to_string()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }
}
