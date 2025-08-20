use crate::CompileError;

use super::{Contract, Expr, Function, Literal, Statement, Var};

pub trait Visitor {
    fn visit_statement(&mut self, s: &Statement);
    fn visit_var_decl(&mut self, v: &Var);
    fn visit_contract_decl(&mut self, c: &Contract);
    fn visit_fn_decl(&mut self, f: &Function);
    fn visit_expr(&mut self, e: &Expr);
    fn visit_literal(&mut self, l: &Literal);
}

pub trait VisitorMut {
    fn visit_statement_mut(&mut self, s: &mut Statement) -> Result<(), CompileError>;
    fn visit_var_decl_mut(&mut self, v: &mut Var) -> Result<(), CompileError>;
    fn visit_contract_decl_mut(&mut self, c: &mut Contract) -> Result<(), CompileError>;
    fn visit_fn_decl_mut(&mut self, f: &mut Function) -> Result<(), CompileError>;
    fn visit_expr_mut(&mut self, e: &mut Expr) -> Result<(), CompileError>;
    fn visit_literal_mut(&mut self, l: &mut Literal) -> Result<(), CompileError>;
}
