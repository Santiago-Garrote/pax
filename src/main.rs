use core::str;
use std::{
    fs::{create_dir, write},
    io::Error,
};

use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Command {
    /// Initiate a new empty library
    Init,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What command the app executes
    #[arg(value_enum)]
    command: Command,
}

const FLAKE_TEMPLATE: &str = include_str!("../templates/flake.nix.template");
const PAPERS_TEMPLATE: &str = include_str!("../templates/papers.nix.template");

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => match init() {
            Ok(_) => println!("Library Initiated!"),
            Err(e) => println!("{}", e),
        },
    }
}

fn init() -> Result<String, Error> {
    match create_dir("./research") {
        Ok(_) => println!("Dir created"),
        Err(e) => return Err(e),
    };
    match write("./research/flake.nix", FLAKE_TEMPLATE) {
        Ok(_) => println!("Flake created!"),
        Err(e) => return Err(e),
    };
    match write("./research/papers.nix", PAPERS_TEMPLATE) {
        Ok(_) => println!("Papers created!"),
        Err(e) => return Err(e),
    };
    Ok("Library initiated!".to_string())
}
