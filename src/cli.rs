use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

use env_logger;

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

    #[command(flatten)]
    verbose: Verbosity,
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

    let log_level = args.verbose.log_level_filter();

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_level(false)
        .format_target(false)
        .format_timestamp(None)
        .init();

    return match args.command {
        Commands::Compress(args) => compress::dispatch(args, log_level),
        Commands::Link(args) => link::dispatch(args),
        Commands::Playlist(args) => playlist::dispatch(args),
        Commands::Rename(args) => rename::dispatch(args),
    };
}
