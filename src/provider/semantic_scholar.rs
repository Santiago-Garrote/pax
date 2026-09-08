use ::semantic_scholar::SemanticScholar;

use super::{CandidateId, CandidateWork, Provider, ProviderError, ProviderId};

pub struct SemanticScholarProvider {
    client: SemanticScholar,
}

impl SemanticScholarProvider {
    pub fn new(api_key: Option<&str>) -> Result<Self, ProviderError> {
        let client = match api_key {
            Some(key) => SemanticScholar::with_api_key(key),
            None => SemanticScholar::new(),
        }
        .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(SemanticScholarProvider { client })
    }
}

impl Provider for SemanticScholarProvider {
    fn id(&self) -> ProviderId {
        ProviderId::SemanticScholar
    }

    async fn search(&self, query: &str) -> Result<Vec<CandidateWork>, ProviderError> {
        let result = self
            .client
            .search_papers(query)
            .send()
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(result.data.into_iter().map(CandidateWork::from).collect())
    }

    async fn get(&self, native_id: &str) -> Result<CandidateWork, ProviderError> {
        let paper = self
            .client
            .get_paper(native_id)
            .send()
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(CandidateWork::from(paper))
    }
}

impl From<::semantic_scholar::Paper> for CandidateWork {
    fn from(value: ::semantic_scholar::Paper) -> Self {
        CandidateWork {
            id: CandidateId {
                provider: ProviderId::SemanticScholar,
                native_id: value.paper_id.clone(),
            },
            title: value.title.expect("Paper does not have title!"),
            authors: value
                .authors
                .unwrap_or_default()
                .into_iter()
                .filter_map(|author| author.name)
                .collect(),
            publish_date: value.publication_date.unwrap_or_default(),
            doi: value.external_ids.as_ref().and_then(|ids| ids.doi.clone()),
        }
    }
}
