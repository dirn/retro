use log::{debug, error};

use super::config::load_global_config;
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
    #[command(about = "Clean up broken links")]
    Clean {
        #[arg(help = "System to clean up")]
        system: Vec<String>,

        #[arg(long, help = "Clean up all systems")]
        all: bool,

        #[arg(long, help = "Don't remove the broken links")]
        dry_run: bool,
    },

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

impl Args {
    pub fn dispatch(self) -> Result<(), String> {
        let cmd = self.command.unwrap_or(Commands::Link(self.link));
        match cmd {
            Commands::Clean {
                system,
                all,
                dry_run,
            } => clean_links(system, all, dry_run),

            Commands::Link(args) => link(args.system, args.all),
        }
    }
}

fn clean_links(systems: Vec<String>, all_systems: bool, dry_run: bool) -> Result<(), String> {
    let config = load_global_config()?.link;

    for destination in config.expand_destinations() {
        if let Err(e) = games::clean(&destination, &systems, all_systems, dry_run) {
            error!("{e:#?}");
        }
    }

    Ok(())
}

fn link(systems: Vec<String>, all_systems: bool) -> Result<(), String> {
    let config = load_global_config()?.link;

    for destination in config.expand_destinations() {
        debug!("Linking games to {destination:?}");
        if let Err(e) = games::link(&config.expand_source(), &destination, &systems, all_systems) {
            error!("{e:#?}");
        }
    }

    Ok(())
}
