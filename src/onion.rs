use std::env::set_current_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, warn};

use super::config::load_link_destination_config;
use super::utils::{capture_output, env_or_exit, find_files};

pub fn copy(source: &PathBuf, systems: &[String], all_systems: bool) -> Result<(), String> {
    let destination = env_or_exit("ONION_GAMES");

    let changed = set_current_dir(Path::new(&destination));
    if changed.is_err() {
        return Err(format!("{:#?}", changed.err()));
    };

    let config = match load_link_destination_config(None) {
        Ok(config) => config,
        Err(e) => {
            return Err(e);
        }
    };

    let configured_systems = config.get_system_names();
    let systems_to_copy = if all_systems {
        &configured_systems
    } else {
        systems
    };

    for system in systems_to_copy {
        let system_config = match config.systems.get(system) {
            Some(config) => config,
            None => {
                warn!("{system} not found in config. Skipping.");
                continue;
            }
        };

        let system_source = Path::new(&source).join(&system_config.dumper).join(&system);
        if !system_source.is_dir() {
            warn!("{} does not exist. Skipping.", system_source.display());
            continue;
        }

        let extensions = system_config.get_extensions(system);

        let files_to_copy = find_files(system_source.clone(), &extensions);

        let destinations = system_config.get_destinations(system);
        for copy_destination in destinations {
            let path = Path::new(&destination).join(copy_destination);
            debug!("Copying {extensions:?} from {system_source:?} to {path:?}.");

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

            let output = capture_output(&mut command, "Failed to copy");
            warn!("{output}");
        }
    }

    Ok(())
}
