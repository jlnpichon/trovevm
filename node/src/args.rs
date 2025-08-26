use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, default_value = "127.0.0.1:4321")]
    pub rpc_addr: String,

    #[arg(short, long)]
    pub state_file: Option<PathBuf>,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        CliArgs::parse()
    }
}
