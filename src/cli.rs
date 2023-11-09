use clap::{Parser, Subcommand};

use super::compress;
use super::link;
use super::playlist;
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
    #[clap(visible_alias = "chd")]
    Compress(compress::Args),
    Link(link::Args),
    #[clap(visible_alias = "m3u")]
    Playlist(playlist::Args),
    Rename(rename::Args),
}

pub fn dispatch() -> Result<(), String> {
    let args = Cli::parse();

    return match args.command {
        Commands::Compress(args) => compress::dispatch(args),
        Commands::Link(args) => link::dispatch(args),
        Commands::Playlist(args) => playlist::dispatch(args),
        Commands::Rename(args) => rename::dispatch(args),
    };
}
