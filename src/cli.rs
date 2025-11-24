use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

use env_logger;

use super::compress;
use super::link;
use super::playlist;
use super::rename;

#[derive(Debug, Parser)]
#[command(name = "retro")]
#[command(about = "synchronize retro games")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: Verbosity,
}

// Macro that defines the Commands enum and automatically generates the dispatch method.
// When you add a new variant to the enum, the dispatch method is automatically updated.
macro_rules! define_commands {
    (
        // Capture enum-level attributes (e.g., #[derive(...)])
        $(#[$enum_meta:meta])*
        // Capture the enum name
        enum $enum_name:ident {
            // Capture zero or more variants (the * means repetition)
            $(
                // Capture variant-level attributes (e.g., #[clap(...)])
                $(#[$variant_meta:meta])*
                // Capture variant name and its type
                $variant:ident($variant_type:ty),
            )*
        }
    ) => {
        // Re-emit the enum definition with all attributes preserved
        $(#[$enum_meta])*
        enum $enum_name {
            $(
                $(#[$variant_meta])*
                $variant($variant_type),
            )*
        }

        // Automatically generate the dispatch implementation
        impl $enum_name {
            fn dispatch(self) -> Result<(), String> {
                match self {
                    // Generate a match arm for each variant
                    // Each arm extracts the args and calls its dispatch method
                    $(
                        $enum_name::$variant(args) => args.dispatch(),
                    )*
                }
            }
        }
    };
}

define_commands! {
    #[derive(Debug, Subcommand)]
    enum Commands {
        #[clap(visible_alias = "chd")]
        Compress(compress::Args),
        Link(link::Args),
        #[clap(visible_alias = "m3u")]
        Playlist(playlist::Args),
        Rename(rename::Args),
    }
}

pub fn dispatch() -> Result<(), String> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .format_level(false)
        .format_target(false)
        .format_timestamp(None)
        .init();

    args.command.dispatch()
}
