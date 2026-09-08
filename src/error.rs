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
    #[error("nothing to edit: specify --add-tag, --remove-tag, and/or --notes")]
    NoChangesSpecified,
    #[error("no PDF source recorded for {0:?}; nothing to fetch")]
    NoSourceUrl(String),
    #[error("fetch failed: {0}")]
    Fetch(String),
}
