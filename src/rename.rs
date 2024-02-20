use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use log::{debug, error};

use super::utils::{find_files, longest_common_prefix};

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

pub fn dispatch(args: Args) -> Result<(), String> {
    let cmd = args.command.unwrap_or(Commands::BinCue(args.bin_cue));
    match cmd {
        Commands::BinCue(args) => rename_bin_cue_files(args.source, args.new),
    }
}

fn rename_bin_cue_files(source: PathBuf, replacement_root: Option<String>) -> Result<(), String> {
    let new_prefix = match replacement_root {
        Some(replacement_root) => replacement_root,
        None => {
            let tmp = source.to_str().unwrap();
            tmp.strip_suffix("/").unwrap_or(tmp).to_string()
        }
    };
    debug!("Renaming all bin and cue files in \"{source:?}\" to start with \"{new_prefix}\"");

    let mut file_names = Vec::new();
    for file in find_files(source.clone(), &["bin".to_string(), "cue".to_string()]) {
        if let Some(file_name) = file.file_name() {
            file_names.push(file_name.to_str().unwrap().to_string());
        }
    }

    let common = longest_common_prefix(&file_names);
    if common.is_empty() {
        return Err("No common prefix found".to_string());
    }

    for file_name in &file_names {
        let old_path = source.join(file_name);
        let new_file_name = file_name.replace(&common, &new_prefix);
        let new_path = source.join(new_file_name);

        match fs::rename(old_path, new_path.clone()) {
            Ok(_) => (),
            Err(e) => {
                error!("{e}");
            }
        }

        if new_path.extension().unwrap() == "cue" {
            let contents = fs::read_to_string(new_path.clone()).unwrap();
            let new = contents.replace(&common, &new_prefix);
            match fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(new_path.clone())
            {
                Ok(mut file) => {
                    let _ = file.write(new.as_bytes());
                }
                Err(e) => {
                    error!("{e}");
                }
            };
        }
    }

    Ok(())
}
