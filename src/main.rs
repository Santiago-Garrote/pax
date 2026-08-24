use core::str;
use std::{
    fs::{create_dir, write},
    io::Error,
};

use clap::{Parser, Subcommand};
use crossref::{Crossref, WorkList};
use papers_openalex::{ListParams, ListResponse, OpenAlexClient, OpenAlexError};
use semantic_scholar::{Paper, SemanticScholar};

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
        Command::Search { query } => match search(query).await {
            Ok(works) => {
                for work in works {
                    println!("{}", work.title);
                }
            }
            Err(e) => println!("Error in search: {}", e),
        },
    }
}

fn init() -> Result<(), Error> {
    create_dir("./research")?;
    write("./research/flake.nix", FLAKE_TEMPLATE)?;
    write("./research/papers.nix", PAPERS_TEMPLATE)?;
    println!("Library created");
    Ok(())
}

struct Work {
    title: String,
}

impl From<papers_openalex::Work> for Work {
    fn from(value: papers_openalex::Work) -> Self {
        Work {
            title: value.title.expect("OpenAlex Work with no title found"),
        }
    }
}

impl From<crossref::Work> for Work {
    fn from(value: crossref::Work) -> Self {
        Work {
            title: value.title.into_iter().next().unwrap_or_default(),
        }
    }
}

impl From<semantic_scholar::Paper> for Work {
    fn from(value: semantic_scholar::Paper) -> Self {
        Work {
            title: value.title.expect("Paper does not have title!"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum SearchError {
    #[error("request failed: {0}")]
    Request(String),
}

impl From<OpenAlexError> for SearchError {
    fn from(value: OpenAlexError) -> Self {
        SearchError::Request(value.to_string())
    }
}

impl From<crossref::Error> for SearchError {
    fn from(value: crossref::Error) -> Self {
        SearchError::Request(value.to_string())
    }
}

impl From<semantic_scholar::Error> for SearchError {
    fn from(value: semantic_scholar::Error) -> Self {
        SearchError::Request(value.to_string())
    }
}

async fn search(query: String) -> Result<Vec<Work>, SearchError> {
    let mut final_results: Vec<Work> = Vec::new();
    match open_alex_search(query.clone()).await {
        Ok(alex_results) => final_results.extend(alex_results.results.into_iter().map(Work::from)),
        Err(e) => println!("OpenAlex error: {}", e),
    };
    match crossref_search(query.clone()).await {
        Ok(crossref_results) => {
            final_results.extend(crossref_results.items.into_iter().map(Work::from));
        }
        Err(e) => println!("Crossref error: {}", e),
    };
    match semanticscholar_search(query.clone()).await {
        Ok(semanticscholar_results) => {
            final_results.extend(semanticscholar_results.into_iter().map(Work::from));
        }
        Err(e) => println!("Semmantic Scholar error: {}", e),
    }
    Ok(final_results)
}

async fn open_alex_search(
    query: String,
) -> Result<ListResponse<papers_openalex::Work>, OpenAlexError> {
    let client: OpenAlexClient = OpenAlexClient::new();
    let params = ListParams::builder().search(query).build();
    client.list_works(&params).await
}

async fn crossref_search(query: String) -> Result<WorkList, crossref::Error> {
    let client = Crossref::builder().build()?;
    client.works(query)
}

async fn semanticscholar_search(query: String) -> Result<Vec<Paper>, semantic_scholar::Error> {
    let client = SemanticScholar::with_api_key("s2k-lqEUK8qhHH5MGk6NKa76zDxmrZxXB6wPJ5uWsPJJ")?;
    let result = client.search_papers(&query).send().await?;
    Ok(result.data)
}
