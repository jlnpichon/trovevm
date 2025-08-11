use super::Literal;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    Assign {
        target: Box<Expr>,
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
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
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

impl Op {
    pub fn is_cmp(&self) -> bool {
        matches!(self, Op::Gt | Op::Ge | Op::Lt | Op::Le | Op::Eq | Op::Neq)
    }
}
