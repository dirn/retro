use std::env::set_current_dir;
use std::path::Path;
use std::process::{exit, Command};

use super::config::load_config;
use super::utils::{capture_output, env_or_exit};

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
    let backup_location = env_or_exit("RETRO_BACKUPS");
    let destination = env_or_exit("RETRO_GAMES");

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

    let systems_to_link = if all_systems {
        config.get_system_names()
    } else {
        systems
    };

    for system in systems_to_link {
        let path = Path::new(&destination).join(&system);

        let system_config = match config.systems.get(&system) {
            Some(config) => config,
            None => {
                eprintln!("{system} not found in config. Skipping.");
                continue;
            }
        };

        let extensions = system_config.get_extensions(system.clone());

        let _ = set_current_dir(&path).is_ok();
        let mut source = Path::new(&backup_location)
            .join(&system_config.dumper)
            .join(&system);
        if let Some(extra_path) = &system_config.extra_path {
            source = source.join(extra_path);
        }
        println!("Linking {extensions:?} from {source:?} to {path:?}.");

        if !source.is_dir() {
            eprintln!("{source:?} does not exist. Skipping.");
            continue;
        }

        for file in source.read_dir().unwrap() {
            let path = file.unwrap().path();
            if let Some(extension) = path.extension() {
                if let Some(extension) = extension.to_str() {
                    if extensions.iter().any(|e| e == extension) {
                        println!(
                            "{}",
                            capture_output(
                                Command::new("ln").args([
                                    "-s",
                                    "-F",
                                    "-f",
                                    "-v",
                                    path.to_str().unwrap()
                                ]),
                                "Failed to link"
                            )
                        );
                    }
                }
            }
        }
    }

    println!("Done.");

    Ok(())
}
