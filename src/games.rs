use std::env::set_current_dir;
use std::fs::{canonicalize, create_dir_all, remove_file, symlink_metadata};
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, error, info, warn};

use super::config::load_link_destination_config;
use super::utils::{capture_output, find_files_with_extension};

pub fn clean(
    destination: &Path,
    systems: &[String],
    all_systems: bool,
    dry_run: bool,
) -> Result<(), String> {
    set_current_dir(destination).map_err(|e| {
        format!(
            "Failed to change directory to {}: {}",
            destination.display(),
            e
        )
    })?;

    let config = load_link_destination_config(None)?;

    let configured_systems = config.get_system_names();
    let systems_to_clean = if all_systems {
        &configured_systems
    } else {
        systems
    };

    for system in systems_to_clean {
        let Some(system_config) = config.systems.get(system) else {
            info!("{system} not found in config. Skipping.");
            continue;
        };

        let extensions = system_config.get_extensions(system);

        let destinations = system_config.get_destinations(system);
        for clean_destination in destinations {
            let mut path = destination.join(clean_destination);
            if let Some(extra_path) = &system_config.extra_path {
                path = path.join(extra_path);
            }
            let _ = set_current_dir(&path).is_ok();
            debug!("Checking for broken {extensions:?} links in {path:?}.");

            let files_to_clean = find_files_with_extension(&path, &extensions)?;

            for file in &files_to_clean {
                let metadata = symlink_metadata(file)
                    .map_err(|e| format!("Failed to get metadata for {}: {}", file.display(), e))?;
                if metadata.is_symlink() {
                    if canonicalize(file).is_err() {
                        if dry_run {
                            error!("Broken symlink found at {file:?}. Skipping.");
                        } else {
                            if let Err(e) = remove_file(file) {
                                error!("Failed to remove broken symlink {}: {}", file.display(), e);
                            } else {
                                error!("{file:?} unlinked");
                            };
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn link(
    source: &Path,
    destination: &Path,
    systems: &[String],
    all_systems: bool,
) -> Result<(), String> {
    set_current_dir(destination).map_err(|e| {
        format!(
            "Failed to change directory to {}: {}",
            destination.display(),
            e
        )
    })?;

    let config = load_link_destination_config(None)?;

    let configured_systems = config.get_system_names();
    let systems_to_link = if all_systems {
        &configured_systems
    } else {
        systems
    };

    for system in systems_to_link {
        let Some(system_config) = config.systems.get(system) else {
            info!("{system} not found in config. Skipping.");
            continue;
        };

        let system_source = Path::new(&source).join(&system_config.dumper).join(&system);
        if !system_source.is_dir() {
            info!("{} does not exist. Skipping.", system_source.display());
            continue;
        }

        let extensions = system_config.get_extensions(system);

        let files_to_link = find_files_with_extension(&system_source, &extensions)?;

        let destinations = system_config.get_destinations(system);
        for link_destination in destinations {
            let (current_system_source, path) = if let Some(extra_path) = &system_config.extra_path
            {
                (
                    system_source.join(extra_path),
                    destination.join(&link_destination).join(extra_path),
                )
            } else {
                (system_source.clone(), destination.join(&link_destination))
            };
            create_dir_all(&path)
                .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))?;
            let _ = set_current_dir(&path).is_ok();
            debug!("Linking {extensions:?} from {current_system_source:?} to {path:?}.");

            if !current_system_source.is_dir() {
                info!(
                    "{} does not exist. Skipping.",
                    current_system_source.display()
                );
                continue;
            }

            for file in &files_to_link {
                let destination_file_name = file
                    .file_name()
                    .ok_or_else(|| format!("Failed to get filename for {}", file.display()))?;
                let destination_path = path.join(destination_file_name);
                if destination_path.exists() {
                    let metadata = symlink_metadata(&destination_path).map_err(|e| {
                        format!(
                            "Failed to get metadata for {}: {}",
                            destination_path.display(),
                            e
                        )
                    })?;
                    if metadata.is_symlink() {
                        if let Ok(canonical) = canonicalize(&destination_path) {
                            if canonical == *file {
                                warn!("{destination_file_name:?} already linked. Skipping.");
                                continue;
                            }
                        }
                    }
                }
                let file_str = file.to_str().ok_or_else(|| {
                    format!("Failed to convert file path {} to UTF-8", file.display())
                })?;
                let output = capture_output(
                    &mut Command::new("ln").args(["-s", "-F", "-f", "-v", file_str]),
                    "Failed to link",
                )?;
                error!("{output}");
            }
        }
    }

    Ok(())
}
