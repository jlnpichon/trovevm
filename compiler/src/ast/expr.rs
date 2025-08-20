use crate::CompileError;

use super::{Literal, Visitor, visitor::VisitorMut};

#[derive(Debug, Clone)]
pub enum BuiltinVariableKind {
    MsgSender,
    MsgValue,
    MsgData,
    MsgBalance,
    BlockNumber,
    BlockTimestamp,
    BlockHash,
    BlockGasLimit,
    BlockCoinbase,
    Balance(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    BuiltinVariable(BuiltinVariableKind),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },
    FnCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Substract,
    Multiply,
    Divide,
    Modulo,

    Or,
    And,

    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Neq,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Minus,
}

impl BinaryOp {
    pub fn is_cmp(&self) -> bool {
        matches!(
            self,
            BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Eq
                | BinaryOp::Neq
        )
    }
}

impl Expr {
    pub fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_expr(self)
    }

    pub fn accept_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> Result<(), CompileError> {
        visitor.visit_expr_mut(self)
    }
}
