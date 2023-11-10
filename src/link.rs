use std::path::PathBuf;

use log::{error, info};

use super::games;
use super::onion;
use super::utils::env_or_exit;

#[derive(Debug, clap::Args)]
#[command(about = "Link backups")]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    link: LinkArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(about = "Create links for backed up games")]
    Link(LinkArgs),
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct LinkArgs {
    #[arg(help = "System to synchronize")]
    system: Vec<String>,

    #[arg(long, help = "Synchronize all systems")]
    all: bool,
}

pub fn dispatch(args: Args) -> Result<(), String> {
    let cmd = args.command.unwrap_or(Commands::Link(args.link));
    match cmd {
        Commands::Link(args) => {
            return link(args.system, args.all);
        }
    };
}

fn link(systems: Vec<String>, all_systems: bool) -> Result<(), String> {
    let backup_location = PathBuf::from(env_or_exit("RETRO_BACKUPS"));

    match games::link(&backup_location, &systems, all_systems) {
        Ok(_) => info!(""),
        Err(e) => {
            error!("{e:#?}");
        }
    }
    match onion::copy(&backup_location, &systems, all_systems) {
        Ok(_) => info!(""),
        Err(e) => {
            error!("{e:#?}");
        }
    }

    // This isn't really an error but I currently want it to appear unless output is suppressed.
    error!("Done.");

    Ok(())
}
