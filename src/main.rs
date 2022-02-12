use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "retro", about = "Synchronize retro games.")]
struct Cli {
    #[structopt(long, group = "method", help = "Synchronize to an SD card.")]
    sd: bool,

    #[structopt(long, group = "method", help = "Synchronize over SSH.")]
    ssh: bool,

    #[structopt(long, env = "RETRO_GAMES", help = "The location to synchronize from.")]
    src: PathBuf,

    #[structopt(short, long, help = "The volume or host to synchronize to.")]
    dest: Option<String>,

    #[structopt(short, long, help = "The system to synchronize.")]
    system: Vec<String>,
}

fn dispatch() -> Result<(), String> {
    let cli = Cli::from_args();
    println!("{:?}", cli);

    let systems = match validate_systems(cli.src, &cli.system) {
        Ok(s) => s,
        Err(e) => {
            return Err(e);
        }
    };
    println!("systems: {:?}", systems);

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

fn validate_systems(src: PathBuf, systems: &Vec<String>) -> Result<&Vec<String>, String> {
    for system in systems {
        if !src.join(system).is_dir() {
            return Err(format!("'{}' is not a valid system", system));
        }
    }

    Ok(systems)
}
