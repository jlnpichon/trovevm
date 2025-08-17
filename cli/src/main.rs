mod args;
pub mod commands;
mod config;
mod input;
mod tracing;

pub use args::Args;
use commands::{CompileCommandError, EvalError, ParseCommandError, RunError};
pub use config::{Config, ConfigCommand};
pub use input::InputSource;
pub use tracing::init_logging;

use anyhow::Result;
use ariadne::Source;

use clap::Parser;

fn main() -> Result<()> {
    let cli = Args::parse();
    let conf = Config::try_from(cli)?;

    init_logging(conf.verbose)?;

    match conf.command {
        ConfigCommand::Compile { input } => match commands::compile(input) {
            Ok(bytecode) => println!("{bytecode:#?}"),
            Err(err) => match err {
                CompileCommandError::Input(e) => eprintln!("{e}"),
                CompileCommandError::Parse(e) => e
                    .report()
                    .print((&e.source_name, Source::from(&e.source_code)))
                    .unwrap(),
                CompileCommandError::Compile(e) => eprintln!("{e}"),
            },
        },
        ConfigCommand::Parse { input } => match commands::parse(input) {
            Ok(program) => println!("{program:#?}"),
            Err(err) => {
                match err {
                    ParseCommandError::Input(e) => eprintln!("{e}"),
                    ParseCommandError::Parse(e) => e
                        .report()
                        .print((&e.source_name, Source::from(&e.source_code)))
                        .unwrap(),
                };
                std::process::exit(1);
            }
        },
        ConfigCommand::Run { input } => match commands::run(input) {
            Ok(result) => println!("{result}"),
            Err(err) => {
                match err {
                    RunError::Input(e) => eprintln!("{e}"),
                    RunError::Parse(e) => e
                        .report()
                        .eprint((&e.source_name, Source::from(&e.source_code)))
                        .unwrap(),
                    RunError::Compile(e) => eprintln!("{e}"),
                    RunError::Runtime(e) => eprintln!("{e}"),
                };
                std::process::exit(1);
            }
        },
        ConfigCommand::Eval { input } => match commands::eval(input) {
            Ok(result) => println!("{result}"),
            Err(err) => {
                match err {
                    EvalError::Input(e) => eprintln!("{e}"),
                    EvalError::Parse(e) => e
                        .report()
                        .eprint((&e.source_name, Source::from(&e.source_code)))
                        .unwrap(),
                    EvalError::Compile(e) => eprintln!("{e}"),
                    EvalError::Runtime(e) => eprintln!("{e}"),
                };
                std::process::exit(1);
            }
        },
    };

    Ok(())
}
