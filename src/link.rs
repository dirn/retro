use std::collections::HashMap;
use std::env::set_current_dir;
use std::fs::read_to_string;
use std::path::Path;
use std::process::{exit, Command};

use toml;

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

#[derive(Debug, serde::Deserialize)]
struct Config {
    systems: HashMap<String, System>,
}

#[derive(Debug, serde::Deserialize)]
struct System {
    dumper: String,
    extension: Option<String>,
    extensions: Option<Vec<String>>,
    extra_path: Option<String>,
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

    let config_path = Path::new("systems.toml");
    let data = match read_to_string(config_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("read_to_string: {e:#?}");
            exit(1);
        }
    };

    let config: Config = match toml::from_str(&data) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("from_str: {e:#?}");
            exit(1);
        }
    };

    let systems_to_link = if all_systems {
        Vec::from_iter(config.systems.keys().map(|k| k.to_string()))
    } else {
        systems
    };

    for system in systems_to_link {
        let path = Path::new(&destination).join(&system);

        let system_config = match config.systems.get(&system) {
            Some(config) => config,
            None => {
                eprintln!("{system} not found in {config_path:?}. Skipping.");
                continue;
            }
        };
        let extensions = if system_config.extensions.is_some() {
            system_config.extensions.clone().unwrap()
        } else {
            vec![system_config.extension.clone().unwrap_or(system.clone())]
        };

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
