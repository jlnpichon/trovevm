use pest::pratt_parser::PrattParser;

use crate::{
    ast::{BinaryOp, Expr, Literal, UnaryOp},
    parser::grammar::accept_rule,
};

use super::grammar::{PestError, Rule, expect_rule};

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

impl From<Rule> for BinaryOp {
    fn from(value: Rule) -> Self {
        match value {
            Rule::EQEQ => BinaryOp::Eq,
            Rule::NEQ => BinaryOp::Neq,
            Rule::GT => BinaryOp::Gt,
            Rule::GE => BinaryOp::Ge,
            Rule::LT => BinaryOp::Lt,
            Rule::LE => BinaryOp::Le,
            Rule::AND => BinaryOp::And,
            Rule::OR => BinaryOp::Or,
            Rule::PLUS => BinaryOp::Add,
            Rule::MINUS => BinaryOp::Substract,
            Rule::SLASH => BinaryOp::Divide,
            Rule::STAR => BinaryOp::Multiply,
            Rule::MODULO => BinaryOp::Modulo,
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

            let mut expression = match primary_expr.as_rule() {
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
                            negatives: vec![rule],
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
                        if accept_rule(&mut inner, Rule::DOT).is_some() {
                            let name = expect_rule(&mut inner, Rule::Identifier)?;
                            expression = Expr::Get {
                                object: expression.into(),
                                name: name.as_str().to_string(),
                            };
                        } else {
                            expect_rule(&mut inner, Rule::LPAREN)?;
                            let args = args(&mut inner)?;
                            expression = Expr::FnCall {
                                callee: Box::new(expression),
                                args,
                            };
                            expect_rule(&mut inner, Rule::RPAREN)?;
                        }
                    }
                    _ => panic!("unexpected suffix: {:?}", suffix.as_rule()),
                }
            }

            Ok(expression)
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
                match lhs? {
                    Expr::Get { object, name } => Ok(Expr::Set {
                        object,
                        name,
                        value: rhs?.into(),
                    }),
                    lhs => Ok(Expr::Assign {
                        target: lhs.into(),
                        value: rhs?.into(),
                    }),
                }
            } else {
                Ok(Expr::Binary {
                    lhs: lhs?.into(),
                    rhs: rhs?.into(),
                    op: BinaryOp::from(rule),
                })
            }
        })
        .parse(pairs)
}
