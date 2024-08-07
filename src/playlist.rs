use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use log::{debug, error};

use regex::Regex;

use super::utils::find_files;

#[derive(Debug, clap::Args)]
#[command(about = "Create playlist files for multidisc games")]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    generate: GenerateArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(about = "Generate playlist files for multidisc games")]
    Generate(GenerateArgs),
}

#[derive(Debug, clap::Args)]
struct GenerateArgs {
    #[arg(help = "The location to check for files")]
    source: PathBuf,
}

impl Args {
    pub fn dispatch(self) -> Result<(), String> {
        let cmd = self.command.unwrap_or(Commands::Generate(self.generate));
        match cmd {
            Commands::Generate(args) => generate_m3u_playlists(args.source),
        }
    }
}

fn generate_m3u_playlists(source: PathBuf) -> Result<(), String> {
    debug!("Generating playlists for files in {source:?}");

    let re = Regex::new(r"(?<before>.+) \(Disc (?<disc>\d+)\)(?<after>.*)\.[^.]+$").unwrap();

    let mut matches: HashMap<String, Vec<String>> = HashMap::new();

    for file in find_files(source.clone(), &["chd".to_string()]) {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let capture = re.captures(file_name);
        if let Some(capture) = capture {
            let before = capture.name("before").unwrap().as_str().to_string();
            let after = capture.name("after").unwrap().as_str().to_string();
            matches
                .entry(format!("{before}{after}"))
                .or_insert_with(|| vec![])
                .push(capture.get(0).unwrap().as_str().to_string())
        }
    }

    for (playlist, files) in &matches {
        let playlist_file = source.join(playlist).with_extension("m3u");
        if playlist_file.exists() {
            continue;
        }

        error!("Generating {playlist_file:?}");

        let mut f = File::create(playlist_file.clone()).expect("Unable to create playlist");
        for file in files {
            match f.write_all(&file.clone().into_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    error!("{e}");
                }
            }
        }
    }

    Ok(())
}
