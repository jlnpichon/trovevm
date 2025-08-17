use anyhow::Result;

use super::{
    InputSource,
    args::{Args, ArgsCommand},
};

#[derive(Debug)]
pub struct Config {
    pub verbose: u8,
    pub command: ConfigCommand,
}

#[derive(Debug)]
pub enum ConfigCommand {
    Compile { input: InputSource },
    Parse { input: InputSource },
    Run { input: InputSource },
    Eval { input: InputSource },
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self> {
        let command = ConfigCommand::try_from(args.command)?;

        Ok(Self {
            verbose: args.verbose,
            command,
        })
    }
}

impl TryFrom<ArgsCommand> for ConfigCommand {
    type Error = anyhow::Error;

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
            ArgsCommand::Eval { input } => ConfigCommand::Eval {
                input: InputSource::from(input),
            },
        })
    }
}
