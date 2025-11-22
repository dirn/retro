use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use log::{debug, error};

use super::utils::{find_files_with_extension, longest_common_prefix};

#[derive(Debug, clap::Args)]
#[command(about = "Rename files")]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    bin_cue: BinCueArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(about = "Rename bin/cue files")]
    BinCue(BinCueArgs),
}

#[derive(Debug, clap::Args)]
struct BinCueArgs {
    #[arg(help = "The location to check for files")]
    source: PathBuf,

    #[arg(help = "The new prefix to use, defaults to the source directory's name")]
    new: Option<String>,
}

impl Args {
    pub fn dispatch(self) -> Result<(), String> {
        let cmd = self.command.unwrap_or(Commands::BinCue(self.bin_cue));
        match cmd {
            Commands::BinCue(args) => rename_bin_cue_files(args.source, args.new),
        }
    }
}

fn rename_bin_cue_files(source: PathBuf, replacement_root: Option<String>) -> Result<(), String> {
    let new_prefix = match replacement_root {
        Some(replacement_root) => replacement_root,
        None => {
            let source_str = source
                .to_str()
                .ok_or_else(|| format!("Source path {} is not valid UTF-8", source.display()))?;
            // Remove trailing slash if present
            source_str
                .strip_suffix("/")
                .unwrap_or(source_str)
                .to_string()
        }
    };
    debug!("Renaming all bin and cue files in \"{source:?}\" to start with \"{new_prefix}\"");

    let mut file_names = Vec::new();
    let bin_cue_ext = ["bin".to_string(), "cue".to_string()];
    for file in find_files_with_extension(&source, &bin_cue_ext)? {
        if let Some(file_name) = file.file_name() {
            let file_name_str = file_name
                .to_str()
                .ok_or_else(|| format!("File name {} is not valid UTF-8", file.display()))?;
            file_names.push(file_name_str.to_string());
        }
    }

    let common = longest_common_prefix(&file_names);
    if common.is_empty() {
        return Err("No common prefix found".to_string());
    }

    for file_name in &file_names {
        let old_path = source.join(file_name);
        let new_file_name = file_name.replace(&common, &new_prefix);
        let new_path = source.join(&new_file_name);

        if let Err(e) = fs::rename(&old_path, &new_path) {
            error!(
                "Failed to rename {} to {}: {}",
                old_path.display(),
                new_path.display(),
                e
            );
            continue;
        }

        if let Some(ext) = new_path.extension() {
            if ext == "cue" {
                let contents = fs::read_to_string(&new_path).map_err(|e| {
                    format!("Failed to read cue file {}: {}", new_path.display(), e)
                })?;
                let new = contents.replace(&common, &new_prefix);
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&new_path)
                    .map_err(|e| {
                        format!(
                            "Failed to open cue file {} for writing: {}",
                            new_path.display(),
                            e
                        )
                    })?;
                file.write_all(new.as_bytes()).map_err(|e| {
                    format!("Failed to write to cue file {}: {}", new_path.display(), e)
                })?;
            }
        }
    }

    Ok(())
}
