use pest::Parser as PestParser;
use pest_derive::Parser;

use super::{error::ParseErrorWithContext, expr::expr};
use crate::core::ast::{Contract, Function, Identifier, Program, Statement, Var};

pub type PestError = Box<pest::error::Error<Rule>>;

#[derive(Parser)]
#[grammar = "core/parser/grammar.pest"]
pub struct Parser;

pub fn expect_rule<'i>(
    pairs: &mut pest::iterators::Pairs<'i, Rule>,
    expected: Rule,
) -> Result<pest::iterators::Pair<'i, Rule>, PestError> {
    let pair = pairs.next().ok_or_else(|| {
        Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::ParsingError {
                positives: vec![expected],
                negatives: vec![],
            },
            pest::Position::from_start(""),
        ))
    })?;

    if pair.as_rule() == expected {
        Ok(pair)
    } else {
        Err(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::ParsingError {
                positives: vec![expected],
                negatives: vec![pair.as_rule()],
            },
            pair.as_span(),
        )))
    }
}

pub fn accept_rule<'i>(
    pairs: &mut pest::iterators::Pairs<'i, Rule>,
    expected: Rule,
) -> Option<pest::iterators::Pair<'i, Rule>> {
    if let Some(peek) = pairs.peek() {
        if peek.as_rule() == expected {
            return pairs.next();
        }
    }
    None
}

pub fn peek_rule(pairs: &mut pest::iterators::Pairs<'_, Rule>, expected: Rule) -> bool {
    pairs
        .peek()
        .map(|p| p.as_rule() == expected)
        .unwrap_or(false)
}

fn var_decl(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Var, PestError> {
    expect_rule(&mut pairs, Rule::LET_KW)?;

    let mut initializer = None;
    let name = pairs.next().unwrap().as_str().to_string();

    if accept_rule(&mut pairs, Rule::EQ).is_some() {
        initializer = Some(expr(&mut pairs.next().unwrap().into_inner())?);
    }

    Ok(Var {
        name: name.into(),
        initializer,
    })
}

fn params(pairs: &mut pest::iterators::Pairs<Rule>) -> Result<Vec<String>, PestError> {
    let mut params = vec![];

    while let Some(peek) = pairs.peek() {
        match peek.as_rule() {
            Rule::Identifier => {
                let param = pairs.next().unwrap().as_str().to_string();
                params.push(param);
            }
            Rule::COMMA => {
                pairs.next().unwrap();
                continue;
            }
            _ => break,
        }
    }

    Ok(params)
}

fn expr_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    Ok(Statement::Expr(expr(
        &mut pairs.next().unwrap().into_inner(),
    )?))
}

fn return_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    expect_rule(&mut pairs, Rule::RETURN_KW)?;
    Ok(Statement::Return(expr(
        &mut pairs.next().unwrap().into_inner(),
    )?))
}

fn if_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    expect_rule(&mut pairs, Rule::IF_KW)?;

    expect_rule(&mut pairs, Rule::LPAREN)?;
    let condition = expr(&mut pairs.next().unwrap().into_inner())?;
    expect_rule(&mut pairs, Rule::RPAREN)?;
    let then_branch = block(pairs.next().unwrap().into_inner())?.into();

    let mut else_branch = None;
    if accept_rule(&mut pairs, Rule::ELSE_KW).is_some() {
        else_branch = Some(block(pairs.next().unwrap().into_inner())?.into());
    }

    Ok(Statement::If {
        condition,
        then_branch,
        else_branch,
    })
}

fn for_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    expect_rule(&mut pairs, Rule::FOR_KW)?;

    expect_rule(&mut pairs, Rule::LPAREN)?;

    let initializer = match pairs.peek().unwrap().as_rule() {
        Rule::var_decl => {
            Some(Statement::VarDecl(var_decl(pairs.next().unwrap().into_inner())?).into())
        }
        Rule::expr_stmt => Some(expr_statement(pairs.next().unwrap().into_inner())?.into()),
        Rule::SEMICOLON => {
            pairs.next().unwrap();
            None
        }
        r => {
            println!("{r:#?}");
            unreachable!()
        }
    };

    let condition = if accept_rule(&mut pairs, Rule::SEMICOLON).is_some() {
        None
    } else {
        let xpr = expr(&mut pairs.next().unwrap().into_inner())?;
        expect_rule(&mut pairs, Rule::SEMICOLON)?;
        Some(xpr)
    };

    let increment = if accept_rule(&mut pairs, Rule::RPAREN).is_some() {
        None
    } else {
        let xpr = expr(&mut pairs.next().unwrap().into_inner())?;
        expect_rule(&mut pairs, Rule::RPAREN)?;
        Some(xpr)
    };

    let body = block(pairs.next().unwrap().into_inner())?.into();

    Ok(Statement::For {
        initializer,
        condition,
        increment,
        body,
    })
}

