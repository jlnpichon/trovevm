use miette::Result;

use super::{
    InputSource,
    args::{Args, ArgsCommand},
};

#[derive(Debug)]
pub struct Config {
    pub verbose: u8,
    pub pretty_report: bool,
    pub command: ConfigCommand,
}

#[derive(Debug)]
pub enum ConfigCommand {
    Compile { input: InputSource },
    Parse { input: InputSource },
    Run { input: InputSource },
}

impl TryFrom<Args> for Config {
    type Error = miette::Error;

    fn try_from(args: Args) -> Result<Self> {
        let command = ConfigCommand::try_from(args.command)?;

        Ok(Self {
            verbose: args.verbose,
            pretty_report: args.pretty_report,
            command,
        })
    }
}

impl TryFrom<ArgsCommand> for ConfigCommand {
    type Error = miette::Error;

    fn try_from(cmd: ArgsCommand) -> Result<Self> {
        Ok(match cmd {
            ArgsCommand::Compile { input } => ConfigCommand::Compile {
                input: InputSource::from(input),
            },
            ArgsCommand::Parse { input } => ConfigCommand::Parse {
                input: InputSource::from(input),
            },
            ArgsCommand::Run { input } => ConfigCommand::Run {
                input: InputSource::from(input),
            },
        })
    }
}
