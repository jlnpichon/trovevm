use super::expr::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    ContractDecl(Contract),
    FnDecl(Function),
    VarDecl(Var),
    Block(Vec<Statement>),
    Params,
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: Identifier,
    pub vars: Vec<Var>,
    pub funcs: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub name: Identifier,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<String>,
    pub body: Box<Statement>,
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}
