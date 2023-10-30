use std::env::set_current_dir;
use std::path::Path;
use std::process::{exit, Command};

use super::config::load_config;
use super::utils::{capture_output, env_or_exit};

#[derive(Debug, clap::Args)]
#[command(about = "Interact with an OnionOS setup")]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    copy: CopyArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(about = "Copy backed up games for use with OnionOS")]
    Copy(CopyArgs),
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct CopyArgs {
    #[arg(required = true, help = "System to copy")]
    system: Vec<String>,

    #[arg(long, help = "Copy all systems")]
    all: bool,
}

pub fn dispatch(args: Args) -> Result<(), String> {
    let cmd = args.command.unwrap_or(Commands::Copy(args.copy));
    match cmd {
        Commands::Copy(args) => {
            return copy(args.system, args.all);
        }
    }
}

fn copy(systems: Vec<String>, all_systems: bool) -> Result<(), String> {
    let backup_location = env_or_exit("RETRO_BACKUPS");
    let destination = env_or_exit("ONION_GAMES");

    let changed = set_current_dir(Path::new(&destination));
    if changed.is_err() {
        let err = changed.err();
        eprintln!("{err:#?}");
        exit(1);
    };

    let config = match load_config(None) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };

    let systems_to_copy = if all_systems {
        config.get_system_names()
    } else {
        systems
    };

    for system in systems_to_copy {
        let system_config = match config.systems.get(&system) {
            Some(config) => config,
            None => {
                eprintln!("{system} not found in config. Skipping.");
                continue;
            }
        };

        let source = Path::new(&backup_location)
            .join(&system_config.dumper)
            .join(&system);
        if !source.is_dir() {
            eprintln!("{source:?} does not exist. Skipping.");
            continue;
        }

        let extensions = system_config.get_extensions(system.clone());

        let mut files_to_copy = Vec::new();
        for file in source.read_dir().unwrap() {
            let path = file.unwrap().path();
            if let Some(extension) = path.extension() {
                if let Some(extension) = extension.to_str() {
                    if extensions.iter().any(|e| e == extension) {
                        files_to_copy.push(path);
                    }
                }
            }
        }

        let destinations = system_config.get_destinations(system);
        for copy_destination in destinations {
            let path = Path::new(&destination).join(copy_destination);
            println!("Copying {extensions:?} from {source:?} to {path:?}.");

            let mut command = Command::new("rsync");
            command.args([
                "--archive",
                "--verbose",
                "--progress",
                "--copy-links",
                "--copy-dirlinks",
                "--size-only",
                "--delete",
                "--exclude=.DS_Store",
            ]);
            command.args(files_to_copy.clone());
            command.arg(path.to_str().unwrap());

            println!("{}", capture_output(&mut command, "Failed to copy"));
        }
    }

    Ok(())
}
