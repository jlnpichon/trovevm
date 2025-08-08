use pest::pratt_parser::PrattParser;

use super::ast::Literal;
use super::grammar::{PestError, Rule, expect_rule};

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

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc, PrattParser, Op};

        PrattParser::new()
            .op(Op::prefix(Rule::NOT))
            .op(Op::prefix(Rule::NEG))
            .op(Op::infix(Rule::EQ, Assoc::Right))
            .op(Op::infix(Rule::OR, Assoc::Left))
            .op(Op::infix(Rule::AND, Assoc::Left))
            .op(Op::infix(Rule::EQEQ, Assoc::Left) | Op::infix(Rule::NEQ, Assoc::Left))
            .op(Op::infix(Rule::GT, Assoc::Left) | Op::infix(Rule::GE, Assoc::Left) |
                Op::infix(Rule::LT, Assoc::Left) | Op::infix(Rule::LE, Assoc::Left))
            .op(Op::infix(Rule::PLUS, Assoc::Left) | Op::infix(Rule::MINUS, Assoc::Left))
            .op(Op::infix(Rule::STAR, Assoc::Left) |
                Op::infix(Rule::SLASH, Assoc::Left) |
                Op::infix(Rule::MODULO, Assoc::Left))
    };
}

impl From<Rule> for UnaryOp {
    fn from(value: Rule) -> Self {
        match value {
            Rule::NOT => UnaryOp::Not,
            Rule::NEG => UnaryOp::Minus,
            _ => unreachable!(),
        }
    }
}

impl From<Rule> for Op {
    fn from(value: Rule) -> Self {
        match value {
            Rule::EQEQ => Op::Eq,
            Rule::NEQ => Op::Neq,
            Rule::GT => Op::Gt,
            Rule::GE => Op::Ge,
            Rule::LT => Op::Lt,
            Rule::LE => Op::Le,
            Rule::AND => Op::And,
            Rule::OR => Op::Or,
            Rule::PLUS => Op::Add,
            Rule::MINUS => Op::Substract,
            Rule::SLASH => Op::Divide,
            Rule::STAR => Op::Multiply,
            Rule::MODULO => Op::Modulo,
            _ => unreachable!(),
        }
    }
}

fn args(pairs: &mut pest::iterators::Pairs<Rule>) -> Result<Vec<Expr>, PestError> {
    let mut args = vec![];
    while let Some(peek) = pairs.peek() {
        match peek.as_rule() {
            Rule::expr => {
                let mut pairs = pairs.next().unwrap().into_inner();
                args.push(expr(&mut pairs)?)
            }
            Rule::COMMA => {
                pairs.next().unwrap();
                continue;
            }
            _ => break,
        }
    }

    Ok(args)
}

fn literal(pairs: &mut pest::iterators::Pairs<Rule>) -> Result<Expr, PestError> {
    let pair = pairs.next().unwrap();
    Ok(match pair.as_rule() {
        Rule::BOOL => {
            let value = match pair.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            };
            Expr::Literal(Literal::Bool(value))
        }
        Rule::String => Expr::Literal(Literal::String(
            pair.as_str()
                .trim_matches('\'')
                .trim_matches('"')
                .to_owned(),
        )),
        Rule::NULL_KW => Expr::Literal(Literal::Null),
        Rule::INT => Expr::Literal(Literal::Number(pair.as_str().parse::<f64>().map_err(
            |e| {
                pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: format!("{e}"),
                    },
                    pair.as_span(),
                )
            },
        )?)),
        rule => {
            return Err(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("unexpected {:?}, expected Literal", rule),
                },
                pair.as_span(),
            )));
        }
    })
}

pub fn expr(pairs: &mut pest::iterators::Pairs<Rule>) -> Result<Expr, PestError> {
    PRATT_PARSER
        .map_primary(|unary_expr| {
            assert!(matches!(unary_expr.as_rule(), Rule::unary_expr));
            let mut inner = unary_expr.into_inner();
            let primary_expr = inner.next().unwrap();

            let mut expr = match primary_expr.as_rule() {
                Rule::Literal => literal(&mut primary_expr.into_inner())?,
                Rule::grouped_expr => {
                    // Here we have grouped_expr { inner: [ expr { inner: [ unary_expr ] } ] }
                    let mut inner = primary_expr.into_inner(); // peel grouped_expr
                    let xpr = inner.next().unwrap(); // get expr
                    expr(&mut xpr.into_inner())?
                }
                Rule::Identifier => Expr::Variable(primary_expr.as_str().into()),
                rule => {
                    return Err(pest::error::Error::new_from_span(
                        pest::error::ErrorVariant::ParsingError {
                            positives: vec![],
                            negatives: vec![primary_expr.as_rule()],
                        },
                        primary_expr.as_span(),
                    )
                    .into());
                }
            };

            for suffix in inner {
                match suffix.as_rule() {
                    Rule::call_suffix => {
                        let mut inner = suffix.into_inner();
                        expect_rule(&mut inner, Rule::LPAREN)?;
                        let args = args(&mut inner)?;
                        expr = Expr::FnCall {
                            callee: Box::new(expr),
                            args,
                        };
                        expect_rule(&mut inner, Rule::RPAREN)?;
                    }
                    _ => panic!("unexpected suffix: {:?}", suffix.as_rule()),
                }
            }

            Ok(expr)
        })
        .map_prefix(|op, rhs| {
            Ok(Expr::Unary {
                op: UnaryOp::from(op.as_rule()),
                expr: rhs?.into(),
            })
        })
        .map_infix(|lhs, op, rhs| {
            let rule = op.as_rule();
            if rule == Rule::EQ {
                Ok(Expr::Assign {
                    target: lhs?.into(),
                    value: rhs?.into(),
                })
            } else {
                Ok(Expr::Binary {
                    lhs: lhs?.into(),
                    rhs: rhs?.into(),
                    op: Op::from(rule),
                })
            }
        })
        .parse(pairs)
}
