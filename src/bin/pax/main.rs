use std::path::Path;

use clap::{Parser, Subcommand};
use pax_core::{CandidateId, ProviderId};

mod sink;
use sink::{Sink, TextSink};

#[derive(Subcommand)]
enum Command {
    ///Initiate a new empty library
    Init,
    ///Search for papers
    Search { query: String },
    ///Inspect a search result before adding it
    Show {
        /// A fully-qualified reference from `search`, e.g. openalex:W2072794470
        reference: String,
    },
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
    let mut sink = TextSink;

    match cli.command {
        Command::Init => match pax_core::init_library(Path::new(".")) {
            Ok(()) => sink.message("Library created"),
            Err(e) => sink.error(&e.to_string()),
        },
        Command::Search { query } => {
            let results = pax_core::search_all(&query).await;
            for provider in [
                ProviderId::OpenAlex,
                ProviderId::Crossref,
                ProviderId::SemanticScholar,
                ProviderId::ArXiv,
            ] {
                sink.provider_header(provider);
                match results.get(&provider) {
                    Some(Ok(works)) => sink.candidates(works),
                    Some(Err(e)) => sink.error(&e.to_string()),
                    None => {}
                }
            }
        }
        Command::Show { reference } => {
            let id: CandidateId = match reference.parse() {
                Ok(id) => id,
                Err(e) => {
                    sink.error(&e.to_string());
                    return;
                }
            };
            match pax_core::resolve_candidate(&id).await {
                Ok(work) => sink.candidate(&work),
                Err(e) => sink.error(&e.to_string()),
            }
        }
    }
}
