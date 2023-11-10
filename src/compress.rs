use std::path::PathBuf;

use log::{debug, warn};

use super::utils::{find_files, require_command, stream_output};

#[derive(Debug, clap::Args)]
#[command(about = "Compress games")]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    chd: ChdArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(about = "Convert files to CHD")]
    Chd(ChdArgs),
}

#[derive(Debug, clap::Args)]
struct ChdArgs {
    #[arg(help = "The file to compress")]
    source: PathBuf,

    #[arg(help = "Where to place the compressed file, defaults to the current directory")]
    dest: Option<PathBuf>,
}

pub fn dispatch(args: Args) -> Result<(), String> {
    let cmd = args.command.unwrap_or(Commands::Chd(args.chd));
    match cmd {
        Commands::Chd(args) => {
            return compress_to_chd(args.source, args.dest.clone());
        }
    }
}

fn compress_to_chd(source: PathBuf, dest: Option<PathBuf>) -> Result<(), String> {
    let output_path = dest.unwrap_or(PathBuf::new());
    debug!("Compressing from {source:?} to {output_path:?}");

    let files_to_compress = find_files(source, &["cue".to_string(), "iso".to_string()]);

    for file in files_to_compress {
        let mut output_file = output_path.join(file.file_name().unwrap());
        output_file.set_extension("chd");
        if output_file.exists() {
            warn!("{output_file:?} exists. Skipping.");
            continue;
        }

        stream_output(
            require_command("chdman").args(&[
                "createcd",
                "-i",
                file.to_str().unwrap(),
                "-o",
                output_file.to_str().unwrap(),
            ]),
            "Could not compress {file:?}",
        );
    }

    Ok(())
}
