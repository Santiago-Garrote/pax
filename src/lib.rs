//! `pax-core`: the paper-discovery, declaration, and library domain logic for
//! PAX. This is the primary artifact of this repository — any CLI or other
//! adapter (including the `pax` binary in `src/bin/pax.rs`, or a future
//! `lazypax` TUI) is a client of this library, not the other way around.

pub mod error;
pub mod library;
pub mod nix;
pub mod paper;
pub mod provider;

use std::collections::HashMap;

pub use error::PaxError;
pub use library::Library;
pub use nix::init_library;
pub use paper::{Artifact, Identity, Local, Paper, PaperRef};
pub use provider::{
    ArxivProvider, CandidateId, CandidateWork, CrossrefProvider, OpenAlexProvider, Provider,
    ProviderError, ProviderId, SemanticScholarProvider,
};

/// Searches every configured provider and aggregates results by provider.
/// A failing provider doesn't fail the whole search — its error is reported
/// alongside whatever providers did succeed, so callers (e.g. the CLI) can
/// decide how to surface it.
pub async fn search_all(query: &str) -> HashMap<ProviderId, Result<Vec<CandidateWork>, ProviderError>> {
    let mut results = HashMap::new();

    let openalex = OpenAlexProvider::new();
    results.insert(ProviderId::OpenAlex, openalex.search(query).await);

    match CrossrefProvider::new() {
        Ok(crossref) => {
            results.insert(ProviderId::Crossref, crossref.search(query).await);
        }
        Err(e) => {
            results.insert(ProviderId::Crossref, Err(e));
        }
    }

    // TODO(deferred): hardcoded API key, see docs/mvp.md gap list — moving
    // this to configuration is out of scope for this restructuring.
    match SemanticScholarProvider::new("s2k-lqEUK8qhHH5MGk6NKa76zDxmrZxXB6wPJ5uWsPJJ") {
        Ok(semantic_scholar) => {
            results.insert(
                ProviderId::SemanticScholar,
                semantic_scholar.search(query).await,
            );
        }
        Err(e) => {
            results.insert(ProviderId::SemanticScholar, Err(e));
        }
    }

    match ArxivProvider::new("santiagogarrote2005@gmail.com") {
        Ok(arxiv) => {
            results.insert(ProviderId::ArXiv, arxiv.search(query).await);
        }
        Err(e) => {
            results.insert(ProviderId::ArXiv, Err(e));
        }
    }

    results
}
