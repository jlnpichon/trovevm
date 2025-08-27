use crate::ast::{
    Contract, Expr, Function, Literal, Statement, Var, expr::BuiltinVariableKind,
    visitor::VisitorMut,
};

use super::CompileError;

struct DesugarBuiltins;

pub fn desugar_ast(statements: &mut [Statement]) -> Result<(), CompileError> {
    let mut desugar = DesugarBuiltins;

    for statement in statements {
        desugar.visit_statement_mut(statement)?;
    }

    Ok(())
}

impl VisitorMut for DesugarBuiltins {
    fn visit_statement_mut(&mut self, s: &mut Statement) -> Result<(), CompileError> {
        match s {
            Statement::ContractDecl(contract) => self.visit_contract_decl_mut(contract),
            Statement::FnDecl(function) => self.visit_fn_decl_mut(function),
            Statement::VarDecl(var) => self.visit_var_decl_mut(var),
            Statement::Block(statements) => {
                for statement in statements {
                    statement.accept_mut(self)?;
                }
                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    expr.accept_mut(self)?;
                }
                Ok(())
            }
            Statement::Expr(expr) => expr.accept_mut(self),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.accept_mut(self)?;

                then_branch.accept_mut(self)?;

                if let Some(else_branch) = else_branch.as_mut() {
                    else_branch.accept_mut(self)?;
                }

                Ok(())
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    initializer.accept_mut(self)?;
                }

                if let Some(condition) = condition {
                    condition.accept_mut(self)?;
                }

                if let Some(increment) = increment {
                    increment.accept_mut(self)?;
                }

                body.accept_mut(self)?;

                Ok(())
            }
            Statement::While { condition, body } => {
                condition.accept_mut(self)?;
                body.accept_mut(self)
            }
        }
    }

    fn visit_var_decl_mut(&mut self, v: &mut Var) -> Result<(), CompileError> {
        let name = v.name.as_str();

        if name == "msg" || name == "block" {
            return Err(CompileError::ReservedVariable(name.to_string()));
        }

        if let Some(initializer) = v.initializer.as_mut() {
            initializer.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_contract_decl_mut(&mut self, c: &mut Contract) -> Result<(), CompileError> {
        for var in c.vars.iter_mut() {
            self.visit_var_decl_mut(var)?;
        }

        for func in c.funcs.iter_mut() {
            self.visit_fn_decl_mut(func)?;
        }

        Ok(())
    }

    fn visit_fn_decl_mut(&mut self, f: &mut Function) -> Result<(), CompileError> {
        for _ in &f.params {}
        f.body.accept_mut(self)
    }

    fn visit_expr_mut(&mut self, e: &mut Expr) -> Result<(), CompileError> {
        match e {
            Expr::Get { object, name } => {
                if let Expr::Variable(obj) = object.as_mut() {
                    let new_expr = match obj.as_str() {
                        "msg" => match name.as_str() {
                            "sender" => Some(Expr::BuiltinVariable(BuiltinVariableKind::MsgSender)),
                            "value" => Some(Expr::BuiltinVariable(BuiltinVariableKind::MsgValue)),
                            "data" => Some(Expr::BuiltinVariable(BuiltinVariableKind::MsgData)),
                            "balance" => {
                                Some(Expr::BuiltinVariable(BuiltinVariableKind::MsgBalance))
                            }
                            _ => {
                                return Err(CompileError::InvalidFieldAccess(
                                    "msg".to_string(),
                                    name.to_string(),
                                ));
                            }
                        },
                        "block" => match name.as_str() {
                            "number" => {
                                Some(Expr::BuiltinVariable(BuiltinVariableKind::BlockNumber))
                            }
                            "timestamp" => {
                                Some(Expr::BuiltinVariable(BuiltinVariableKind::BlockTimestamp))
                            }
                            "hash" => Some(Expr::BuiltinVariable(BuiltinVariableKind::BlockHash)),
                            "gas_limit" => {
                                Some(Expr::BuiltinVariable(BuiltinVariableKind::BlockGasLimit))
                            }
                            "coinbase" => {
                                Some(Expr::BuiltinVariable(BuiltinVariableKind::BlockCoinbase))
                            }
                            _ => {
                                return Err(CompileError::InvalidFieldAccess(
                                    "block".to_string(),
                                    name.to_string(),
                                ));
                            }
                        },
                        _ => match name.as_str() {
                            "balance" => Some(Expr::BuiltinVariable(BuiltinVariableKind::Balance(
                                object.clone(),
                            ))),
                            _ => None,
                        },
                    };

                    if let Some(expr) = new_expr {
                        *e = expr;
                    }
                } else if let Expr::Literal(Literal::Number(_)) = object.as_mut()
                    && name.as_str() == "balance"
                {
                    *e = Expr::BuiltinVariable(BuiltinVariableKind::Balance(object.clone()));
                }
            }
            Expr::Set { object, value, .. } => {
                object.accept_mut(self)?;
                value.accept_mut(self)?;
            }
            Expr::IndexGet { object, index } => {
                object.accept_mut(self)?;
                index.accept_mut(self)?;
            }
            Expr::IndexSet {
                object,
                index,
                value,
            } => {
                object.accept_mut(self)?;
                index.accept_mut(self)?;
                value.accept_mut(self)?;
            }
            Expr::Assign { target, value } => {
                target.accept_mut(self)?;
                value.accept_mut(self)?;
            }
            Expr::FnCall { callee, args } => {
                callee.accept_mut(self)?;
                for arg in args {
                    arg.accept_mut(self)?;
                }
            }
            Expr::Unary { expr, .. } => expr.accept_mut(self)?,
            Expr::Binary { lhs, rhs, .. } => {
                lhs.accept_mut(self)?;
                rhs.accept_mut(self)?;
            }
            _ => (),
        };

        Ok(())
    }

    fn visit_literal_mut(&mut self, _l: &mut Literal) -> Result<(), CompileError> {
        Ok(())
    }
}
