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

// TODO(deferred): hardcoded API key, see docs/mvp.md gap list — moving this
// to configuration is out of scope for now.
fn semantic_scholar_provider() -> Result<SemanticScholarProvider, ProviderError> {
    SemanticScholarProvider::new("s2k-lqEUK8qhHH5MGk6NKa76zDxmrZxXB6wPJ5uWsPJJ")
}

fn arxiv_provider() -> Result<ArxivProvider, ProviderError> {
    ArxivProvider::new("santiagogarrote2005@gmail.com")
}

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

    match semantic_scholar_provider() {
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

    match arxiv_provider() {
        Ok(arxiv) => {
            results.insert(ProviderId::ArXiv, arxiv.search(query).await);
        }
        Err(e) => {
            results.insert(ProviderId::ArXiv, Err(e));
        }
    }

    results
}

/// Resolves a single, already-unambiguous candidate reference by asking its
/// provider directly for that id — no search or disambiguation involved.
pub async fn resolve_candidate(id: &CandidateId) -> Result<CandidateWork, ProviderError> {
    match id.provider {
        ProviderId::OpenAlex => OpenAlexProvider::new().get(&id.native_id).await,
        ProviderId::Crossref => CrossrefProvider::new()?.get(&id.native_id).await,
        ProviderId::SemanticScholar => semantic_scholar_provider()?.get(&id.native_id).await,
        ProviderId::ArXiv => arxiv_provider()?.get(&id.native_id).await,
    }
}
