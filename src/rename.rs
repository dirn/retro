use std::fs;
use std::io::prelude::*;

use glob::{glob_with, MatchOptions};

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
    #[arg(help = "The current common root of the files to rename")]
    current: String,

    #[arg(help = "The new common root of the files to rename")]
    new: String,
}

pub fn dispatch(args: Args) -> Result<(), String> {
    let cmd = args.command.unwrap_or(Commands::BinCue(args.bin_cue));

    match cmd {
        Commands::BinCue(args) => {
            return rename_bin_cue_files(args.current, args.new);
        }
    }
}

fn rename_bin_cue_files(current_root: String, replacement_root: String) -> Result<(), String> {
    println!("Renaming all files that start with \"{current_root}\" to \"{replacement_root}\"");

    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    for entry in glob_with(&format!("{current_root}*"), options).unwrap() {
        if let Ok(path) = entry {
            if path.extension().unwrap() == "cue" {
                let contents = fs::read_to_string(path.clone()).unwrap();
                let new = contents.replace(&current_root, &replacement_root);
                match fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(path.clone())
                {
                    Ok(mut file) => {
                        let _ = file.write(new.as_bytes());
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                };
            }

            let old_name = path.file_name().unwrap();
            let new_name = old_name
                .to_str()
                .unwrap()
                .replace(&current_root, &replacement_root);
            match fs::rename(old_name, new_name) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                }
            }
        }
    }

    Ok(())
}
