use core::str;
use std::{
    fs::{create_dir, write},
    io::Error,
};

use clap::{Parser, Subcommand};
use papers_openalex::{ListParams, ListResponse, OpenAlexClient, OpenAlexError, Work};

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

const FLAKE_TEMPLATE: &str = include_str!("../templates/flake.nix.template");
const PAPERS_TEMPLATE: &str = include_str!("../templates/papers.nix.template");

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            let _result = init();
        }
        Command::Search { query } => {
            let response = search(query).await.unwrap();
            for work in response.results {
                println!("{}", work.display_name.as_deref().unwrap());
            }
        }
    }
}

fn init() -> Result<(), Error> {
    create_dir("./research")?;
    write("./research/flake.nix", FLAKE_TEMPLATE)?;
    write("./research/papers.nix", PAPERS_TEMPLATE)?;
    println!("Library created");
    Ok(())
}

async fn search(query: String) -> Result<ListResponse<Work>, OpenAlexError> {
    let api_key = std::env::var("OPENALEX_API_KEY").expect("OPENALEX_API_KEY must be set");
    let client: OpenAlexClient = OpenAlexClient::with_api_key(api_key);
    let params = ListParams::builder().search(query).build();
    client.list_works(&params).await
}
