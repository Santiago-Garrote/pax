use ::crossref::Crossref;

use super::{CandidateId, CandidateWork, Provider, ProviderError, ProviderId};

pub struct CrossrefProvider {
    client: Crossref,
}

impl CrossrefProvider {
    pub fn new() -> Result<Self, ProviderError> {
        let client = Crossref::builder()
            .build()
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(CrossrefProvider { client })
    }
}

impl Provider for CrossrefProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Crossref
    }

    async fn search(&self, query: &str) -> Result<Vec<CandidateWork>, ProviderError> {
        let works = self
            .client
            .works(query.to_string())
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(works.items.into_iter().map(CandidateWork::from).collect())
    }

    async fn get(&self, native_id: &str) -> Result<CandidateWork, ProviderError> {
        let work = self
            .client
            .work(native_id)
            .map_err(|e| ProviderError::Request(e.to_string()))?;
        Ok(CandidateWork::from(work))
    }
}

impl From<::crossref::Work> for CandidateWork {
    fn from(value: ::crossref::Work) -> Self {
        CandidateWork {
            id: CandidateId {
                provider: ProviderId::Crossref,
                native_id: value.doi.clone(),
            },
            title: value.title.into_iter().next().unwrap_or_default(),
            authors: value
                .author
                .unwrap_or_default()
                .into_iter()
                .map(|author| match author.given {
                    Some(given) => format!("{} {}", given, author.family),
                    None => author.family,
                })
                .collect(),
            publish_date: value
                .issued
                .date_parts
                .0
                .first()
                .map(|parts| {
                    parts
                        .iter()
                        .flatten()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join("-")
                })
                .unwrap_or_default(),
            doi: value.doi,
        }
    }
}
