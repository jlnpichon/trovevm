use crate::core::{
    ast::{BinaryOp, Contract, Expr, Function, Literal, Statement, UnaryOp, Var, Visitor},
    vm::{Opcode, Program, Value},
};

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
    name: String,
    depth: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    program: Program,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn compile(statements: &[Statement]) -> Result<Program, CompileError> {
    let compiler = Compiler::new();
    compiler.compile(statements)
}

impl Compiler {
    pub fn compile(mut self, statements: &[Statement]) -> Result<Program, CompileError> {
        for statement in statements {
            self.visit_statement(statement);
        }
        Ok(self.program)
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Pop all the local variables
        while let Some(local) = self.locals.last() {
            match local.depth {
                Some(d) if d > self.scope_depth => {
                    self.program.emit_opcode(Opcode::Pop);
                    self.locals.pop();
                }
                _ => break,
            }
        }
    }

    fn add_local(&mut self, name: &str) -> Result<(), CompileError> {
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

        Ok(())
    }

    fn resolve_local(&self, name: &str) -> Result<usize, CompileError> {
        for (index, local) in self.locals.iter().rev().enumerate() {
            if name == local.name {
                match local.depth {
                    Some(_) => return Ok(index),
                    None => return Err(CompileError::VariableInOwnInitializer),
                }
            }
        }
        Err(CompileError::VariableNotFound(name.to_string()))
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
            Statement::Return(expr) => expr.accept(self),
            Statement::Expr(expr) => expr.accept(self),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.accept(self);
                then_branch.accept(self);
                if let Some(else_branch) = &else_branch {
                    else_branch.accept(self)
                }
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    initializer.accept(self);
                }
                if let Some(condition) = condition {
                    condition.accept(self);
                }
                if let Some(increment) = increment {
                    increment.accept(self);
                }
                body.accept(self);
            }
            Statement::While { condition, body } => {
                condition.accept(self);
                body.accept(self);
            }
        }
    }

    fn visit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Variable(var) => {
                if self.scope_depth == 0 {
                    let index = self.program.define_constant(Value::String(var.clone()));
                    self.program.emit_opcode(Opcode::GetGlobal(index));
                } else {
                    let index = self.resolve_local(var).expect("resolve_local");
                    self.program.emit_opcode(Opcode::GetLocal(index));
                }
            }
            Expr::Assign { target, value } => {
                target.accept(self);
                value.accept(self);
                self.program.emit_opcode(Opcode::SetGlobal);
            }
            Expr::FnCall { callee, args } => {
                callee.accept(self);
                for arg in args {
                    arg.accept(self);
                }
            }
            Expr::Unary { op, expr } => {
                expr.accept(self);
                match op {
                    UnaryOp::Not => todo!(),
                    UnaryOp::Minus => self.program.emit_opcode(Opcode::Neg),
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                lhs.accept(self);
                rhs.accept(self);
                match op {
                    BinaryOp::Add => self.program.emit_opcode(Opcode::Add),
                    BinaryOp::Substract => self.program.emit_opcode(Opcode::Sub),
                    BinaryOp::Multiply => self.program.emit_opcode(Opcode::Mul),
                    BinaryOp::Divide => self.program.emit_opcode(Opcode::Div),
                    BinaryOp::Modulo => self.program.emit_opcode(Opcode::Mod),
                    BinaryOp::Or => self.program.emit_opcode(Opcode::Or),
                    BinaryOp::And => self.program.emit_opcode(Opcode::And),
                    BinaryOp::Gt => self.program.emit_opcode(Opcode::Gt),
                    BinaryOp::Ge => self.program.emit_opcode(Opcode::Ge),
                    BinaryOp::Lt => self.program.emit_opcode(Opcode::Lt),
                    BinaryOp::Le => self.program.emit_opcode(Opcode::Le),
                    BinaryOp::Eq => self.program.emit_opcode(Opcode::Eq),
                    BinaryOp::Neq => self.program.emit_opcode(Opcode::Neq),
                }
            }
        }
    }

    fn visit_literal(&mut self, l: &Literal) {
        let value = Value::from(l);
        self.program.emit_constant(value);
    }

    fn visit_var_decl(&mut self, v: &Var) {
        if self.scope_depth > 0 {
            self.add_local(&v.name).expect("add_local");
        }

        if let Some(initializer) = &v.initializer {
            initializer.accept(self);
        } else {
            self.program.emit_null();
        }

        if self.scope_depth > 0 {
            self.mark_initialized();
        } else {
            self.program.define_global(&v.name);
        }
    }

    fn visit_contract_decl(&mut self, c: &Contract) {
        for var in &c.vars {
            self.visit_var_decl(var);
        }
        for func in &c.funcs {
            self.visit_fn_decl(func);
        }
    }

    fn visit_fn_decl(&mut self, f: &Function) {
        f.body.accept(self);
    }
}
