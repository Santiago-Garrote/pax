use thiserror::Error;

use crate::provider::{CandidateIdParseError, ProviderError};

#[derive(Debug, Error)]
pub enum PaxError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse library: {0}")]
    LibraryParse(String),
    #[error(transparent)]
    InvalidReference(#[from] CandidateIdParseError),
    #[error("no paper found for {0:?}: not declared locally, and not a valid candidate reference")]
    NotFound(String),
    #[error("no paper found with citation key {0:?}")]
    NoSuchPaper(String),
    #[error(
        "nothing to edit: specify --add-tag, --remove-tag, --notes, --rename, --title, \
         --author, --year, and/or --doi"
    )]
    NoChangesSpecified,
    #[error("no PDF source recorded for {0:?}; nothing to fetch")]
    NoSourceUrl(String),
    #[error("fetch failed: {0}")]
    Fetch(String),
    #[error("build failed: {0}")]
    Build(String),
    #[error("upload failed: {0}")]
    Upload(String),
    #[error("citation key {0:?} is already in use")]
    CitationKeyExists(String),
    #[error(
        "{0:?} is not a valid citation key (must start with a letter or underscore, and \
         contain only letters, digits, underscores, and hyphens)"
    )]
    InvalidCitationKey(String),
}
