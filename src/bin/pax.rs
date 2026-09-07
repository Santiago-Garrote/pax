use std::path::Path;

use clap::{Parser, Subcommand};
use pax_core::ProviderId;

#[derive(Subcommand)]
enum Command {
    ///Initiate a new empty library
    Init,
    ///Search for papers
    Search { query: String },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Command::Init => match pax_core::init_library(Path::new(".")) {
            Ok(()) => println!("Library created"),
            Err(e) => println!("Error creating library: {}", e),
        },
        Command::Search { query } => {
            let results = pax_core::search_all(&query).await;
            for provider in [
                ProviderId::OpenAlex,
                ProviderId::Crossref,
                ProviderId::SemanticScholar,
                ProviderId::ArXiv,
            ] {
                println!("{}:", provider);
                match results.get(&provider) {
                    Some(Ok(works)) => {
                        for work in works {
                            println!("\t{}", work.title);
                            println!("\t\t{}", work.id);
                            println!("\t\t{}", work.doi);
                        }
                    }
                    Some(Err(e)) => println!("\tError: {}", e),
                    None => {}
                }
            }
        }
    }
}
