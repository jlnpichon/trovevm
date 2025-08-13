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
        for (i_rev, local) in self.locals.iter().rev().enumerate() {
            if name == local.name {
                match local.depth {
                    Some(_) => {
                        let index = self.locals.len() - 1 - i_rev;
                        return Ok(index);
                    }
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
            Statement::Expr(expr) => {
                expr.accept(self);
                self.program.emit_opcode(Opcode::Pop);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.accept(self);
                let jump_to_else = self.program.emit_jump(Opcode::JumpIfFalse(0));
                self.program.emit_opcode(Opcode::Pop); // Pop the condition

                then_branch.accept(self);
                let mut jump_to_end = 0;
                if else_branch.is_some() {
                    jump_to_end = self.program.emit_jump(Opcode::Jump(0));
                }

                let else_offset = self.program.current_opcode_index();
                self.program
                    .patch_jump(jump_to_else, else_offset - jump_to_else - 1);

                if let Some(else_branch) = &else_branch {
                    self.program.emit_opcode(Opcode::Pop); // Pop the condition
                    else_branch.accept(self)
                }

                if else_branch.is_some() {
                    let end_offset = self.program.current_opcode_index();
                    self.program
                        .patch_jump(jump_to_end, end_offset - jump_to_end - 1);
                }
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                let mut start = self.program.current_opcode_index();
                let mut exit_jump = None;

                if let Some(initializer) = initializer {
                    initializer.accept(self);
                }

                if let Some(condition) = condition {
                    condition.accept(self);

                    exit_jump = Some(self.program.emit_jump(Opcode::JumpIfFalse(0)));
                    self.program.emit_opcode(Opcode::Pop); // condition
                }

                if let Some(increment) = increment {
                    let body_jump = self.program.emit_jump(Opcode::Jump(0));
                    let increment_start = self.program.current_opcode_index();

                    increment.accept(self);
                    self.program.emit_opcode(Opcode::Pop);

                    self.program.emit_opcode(Opcode::JumpBack(
                        self.program.current_opcode_index() - start,
                    ));
                    start = increment_start;
                    self.program
                        .patch_jump(body_jump, self.program.current_opcode_index() - body_jump);
                }
                body.accept(self);

                self.program.emit_opcode(Opcode::JumpBack(
                    self.program.current_opcode_index() - start,
                ));

                if let Some(exit_jump) = exit_jump {
                    let current = self.program.current_opcode_index();
                    self.program.patch_jump(exit_jump, current - exit_jump);
                    self.program.emit_opcode(Opcode::Pop); // condition
                }

                self.end_scope();
            }
            Statement::While { condition, body } => {
                let start = self.program.current_opcode_index();
                condition.accept(self);

                let jump_end = self.program.emit_jump(Opcode::JumpIfFalse(0));

                body.accept(self);

                self.program.emit_jump(Opcode::JumpBack(
                    self.program.current_opcode_index() - start,
                ));

                let end = self.program.current_opcode_index();
                self.program.patch_jump(jump_end, end - jump_end);
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
                value.accept(self);

                if let Expr::Variable(name) = target.as_ref() {
                    if self.scope_depth == 0 {
                        let index = self.program.define_constant(Value::String(name.clone()));
                        self.program.emit_opcode(Opcode::SetGlobal(index));
                    } else {
                        let index = self.resolve_local(name).expect("resolve_local");
                        self.program.emit_opcode(Opcode::SetLocal(index));
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

                match op {
                    BinaryOp::And => {
                        let jump_end = self.program.emit_jump(Opcode::JumpIfFalse(0));
                        self.program.emit_opcode(Opcode::Pop);
                        rhs.accept(self);

                        let end = self.program.current_opcode_index();
                        self.program.patch_jump(jump_end, end - jump_end - 1);
                    }
                    BinaryOp::Or => {
                        let jump_else = self.program.emit_jump(Opcode::JumpIfFalse(0));
                        let jump_end = self.program.emit_jump(Opcode::Jump(0));
                        self.program.emit_opcode(Opcode::Pop);

                        let current = self.program.current_opcode_index();
                        self.program.patch_jump(jump_else, current - jump_else - 1);

                        rhs.accept(self);
                        let end = self.program.current_opcode_index();
                        self.program.patch_jump(jump_end, end - jump_end);
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
                        self.program.emit_opcode(opcode);
                    }
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
