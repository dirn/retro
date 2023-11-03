use clap::{Parser, Subcommand};

use super::link;
use super::rename;

#[derive(Debug, Parser)]
#[command(name = "retro")]
#[command(about = "synchronize retro games")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Link(link::Args),
    Rename(rename::Args),
}

pub fn dispatch() -> Result<(), String> {
    let args = Cli::parse();

    return match args.command {
        Commands::Link(args) => link::dispatch(args),
        Commands::Rename(args) => rename::dispatch(args),
    };
}
