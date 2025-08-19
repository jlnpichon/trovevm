pub mod expr;
pub mod visitor;

use std::ops::Deref;

pub use expr::{BinaryOp, Expr, UnaryOp};
pub use visitor::Visitor;

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
    Return(Option<Expr>),
    Expr(Expr),
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    For {
        initializer: Option<Box<Statement>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Statement>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    This,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: Identifier,
    pub vars: Vec<Var>,
    pub funcs: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: Identifier,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Identifier,
    pub params: Vec<String>,
    pub body: Box<Statement>,
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Statement {
    pub fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_statement(self)
    }
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

impl<'a> From<&'a Identifier> for &'a str {
    fn from(value: &'a Identifier) -> Self {
        &value.0
    }
}