fn while_statement(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    expect_rule(&mut pairs, Rule::WHILE_KW)?;

    expect_rule(&mut pairs, Rule::LPAREN)?;
    let condition = expr(&mut pairs.next().unwrap().into_inner())?;
    expect_rule(&mut pairs, Rule::RPAREN)?;
    let body = block(pairs.next().unwrap().into_inner())?.into();

    Ok(Statement::While { condition, body })
}

fn statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement, PestError> {
    Ok(match pair.as_rule() {
        Rule::var_decl => Statement::VarDecl(var_decl(pair.into_inner())?),
        Rule::expr_stmt => expr_statement(pair.into_inner())?,
        Rule::for_stmt => for_statement(pair.into_inner())?,
        Rule::if_stmt => if_statement(pair.into_inner())?,
        Rule::return_stmt => return_statement(pair.into_inner())?,
        Rule::while_stmt => while_statement(pair.into_inner())?,
        Rule::block => block(pair.into_inner())?,
        rule => {
            return Err(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("unexpected {:?}, expected statement", rule),
                },
                pair.as_span(),
            )));
        }
    })
}

fn block(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Statement, PestError> {
    expect_rule(&mut pairs, Rule::LBRACE)?;

    let mut statements = vec![];
    while !peek_rule(&mut pairs, Rule::RBRACE) {
        let pair = pairs.next().unwrap();
        statements.push(decl(pair)?);
    }

    expect_rule(&mut pairs, Rule::RBRACE)?;

    Ok(Statement::Block(statements))
}

fn fn_decl(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Function, PestError> {
    expect_rule(&mut pairs, Rule::FN_KW)?;
    let name = pairs.next().unwrap().as_str().to_string();

    expect_rule(&mut pairs, Rule::LPAREN)?;
    let params = params(&mut pairs)?;
    expect_rule(&mut pairs, Rule::RPAREN)?;

    let body = block(pairs.next().unwrap().into_inner())?;

    Ok(Function {
        name: name.into(),
        params,
        body: body.into(),
    })
}

fn contract_decl(mut pairs: pest::iterators::Pairs<Rule>) -> Result<Contract, PestError> {
    expect_rule(&mut pairs, Rule::CONTRACT_KW)?;

    let mut funcs = vec![];
    let mut vars = vec![];
    let name = expect_rule(&mut pairs, Rule::Identifier)?
        .as_str()
        .to_string();

    expect_rule(&mut pairs, Rule::LBRACE)?;
    for pair in pairs.by_ref() {
        match pair.as_rule() {
            Rule::fn_decl => {
                funcs.push(fn_decl(pair.into_inner())?);
            }
            Rule::var_decl => {
                vars.push(var_decl(pair.into_inner())?);
            }
            Rule::RBRACE => break,
            r => {
                return Err(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::ParsingError {
                        positives: vec![Rule::var_decl],
                        negatives: vec![r],
                    },
                    pair.as_span(),
                )
                .into());
            }
        };
    }

    Ok(Contract {
        name: Identifier(name),
        vars,
        funcs,
    })
}

pub fn decl(pair: pest::iterators::Pair<Rule>) -> Result<Statement, PestError> {
    Ok(match pair.as_rule() {
        Rule::contract_decl => Statement::ContractDecl(contract_decl(pair.into_inner())?),
        Rule::fn_decl => Statement::FnDecl(fn_decl(pair.into_inner())?),
        Rule::var_decl => Statement::VarDecl(var_decl(pair.into_inner())?),
        _ => statement(pair)?,
    })
}

pub fn parse_program(
    source_name: String,
    source_code: &str,
) -> Result<Program, Box<ParseErrorWithContext>> {
    pest::set_error_detail(true);
    let parsed = Parser::parse(Rule::program, source_code).map_err(|err| {
        ParseErrorWithContext::from_pest_error(
            err,
            source_name.to_string(),
            source_code.to_string(),
        )
    })?;

    let mut statements = vec![];

    for pair in parsed {
        if matches!(pair.as_rule(), Rule::EOI) {
            break;
        }

        let statement = decl(pair).map_err(|err| {
            ParseErrorWithContext::from_pest_error(
                *err,
                source_name.to_string(),
                source_code.to_string(),
            )
        })?;
        statements.push(statement);
    }

    Ok(Program { statements })
}
