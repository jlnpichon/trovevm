mod cli;
mod core;

use anyhow::Result;
use ariadne::Source;
use cli::{
    Args, Config, ConfigCommand,
    commands::{self, CompileCommandError, ParseCommandError},
    init_logging,
};

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
        ConfigCommand::Run { input } => {
            let value = commands::run(input)?;
            println!("{value:?}");
        }
    };

    Ok(())
}
