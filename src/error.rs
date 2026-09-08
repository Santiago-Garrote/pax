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
}
