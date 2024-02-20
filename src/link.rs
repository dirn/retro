use log::{error, info};

use super::config::load_config;
use super::games;

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
    let config = match load_config(None) {
        Ok(config) => config.link,
        Err(e) => {
            return Err(e);
        }
    };

    for destination in config.expand_destinations() {
        match games::link(&config.expand_source(), &destination, &systems, all_systems) {
            Ok(_) => info!(""),
            Err(e) => {
                error!("{e:#?}");
            }
        }
    }

    // This isn't really an error but I currently want it to appear unless output is suppressed.
    error!("Done.");

    Ok(())
}
