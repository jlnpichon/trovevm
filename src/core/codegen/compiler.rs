use crate::core::ast::{Contract, Expr, Function, Literal, Statement, Var, Visitor};

#[derive(Debug, Clone)]
pub struct Compiler {}

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
            Expr::Variable(_) => todo!(),
            Expr::Assign { target, value } => {
                target.accept(self);
                value.accept(self);
            }
            Expr::FnCall { callee, args } => {
                callee.accept(self);
                for arg in args {
                    arg.accept(self);
                }
            }
            Expr::Unary { op, expr } => {
                expr.accept(self);
            }
            Expr::Binary { op, lhs, rhs } => {
                lhs.accept(self);
                rhs.accept(self);
            }
        }
    }

    fn visit_literal(&mut self, l: &Literal) {
        match l {
            Literal::Number(_) => todo!(),
            Literal::String(_) => todo!(),
            Literal::Bool(_) => todo!(),
            Literal::Null => todo!(),
        }
    }

    fn visit_var_decl(&mut self, v: &Var) {
        if let Some(initializer) = &v.initializer {
            initializer.accept(self);
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
