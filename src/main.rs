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
    dest: String,

    #[structopt(short, long, help = "The system to synchronize.")]
    system: Vec<String>,
}

enum Method {
    SD,
    SSH,
}

fn dispatch() -> Result<(), String> {
    let cli = Cli::from_args();
    println!("{:?}", cli);

    let method = if cli.sd {
        Method::SD
    } else if cli.ssh {
        Method::SSH
    } else {
        return Err("A valid method is required".to_string());
    };

    let destination = match validate_destination(method, &cli.dest) {
        Ok(d) => d,
        Err(e) => {
            return Err(e);
        }
    };
    println!("destination: {:?}", destination);

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

fn validate_destination(method: Method, destination: &String) -> Result<&String, String> {
    match method {
        Method::SD => {
            if !PathBuf::from("/Volumes").join(destination).is_dir() {
                return Err(format!("'{}' is not a valid SD card label", destination));
            }
        }
        Method::SSH => {
            // There's no great way to check this unless the names are guaranteed to be detined
            // somewhere. instead just assume that the name is valid.
        }
    }
    Ok(destination)
}

fn validate_systems(src: PathBuf, systems: &Vec<String>) -> Result<&Vec<String>, String> {
    for system in systems {
        if !src.join(system).is_dir() {
            return Err(format!("'{}' is not a valid system", system));
        }
    }

    Ok(systems)
}
