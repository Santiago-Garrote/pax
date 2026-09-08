//! Scaffolds a new `research/` library: the `flake.nix` Nix reads to build
//! declared papers, and an initial empty `papers.nix` (see [`crate::library`]
//! for the schema PAX writes into it as papers are declared). Also owns the
//! one place `pax-core` shells out to the `nix` binary — to fetch and hash an
//! artifact, per the project's "PAX understands papers, Nix understands
//! artifacts" split (see `CLAUDE.md`).

use std::fs::{create_dir, write};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::error::PaxError;

const FLAKE_TEMPLATE: &str = include_str!("../templates/flake.nix.template");
const PAPERS_TEMPLATE: &str = include_str!("../templates/papers.nix.template");

/// Where `research/papers.nix` lives under a library root, shared by
/// `init_library` (which writes its initial contents) and `add_candidate`
/// (which reads/writes it as papers are declared).
pub fn papers_path(root: &Path) -> PathBuf {
    root.join("research").join("papers.nix")
}

pub fn init_library(root: &Path) -> Result<(), Error> {
    let research_dir = root.join("research");
    create_dir(&research_dir)?;
    write(research_dir.join("flake.nix"), FLAKE_TEMPLATE)?;
    write(papers_path(root), PAPERS_TEMPLATE)?;
    Ok(())
}

#[derive(Deserialize)]
struct PrefetchOutput {
    hash: String,
}

/// Parses `nix store prefetch-file --json`'s stdout, extracting just the SRI
/// hash (e.g. `"sha256-..."`) — already in the format `research/flake.nix`'s
/// `pkgs.fetchurl { hash = ...; }` expects, so no reformatting is needed.
/// `storePath`, the output's other field, is intentionally discarded: it's
/// fully determined by `url`+`hash`, and Nix's own store is the cache, so
/// nothing needs to persist it in `papers.nix`.
fn parse_prefetch_output(stdout: &[u8]) -> Result<String, PaxError> {
    let parsed: PrefetchOutput =
        serde_json::from_slice(stdout).map_err(|e| PaxError::Fetch(e.to_string()))?;
    Ok(parsed.hash)
}

/// Downloads `url` into the Nix store and returns its content hash, by
/// shelling out to `nix store prefetch-file`. This is the only fetching or
/// hashing pax-core does — both are otherwise Nix's job.
pub fn prefetch_file(url: &str) -> Result<String, PaxError> {
    let output = Command::new("nix")
        .args(["store", "prefetch-file", "--json", url])
        .output()
        .map_err(|e| PaxError::Fetch(e.to_string()))?;
    if !output.status.success() {
        return Err(PaxError::Fetch(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    parse_prefetch_output(&output.stdout)
}

/// Resolves a declared paper's materialized artifact to its Nix store path,
/// by building `research/flake.nix`'s `<citation_key>` package output.
/// `--no-link` is required — without it, `nix build` drops a `./result`
/// symlink in the caller's CWD, which would otherwise leak into a user's
/// project on every `pax open`. Fast (no network) when the hash is already
/// in the local store, since the derivation's hash is fixed upfront; the
/// caller must ensure the paper actually has a hash before calling this, or
/// flake evaluation fails on the `null`.
pub fn build_package(root: &Path, citation_key: &str) -> Result<PathBuf, PaxError> {
    let research_dir = root.join("research");
    let flake_ref = format!("{}#{citation_key}", research_dir.display());
    let output = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", &flake_ref])
        .output()
        .map_err(|e| PaxError::Fetch(e.to_string()))?;
    if !output.status.success() {
        return Err(PaxError::Fetch(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prefetch_output_extracts_hash() {
        let stdout = br#"{"hash":"sha256-vfqmjYmE8NwCvqylJ7dvIH2ZtmbTHR2nKO4HKBgt9pc=","storePath":"/nix/store/ym0pjqsz6b2nxwbr4xp3zpln46b71sd0-1706.03762v7"}"#;
        assert_eq!(
            parse_prefetch_output(stdout).unwrap(),
            "sha256-vfqmjYmE8NwCvqylJ7dvIH2ZtmbTHR2nKO4HKBgt9pc="
        );
    }

    #[test]
    fn parse_prefetch_output_rejects_malformed_json() {
        assert!(parse_prefetch_output(b"not json").is_err());
    }
}
