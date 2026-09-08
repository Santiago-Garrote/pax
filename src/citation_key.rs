//! Generates a citation key (`Local.citation_key`, later addressed by
//! `PaperRef`) for a newly declared paper: `surname` + `year` (e.g.
//! `agha1986`), disambiguated on collision by a word from the title rather
//! than an opaque letter — `agha1986actors` vs `agha1986interactions` tells
//! you which paper is which without a lookup; a letter suffix is kept only
//! as the last resort if even that collides too.

use crate::paper::year_from_publish_date;
use crate::provider::CandidateWork;

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "on", "in", "for", "and", "to", "with", "from", "by",
];

pub fn generate(work: &CandidateWork, existing_keys: &[&str]) -> String {
    let base = base_key(work);
    if !existing_keys.contains(&base.as_str()) {
        return base;
    }

    let with_title_word = format!("{base}{}", title_word(&work.title));
    if !existing_keys.contains(&with_title_word.as_str()) {
        return with_title_word;
    }

    let mut suffix = b'a';
    loop {
        let candidate = format!("{with_title_word}{}", suffix as char);
        if !existing_keys.contains(&candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn base_key(work: &CandidateWork) -> String {
    let surname = work
        .authors
        .first()
        .and_then(|name| name.split_whitespace().last())
        .map(slugify)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "anon".to_string());
    match year_from_publish_date(&work.publish_date) {
        Some(year) => format!("{surname}{year}"),
        None => surname,
    }
}

fn title_word(title: &str) -> String {
    title
        .split_whitespace()
        .map(slugify)
        .find(|word| !word.is_empty() && !STOPWORDS.contains(&word.as_str()))
        .or_else(|| title.split_whitespace().next().map(slugify))
        .unwrap_or_default()
}

fn slugify(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{CandidateId, ProviderId};

    fn work(authors: &[&str], publish_date: &str, title: &str) -> CandidateWork {
        CandidateWork {
            id: CandidateId {
                provider: ProviderId::OpenAlex,
                native_id: "W1".to_string(),
            },
            title: title.to_string(),
            authors: authors.iter().map(|a| a.to_string()).collect(),
            publish_date: publish_date.to_string(),
            doi: None,
        }
    }

    #[test]
    fn no_collision_uses_surname_and_year() {
        let w = work(&["Gul Agha"], "1986-01-01", "Actors: A Model");
        assert_eq!(generate(&w, &[]), "agha1986");
    }

    #[test]
    fn collision_disambiguates_with_title_word() {
        let w1 = work(&["Gul Agha"], "1986-01-01", "Actors: A Model");
        let w2 = work(&["Gul Agha"], "1986-06-01", "Interactions in Systems");
        let base = generate(&w1, &[]);
        assert_eq!(base, "agha1986");
        assert_eq!(generate(&w2, &[base.as_str()]), "agha1986interactions");
    }

    #[test]
    fn double_collision_falls_back_to_letter_suffix() {
        let w = work(&["Gul Agha"], "1986-01-01", "Actors in Distributed Systems");
        let existing = ["agha1986", "agha1986actors"];
        assert_eq!(generate(&w, &existing), "agha1986actorsa");
    }

    #[test]
    fn missing_authors_falls_back_to_anon() {
        let w = work(&[], "2020-01-01", "A paper with no listed authors");
        assert_eq!(generate(&w, &[]), "anon2020");
    }

    #[test]
    fn missing_year_uses_surname_alone() {
        let w = work(&["Gul Agha"], "", "Actors");
        assert_eq!(generate(&w, &[]), "agha");
    }
}
