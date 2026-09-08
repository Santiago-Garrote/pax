//! Scaffolds a new `research/` library: the `flake.nix` Nix reads to build
//! declared papers, and an initial empty `papers.nix` (see [`crate::library`]
//! for the schema PAX writes into it as papers are declared).

use std::fs::{create_dir, write};
use std::io::Error;
use std::path::{Path, PathBuf};

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
