//! Owns how the CLI renders output. Every command handler goes through a
//! `Sink` instead of calling `println!` directly, so a future alternate
//! output mode (e.g. JSON, for an external adapter UI) is a new `Sink` impl
//! rather than a rewrite of every command handler.

use pax_core::{CandidateWork, ProviderId};

pub trait Sink {
    fn message(&mut self, message: &str);
    fn error(&mut self, message: &str);
    fn provider_header(&mut self, provider: ProviderId);
    fn candidates(&mut self, candidates: &[CandidateWork]);
    fn candidate(&mut self, candidate: &CandidateWork);
}

pub struct TextSink;

impl Sink for TextSink {
    fn message(&mut self, message: &str) {
        println!("{message}");
    }

    fn error(&mut self, message: &str) {
        println!("Error: {message}");
    }

    fn provider_header(&mut self, provider: ProviderId) {
        println!("{provider}:");
    }

    fn candidates(&mut self, candidates: &[CandidateWork]) {
        for work in candidates {
            println!("\t{}", work.title);
            println!("\t\t{}", work.id);
            println!("\t\t{}", work.doi);
        }
    }

    fn candidate(&mut self, candidate: &CandidateWork) {
        println!("Title:      {}", candidate.title);
        println!("Authors:    {}", candidate.authors.join(", "));
        println!("Published:  {}", candidate.publish_date);
        println!("DOI:        {}", candidate.doi);
        println!("Reference:  {}", candidate.id);
    }
}
