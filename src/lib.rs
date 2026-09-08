//! `pax-core`: the paper-discovery, declaration, and library domain logic for
//! PAX. This is the primary artifact of this repository — any CLI or other
//! adapter (including the `pax` binary in `src/bin/pax/`, or a future
//! `lazypax` TUI) is a client of this library, not the other way around.

mod citation_key;
pub mod error;
pub mod library;
pub mod nix;
pub mod paper;
pub mod provider;

use std::collections::HashMap;
use std::path::Path;

pub use error::PaxError;
pub use library::Library;
pub use nix::init_library;
pub use paper::{Artifact, Identity, Local, Paper, PaperRef};
use paper::year_from_publish_date;
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

/// Resolves a candidate and declares it in the local library at `root`
/// (`research/papers.nix`) — metadata only. `Artifact.source_url`/`hash`
/// stay unset; fetching bytes and computing a Nix hash is `fetch`'s job,
/// not `add`'s (lazy materialization, see docs/mvp.md).
pub async fn add_candidate(id: &CandidateId, root: &Path) -> Result<PaperRef, PaxError> {
    let work = resolve_candidate(id).await?;
    let path = nix::papers_path(root);
    let mut library = Library::load(&path)?;

    let existing_keys: Vec<&str> = library
        .papers()
        .iter()
        .map(|p| p.local.citation_key.as_str())
        .collect();
    let citation_key = citation_key::generate(&work, &existing_keys);

    library.insert(Paper {
        identity: Identity {
            doi: work.doi.clone(),
            title: work.title.clone(),
            authors: work.authors.clone(),
            year: year_from_publish_date(&work.publish_date),
        },
        artifact: Artifact::default(),
        local: Local {
            citation_key: citation_key.clone(),
            tags: Vec::new(),
            notes: None,
        },
    });
    library.save(&path)?;

    Ok(PaperRef(citation_key))
}

/// The result of resolving a `show` reference: either a paper already
/// declared in the local library, or an unresolved candidate from a
/// provider.
pub enum ShowResult {
    Declared(Paper),
    Candidate(CandidateWork),
}

/// Resolves a `show` reference. A `CandidateId` always contains `:`
/// (`provider:native_id`); a citation key, by construction of
/// `citation_key::generate`, never does — so the colon's presence decides
/// which address space `reference` belongs to before either lookup is
/// attempted, rather than trying one and falling back to the other.
pub async fn show_reference(reference: &str, root: &Path) -> Result<ShowResult, PaxError> {
    if !reference.contains(':') {
        let path = nix::papers_path(root);
        if let Ok(library) = Library::load(&path)
            && let Some(paper) = library.find(reference)
        {
            return Ok(ShowResult::Declared(paper.clone()));
        }
        return Err(PaxError::NotFound(reference.to_string()));
    }

    let id: CandidateId = reference.parse()?;
    let work = resolve_candidate(&id).await?;
    Ok(ShowResult::Candidate(work))
}

/// Removes a declared paper from the local library at `root`. Only the
/// declaration/metadata is removed — Nix remains responsible for garbage
/// collecting any unused artifacts on its own (see docs/mvp.md).
pub fn remove_paper(citation_key: &str, root: &Path) -> Result<(), PaxError> {
    let path = nix::papers_path(root);
    let mut library = Library::load(&path)?;
    if !library.remove(citation_key) {
        return Err(PaxError::NoSuchPaper(citation_key.to_string()));
    }
    library.save(&path)?;
    Ok(())
}
