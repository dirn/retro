use std::path::PathBuf;

use log::{debug, log_enabled, warn, Level};

use super::config::load_config_recursively;
use super::utils::{capture_output, find_files_with_extension, require_command, stream_output};

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

    #[arg(long, help = "Create a compressed DVD image")]
    dvd: bool,

    #[arg(short, long, help = "Force overwriting existing CHD files")]
    force: bool,
}

impl Args {
    pub fn dispatch(self) -> Result<(), String> {
        let cmd = self.command.unwrap_or(Commands::Chd(self.chd));
        match cmd {
            Commands::Chd(args) => {
                compress_to_chd(args.source, args.dest.clone(), args.dvd, args.force)
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    compress: CompressConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compress: CompressConfig::default(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CompressConfig {
    pub extensions: Vec<String>,
    pub format: String,
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self {
            extensions: vec!["cue".to_string(), "iso".to_string()],
            format: "cd".to_string(),
        }
    }
}

fn compress_to_chd(
    source: PathBuf,
    dest: Option<PathBuf>,
    as_dvd: bool,
    force: bool,
) -> Result<(), String> {
    let output_path = dest.unwrap_or(PathBuf::new());
    debug!("Compressing from {source:?} to {output_path:?}");

    let config: CompressConfig = match load_config_recursively(&source) {
        Ok(config) => config,
        Err(_) => {
            debug!("No custom config found, using default compression settings");
            Config::default()
        }
    }
    .compress;

    let files_to_compress = find_files_with_extension(&source, &config.extensions);

    let mut image_format: &str = &format!("create{}", config.format);
    if as_dvd {
        image_format = "createdvd";
    }

    for file in files_to_compress {
        let mut output_file = output_path.join(file.file_name().unwrap());
        output_file.set_extension("chd");
        if !force && output_file.exists() {
            warn!("{} exists. Skipping.", output_file.display());
            continue;
        }

        let mut command = require_command("chdman");
        command.args(&[
            image_format,
            "-i",
            file.to_str().unwrap(),
            "-o",
            output_file.to_str().unwrap(),
        ]);
        if force {
            command.arg("--force");
        }
        let error_message = format!("Could not compress {file:?}");

        if log_enabled!(Level::Warn) {
            stream_output(&mut command, &error_message);
        } else {
            let _ = capture_output(&mut command, &error_message);
            warn!("{} created with {image_format}", output_file.display());
        }
    }

    Ok(())
}
