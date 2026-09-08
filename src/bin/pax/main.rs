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
    ///Declare a search result in the local library
    Add {
        /// A fully-qualified reference from `search`, e.g. openalex:W2072794470
        reference: String,
    },
    ///List papers already declared in the library
    List,
    ///Remove a declared paper from the library
    Remove {
        /// The paper's citation key, e.g. turing1936
        citation_key: String,
    },
    ///Modify a declared paper's tags or notes
    Edit {
        /// The paper's citation key, e.g. turing1936
        citation_key: String,
        /// Add a tag (repeatable)
        #[arg(long = "add-tag")]
        add_tag: Vec<String>,
        /// Remove a tag (repeatable)
        #[arg(long = "remove-tag")]
        remove_tag: Vec<String>,
        /// Set the paper's notes
        #[arg(long)]
        notes: Option<String>,
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
            match pax_core::show_reference(&reference, Path::new(".")).await {
                Ok(pax_core::ShowResult::Declared(paper)) => sink.paper(&paper),
                Ok(pax_core::ShowResult::Candidate(work)) => sink.candidate(&work),
                Err(e) => sink.error(&e.to_string()),
            }
        }
        Command::Add { reference } => {
            let id: CandidateId = match reference.parse() {
                Ok(id) => id,
                Err(e) => {
                    sink.error(&e.to_string());
                    return;
                }
            };
            match pax_core::add_candidate(&id, Path::new(".")).await {
                Ok(paper_ref) => sink.message(&format!("Added {}", paper_ref.0)),
                Err(e) => sink.error(&e.to_string()),
            }
        }
        Command::List => {
            match pax_core::Library::load(&pax_core::nix::papers_path(Path::new("."))) {
                Ok(library) if library.papers().is_empty() => sink.message("Library is empty"),
                Ok(library) => sink.papers(library.papers()),
                Err(e) => sink.error(&e.to_string()),
            }
        }
        Command::Remove { citation_key } => {
            match pax_core::remove_paper(&citation_key, Path::new(".")) {
                Ok(()) => sink.message(&format!("Removed {citation_key}")),
                Err(e) => sink.error(&e.to_string()),
            }
        }
        Command::Edit {
            citation_key,
            add_tag,
            remove_tag,
            notes,
        } => {
            match pax_core::edit_paper(
                &citation_key,
                Path::new("."),
                &add_tag,
                &remove_tag,
                notes.as_deref(),
            ) {
                Ok(()) => sink.message(&format!("Updated {citation_key}")),
                Err(e) => sink.error(&e.to_string()),
            }
        }
    }
}
