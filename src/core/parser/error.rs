use std::collections::BTreeSet;

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind};
use pest::error::{ErrorVariant, InputLocation};

use super::grammar::Rule;

#[derive(Debug, Clone, thiserror::Error)]
#[error("{error}")]
pub struct ParseErrorWithContext {
    pub error: ParseError,
    pub source_name: String,
    pub source_code: String,
}

#[derive(Debug, thiserror::Error, Clone)]
#[error("parse error")]
pub struct ParseError {
    pub span: (usize, usize),
    pub expected_tokens: Vec<String>,
    pub unexpected_tokens: Vec<String>,
    pub message: String,
    pub help_lines: Vec<String>,
}

fn is_whitespace(string: String) -> bool {
    string == "\r\n" || (string.len() == 1 && string.chars().next().unwrap().is_whitespace())
}

fn rule_to_message(rule: &Rule) -> Option<&str> {
    match rule {
        Rule::contract_decl => Some("a contract declaration (e.g. `contract MyContract {}`)"),
        Rule::fn_decl => Some("a function declaration (e.g. `fn my_function() {}`)"),
        Rule::var_decl => Some("a variable declaration (e.g. `let x = 42;`)"),
        Rule::params => todo!(),     //Some(String::from("parameters")),
        Rule::Identifier => todo!(), //Some(String::from("identifier")),
        Rule::block => todo!(),
        Rule::statement => todo!(),
        Rule::expr_stmt => todo!(),
        Rule::for_stmt => todo!(),
        Rule::if_stmt => todo!(),
        Rule::while_stmt => todo!(),
        Rule::return_stmt => todo!(),
        Rule::expr => todo!(),
        _ => None,
    }
}

impl ParseErrorWithContext {
    pub fn report(&self) -> Report<(&std::string::String, std::ops::Range<usize>)> {
        let mut colors = ColorGenerator::new();

        let a = colors.next();
        let b = colors.next();

        Report::build(
            ReportKind::Error,
            (&self.source_name, 0..self.source_code.len()),
        )
        .with_message(self.error.message.to_string())
        .with_label(
            Label::new((&self.source_name, self.error.span.0..self.error.span.1))
                .with_message(format!(
                    "here, expected one of: {}",
                    self.error
                        .expected_tokens
                        .iter()
                        .map(|t| format!("'{}'", t.fg(b)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .with_color(a),
        )
        .with_help(format!(
            "try using: {}",
            self.error
                .help_lines
                .iter()
                .map(|h| format!("\n- {h}"))
                .collect::<String>()
        ))
        .finish()
    }

    pub fn from_pest_error(
        mut err: pest::error::Error<Rule>,
        source_name: String,
        source_code: String,
    ) -> Self {
        err = err.with_path(&source_name);

        let span = match err.location {
            InputLocation::Pos(pos) => (pos.min(source_code.len()), pos.min(source_code.len())),
            InputLocation::Span((start, end)) => (start, start + end.min(1)),
        };

        type IsWhiteSpaceBoxed = Box<dyn Fn(String) -> bool>;
        let is_whitespace_boxed: IsWhiteSpaceBoxed = Box::new(is_whitespace);

        let message = match &err.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                if *positives == [Rule::Identifier] {
                    String::from("Missing identifier")
                } else {
                    String::from("Unexpected token")
                }
            }
            ErrorVariant::CustomError { message } => message.to_string(),
        };

        let mut expected_tokens = vec![];
        let mut unexpected_tokens = vec![];
        let mut context = vec![];
        if let Some(parse_attempts) = err.parse_attempts() {
            context = parse_attempts
                .call_stacks()
                .iter()
                .filter_map(|stack| stack.parent)
                .collect::<BTreeSet<_>>()
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            expected_tokens = parse_attempts
                .expected_tokens()
                .into_iter()
                .map(|t| {
                    if t.is_whitespace(&is_whitespace_boxed) {
                        String::from("WHITESPACE")
                    } else {
                        format!("{t}")
                    }
                })
                .collect::<BTreeSet<String>>()
                .into_iter()
                .collect::<Vec<_>>();
            unexpected_tokens = parse_attempts
                .unexpected_tokens()
                .into_iter()
                .map(|t| {
                    if t.is_whitespace(&is_whitespace_boxed) {
                        String::from("WHITESPACE")
                    } else {
                        format!("{t}")
                    }
                })
                .collect::<BTreeSet<String>>()
                .into_iter()
                .collect::<Vec<_>>();
        }

        let help_lines: Vec<String> = context
            .iter()
            .filter_map(|r| rule_to_message(r))
            .map(|msg| msg.to_string())
            .collect::<Vec<String>>();

        ParseErrorWithContext {
            error: ParseError {
                span,
                expected_tokens,
                unexpected_tokens,
                message,
                help_lines,
            },
            source_name,
            source_code,
        }
    }
}
