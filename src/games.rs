use std::env::set_current_dir;
use std::fs::{canonicalize, symlink_metadata};
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, info, warn};

use super::config::load_link_destination_config;
use super::utils::{capture_output, env_or_exit, find_files};

pub fn link(source: &PathBuf, systems: &[String], all_systems: bool) -> Result<(), String> {
    let destination = env_or_exit("RETRO_GAMES");

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
    let systems_to_link = if all_systems {
        &configured_systems
    } else {
        systems
    };

    for system in systems_to_link {
        let mut path = Path::new(&destination).join(&system);

        let system_config = match config.systems.get(system) {
            Some(config) => config,
            None => {
                warn!("{system} not found in config. Skipping.");
                continue;
            }
        };

        let extensions = system_config.get_extensions(system);

        let mut system_source = Path::new(&source).join(&system_config.dumper).join(&system);
        if let Some(extra_path) = &system_config.extra_path {
            system_source = system_source.join(extra_path);
            path = path.join(extra_path);
        }
        let _ = set_current_dir(&path).is_ok();
        debug!("Linking {extensions:?} from {system_source:?} to {path:?}.");

        if !system_source.is_dir() {
            warn!("{} does not exist. Skipping.", system_source.display());
            continue;
        }

        let files_to_link = find_files(system_source.clone(), &extensions);

        for file in files_to_link {
            let destination_file_name = file.file_name().unwrap();
            let destination_path = path.join(destination_file_name);
            if destination_path.exists() {
                let metadata = symlink_metadata(&destination_path).unwrap();
                if metadata.is_symlink() {
                    if canonicalize(&destination_path).unwrap() == file {
                        info!("{destination_file_name:?} already linked. Skipping.");
                        continue;
                    }
                }
            }
            let output = capture_output(
                Command::new("ln").args(["-s", "-F", "-f", "-v", file.to_str().unwrap()]),
                "Failed to link",
            );
            warn!("{output}");
        }
    }

    Ok(())
}
