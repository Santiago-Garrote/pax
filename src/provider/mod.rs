mod arxiv;
mod crossref;
mod openalex;
mod semantic_scholar;

pub use arxiv::ArxivProvider;
pub use crossref::CrossrefProvider;
pub use openalex::OpenAlexProvider;
pub use semantic_scholar::SemanticScholarProvider;

use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Which external metadata source a [`CandidateId`] or [`CandidateWork`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderId {
    OpenAlex,
    Crossref,
    SemanticScholar,
    ArXiv,
}

impl ProviderId {
    fn tag(self) -> &'static str {
        match self {
            ProviderId::OpenAlex => "openalex",
            ProviderId::Crossref => "crossref",
            ProviderId::SemanticScholar => "semanticscholar",
            ProviderId::ArXiv => "arxiv",
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown provider: {0}")]
pub struct UnknownProviderTag(String);

impl FromStr for ProviderId {
    type Err = UnknownProviderTag;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openalex" => Ok(ProviderId::OpenAlex),
            "crossref" => Ok(ProviderId::Crossref),
            "semanticscholar" => Ok(ProviderId::SemanticScholar),
            "arxiv" => Ok(ProviderId::ArXiv),
            other => Err(UnknownProviderTag(other.to_string())),
        }
    }
}

/// A fully-qualified, unambiguous reference to a single candidate work at a
/// single provider — e.g. `openalex:W2741809807` or `arxiv:2301.01234`.
///
/// This is the *only* reference `show`/`add` accept: it is always a provider's
/// own primary key, so resolving one is guaranteed to yield at most one
/// candidate. Deliberately does not accept a bare DOI, a title, or free text —
/// disambiguating "which paper did you mean" is `search`'s job, not this
/// type's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateId {
    pub provider: ProviderId,
    pub native_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CandidateIdParseError {
    #[error("candidate reference must be in the form 'provider:id', got: {0:?}")]
    MissingSeparator(String),
    #[error(transparent)]
    UnknownProvider(#[from] UnknownProviderTag),
    #[error("candidate reference has an empty id: {0:?}")]
    EmptyId(String),
}

impl fmt::Display for CandidateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.provider, self.native_id)
    }
}

impl FromStr for CandidateId {
    type Err = CandidateIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tag, native_id) = s
            .split_once(':')
            .ok_or_else(|| CandidateIdParseError::MissingSeparator(s.to_string()))?;
        if native_id.is_empty() {
            return Err(CandidateIdParseError::EmptyId(s.to_string()));
        }
        let provider = tag.parse::<ProviderId>()?;
        Ok(CandidateId {
            provider,
            native_id: native_id.to_string(),
        })
    }
}

/// An unresolved search hit from a single provider — not yet declared in the
/// local library. See [`CandidateId`] for how it's addressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateWork {
    pub id: CandidateId,
    pub title: String,
    pub authors: Vec<String>,
    pub publish_date: String,
    pub doi: Option<String>,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("not found")]
    NotFound,
    #[error("fetching by id is not supported by this provider yet")]
    GetByIdUnsupported,
}

/// A single external metadata source PAX can search and resolve candidates
/// from. Implementations live one per module in this directory.
// Implementations are only ever awaited directly (never spawned onto another
// task), so the missing `Send` bound on the returned futures doesn't matter here.
#[allow(async_fn_in_trait)]
pub trait Provider {
    fn id(&self) -> ProviderId;
    async fn search(&self, query: &str) -> Result<Vec<CandidateWork>, ProviderError>;
    async fn get(&self, native_id: &str) -> Result<CandidateWork, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_id_round_trips() {
        let id = CandidateId {
            provider: ProviderId::OpenAlex,
            native_id: "W123".to_string(),
        };
        assert_eq!(id.to_string(), "openalex:W123");
        assert_eq!("openalex:W123".parse::<CandidateId>().unwrap(), id);
    }

    #[test]
    fn candidate_id_rejects_bare_doi() {
        assert!(matches!(
            "10.1145/foo".parse::<CandidateId>(),
            Err(CandidateIdParseError::MissingSeparator(_))
        ));
    }

    #[test]
    fn candidate_id_rejects_free_text() {
        assert!(matches!(
            "actor model".parse::<CandidateId>(),
            Err(CandidateIdParseError::MissingSeparator(_))
        ));
    }

    #[test]
    fn candidate_id_rejects_unknown_provider() {
        assert!(matches!(
            "doi:10.1145/foo".parse::<CandidateId>(),
            Err(CandidateIdParseError::UnknownProvider(_))
        ));
    }

    #[test]
    fn candidate_id_rejects_empty_id() {
        assert!(matches!(
            "openalex:".parse::<CandidateId>(),
            Err(CandidateIdParseError::EmptyId(_))
        ));
    }
}
