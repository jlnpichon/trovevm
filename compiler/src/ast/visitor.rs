use super::{Contract, Expr, Function, Literal, Statement, Var};

pub trait Visitor {
    fn visit_statement(&mut self, s: &Statement);
    fn visit_var_decl(&mut self, v: &Var);
    fn visit_contract_decl(&mut self, c: &Contract);
    fn visit_fn_decl(&mut self, f: &Function);
    fn visit_expr(&mut self, e: &Expr);
    fn visit_literal(&mut self, l: &Literal);
}
