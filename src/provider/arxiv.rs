use arxiv_client::{Arxiv, Search};

use super::{CandidateId, CandidateWork, Provider, ProviderError, ProviderId};

pub struct ArxivProvider {
    client: Arxiv,
}

impl ArxivProvider {
    pub fn new(contact: &str) -> Result<Self, ProviderError> {
        let client = Arxiv::builder()
            .contact(contact)
            .build()
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(ArxivProvider { client })
    }
}

impl Provider for ArxivProvider {
    fn id(&self) -> ProviderId {
        ProviderId::ArXiv
    }

    async fn search(&self, query: &str) -> Result<Vec<CandidateWork>, ProviderError> {
        let search = Search::title(query.to_string());
        let feed = self
            .client
            .search(search)
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(feed.entries.into_iter().map(CandidateWork::from).collect())
    }

    async fn get(&self, native_id: &str) -> Result<CandidateWork, ProviderError> {
        let entry = self
            .client
            .entry(native_id)
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(CandidateWork::from(entry))
    }
}

impl From<arxiv_client::Entry> for CandidateWork {
    fn from(value: arxiv_client::Entry) -> Self {
        CandidateWork {
            id: CandidateId {
                provider: ProviderId::ArXiv,
                native_id: value.id.to_string(),
            },
            title: value.title,
            authors: value.authors.into_iter().map(|author| author.name).collect(),
            publish_date: value.published.to_rfc3339(),
            doi: value.doi,
        }
    }
}
