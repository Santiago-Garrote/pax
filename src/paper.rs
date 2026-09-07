/// A citation key identifying a paper already declared in the local library
/// (e.g. `turing1936`). Distinct from [`crate::provider::CandidateId`]: this
/// addresses the closed set of papers PAX already knows about, not the open
/// set of anything a provider might return.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaperRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Identity {
    pub doi: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Artifact {
    pub source_url: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Local {
    pub citation_key: String,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

/// A paper declared in the local library, per the three-part shape from
/// docs/mvp.md: identity (what the paper is), artifact (where/how Nix
/// fetches it), and local metadata (how the user organizes it).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Paper {
    pub identity: Identity,
    pub artifact: Artifact,
    pub local: Local,
}
