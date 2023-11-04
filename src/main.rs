mod cli;
mod compress;
mod config;
mod games;
mod link;
mod onion;
mod rename;
mod utils;

use std::process::exit;

fn main() {
    exit(match cli::dispatch() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("error: {:?}", e);
            1
        }
    });
}
