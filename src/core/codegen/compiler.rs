use crate::core::{
    ast::{BinaryOp, Contract, Expr, Function, Literal, Statement, UnaryOp, Var, Visitor},
    vm::{Opcode, Program, Value},
};

#[derive(Debug, thiserror::Error)]
pub enum CompileError {}

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    program: Program,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Compiler {
    pub fn compile(mut self, statements: &[Statement]) -> Result<Program, CompileError> {
        for statement in statements {
            self.visit_statement(statement);
        }
        Ok(self.program)
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
                for statement in statements {
                    statement.accept(self);
                }
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
                let index = self.program.define_constant(Value::String(var.clone()));
                self.program.emit_opcode(Opcode::GetGlobal(index));
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
        if let Some(initializer) = &v.initializer {
            initializer.accept(self);
        } else {
            self.program.emit_null();
        }
        self.program.define_global(&v.name);
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
