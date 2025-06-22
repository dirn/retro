mod cli;
mod compress;
mod config;
mod dat;
mod games;
mod link;
mod playlist;
mod rename;
mod utils;

use std::process::exit;

use log::error;

fn main() {
    exit(match cli::dispatch() {
        Ok(_) => 0,
        Err(e) => {
            error!("error: {:?}", e);
            1
        }
    });
}
