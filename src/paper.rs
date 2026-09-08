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

/// Applies incremental tag add/remove and an optional notes overwrite to a
/// paper's local metadata. Adding an already-present tag is a no-op (no
/// duplicates); removing an absent tag is a no-op (no error) — both safe to
/// call again.
pub(crate) fn apply_edits(
    local: &mut Local,
    add_tags: &[String],
    remove_tags: &[String],
    notes: Option<&str>,
) {
    for tag in add_tags {
        if !local.tags.contains(tag) {
            local.tags.push(tag.clone());
        }
    }
    local.tags.retain(|t| !remove_tags.contains(t));
    if let Some(notes) = notes {
        local.notes = Some(notes.to_string());
    }
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

    #[test]
    fn apply_edits_add_is_idempotent() {
        let mut local = Local::default();
        apply_edits(&mut local, &["a".to_string()], &[], None);
        apply_edits(&mut local, &["a".to_string()], &[], None);
        assert_eq!(local.tags, vec!["a".to_string()]);
    }

    #[test]
    fn apply_edits_remove_absent_tag_is_a_no_op() {
        let mut local = Local {
            tags: vec!["a".to_string()],
            ..Default::default()
        };
        apply_edits(&mut local, &[], &["b".to_string()], None);
        assert_eq!(local.tags, vec!["a".to_string()]);
    }

    #[test]
    fn apply_edits_add_and_remove_combine() {
        let mut local = Local {
            tags: vec!["a".to_string()],
            ..Default::default()
        };
        apply_edits(&mut local, &["b".to_string()], &["a".to_string()], None);
        assert_eq!(local.tags, vec!["b".to_string()]);
    }

    #[test]
    fn apply_edits_overwrites_notes() {
        let mut local = Local {
            notes: Some("old".to_string()),
            ..Default::default()
        };
        apply_edits(&mut local, &[], &[], Some("new"));
        assert_eq!(local.notes, Some("new".to_string()));
    }
}
