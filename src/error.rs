use thiserror::Error;

use crate::provider::ProviderError;

#[derive(Debug, Error)]
pub enum PaxError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse library: {0}")]
    LibraryParse(String),
}
