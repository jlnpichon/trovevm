#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Neg,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,

    And,
    Or,

    Pop,
    Push(usize),

    Jump(usize),
}
