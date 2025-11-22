use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use log::{debug, error};

use regex::Regex;

use super::utils::find_files_with_extension;

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

    let re = Regex::new(r"(?<before>.+) \(Disc (?<disc>\d+)\)(?<after>.*)\.[^.]+$")
        .map_err(|e| format!("Failed to compile regex: {}", e))?;

    let mut matches: HashMap<String, Vec<String>> = HashMap::new();

    for file in find_files_with_extension(&source, &["chd".to_string()])? {
        let file_name = file
            .file_name()
            .ok_or_else(|| format!("File {} has no filename", file.display()))?
            .to_str()
            .ok_or_else(|| format!("File name {} is not valid UTF-8", file.display()))?;
        let capture = re.captures(file_name);
        if let Some(capture) = capture {
            let before = capture
                .name("before")
                .ok_or_else(|| format!("Regex capture 'before' not found for {}", file_name))?
                .as_str();
            let after = capture
                .name("after")
                .ok_or_else(|| format!("Regex capture 'after' not found for {}", file_name))?
                .as_str();
            let full_match = capture
                .get(0)
                .ok_or_else(|| format!("Regex full match not found for {}", file_name))?
                .as_str();
            matches
                .entry(format!("{before}{after}"))
                .or_default()
                .push(full_match.to_string())
        }
    }

    for (playlist, files) in &matches {
        let playlist_file = source.join(playlist).with_extension("m3u");
        if playlist_file.exists() {
            continue;
        }

        error!("Generating {playlist_file:?}");

        let mut f = File::create(&playlist_file).map_err(|e| {
            format!(
                "Unable to create playlist {}: {}",
                playlist_file.display(),
                e
            )
        })?;
        for file in files {
            f.write_all(file.as_bytes()).map_err(|e| {
                format!(
                    "Failed to write to playlist {}: {}",
                    playlist_file.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}
