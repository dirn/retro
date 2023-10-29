use clap::{Parser, Subcommand};

use super::link;
use super::onion;

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
    Onion(onion::Args),
}

pub fn dispatch() -> Result<(), String> {
    let args = Cli::parse();

    return match args.command {
        Commands::Link(args) => link::dispatch(args),
        Commands::Onion(args) => onion::dispatch(args),
    };
}
