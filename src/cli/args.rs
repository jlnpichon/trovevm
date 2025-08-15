use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, short, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: ArgsCommand,
}

#[derive(Debug, Subcommand)]
pub enum ArgsCommand {
    Compile { input: String },
    Parse { input: String },
    Run { input: String },
    Eval { input: String },
}
