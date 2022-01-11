use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "retro", about = "Synchronize retro games.")]
struct Cli {
    #[structopt(long, group = "method", help = "Synchronize to an SD card.")]
    sd: bool,

    #[structopt(long, group = "method", help = "Synchronize over SSH.")]
    ssh: bool,

    #[structopt(short, long, help = "The volume or host to synchronize to.")]
    dest: Option<String>,

    #[structopt(short, long, help = "The system to synchronize.")]
    system: Vec<String>,
}

fn dispatch() -> Result<(), String> {
    let cli = Cli::from_args();
    println!("{:?}", cli);
    Ok(())
}

fn main() {
    process::exit(match dispatch() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{:?}", e);
            1
        }
    });
}
