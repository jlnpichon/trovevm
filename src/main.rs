mod cli;
mod core;

use ariadne::Source;
use cli::{
    Args, Config, ConfigCommand,
    commands::{self, ParseCommandError},
    init_logging,
};

use clap::Parser;
use miette::Result;

fn main() -> Result<()> {
    let cli = Args::parse();
    let conf = Config::try_from(cli)?;

    init_logging(conf.verbose)?;

    match conf.command {
        ConfigCommand::Compile { input } => commands::compile(input)?,
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
        ConfigCommand::Run { input } => commands::run(input)?,
    };

    Ok(())
}
