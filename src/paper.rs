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

/// Extracts a year from a provider's `publish_date`, whose format varies
/// (`"1986-01-01"`, an RFC3339 timestamp, or a possibly-empty Crossref
/// string) — takes the leading 4 characters and requires they parse as a
/// plausible year.
pub(crate) fn year_from_publish_date(publish_date: &str) -> Option<i32> {
    let digits: String = publish_date.chars().take(4).collect();
    if digits.len() != 4 {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_from_publish_date_parses_leading_digits() {
        assert_eq!(year_from_publish_date("1986-01-01"), Some(1986));
        assert_eq!(
            year_from_publish_date("2022-03-23T14:33:15+00:00"),
            Some(2022)
        );
    }

    #[test]
    fn year_from_publish_date_rejects_short_or_non_numeric() {
        assert_eq!(year_from_publish_date(""), None);
        assert_eq!(year_from_publish_date("198"), None);
        assert_eq!(year_from_publish_date("actor model"), None);
    }
}
