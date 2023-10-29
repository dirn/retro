mod cli;
mod link;
mod onion;
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
