# PAX — MVP CLI Features

## 1. Purpose

PAX is a Rust CLI for discovering, declaring, managing, and reproducibly acquiring academic papers using Nix as the artifact backend.

The MVP should focus on one core workflow:

```text
Search → Select → Declare → Fetch → Manage → Reproduce
```

PAX should **not** attempt to replace Nix. Nix is responsible for reproducible artifact acquisition and storage; PAX provides the academic-paper domain layer.

---

## 2. Core MVP Features

### 2.1 Library initialization

Create a new PAX research library.

```bash
pax init
```

Creates the minimum required structure:

```text
research/
├── flake.nix
├── papers.nix
```

The library should be Git-friendly and reproducible.

---

### 2.2 Paper discovery

Search external academic metadata sources.

```bash
pax search "actor model"
pax search --author "Carl Hewitt"
pax search --doi "10.1145/..."
```

Search results should expose:

* Title
* Authors
* Publication year
* Venue
* DOI
* Abstract, when available
* Available PDF sources
* Source/provider
* Whether the paper is already in the library

#### MVP providers

The MVP should initially support only a small number of reliable providers:
- OpenAlex — primary general-purpose academic discovery and metadata.
- Crossref — DOI resolution and canonical bibliographic metadata.
- Semantic Scholar — academic discovery, citations, references, and related papers.
- arXiv — preprints and papers, particularly for computer science, mathematics, and physics.
- DBLP — specialized computer-science bibliography and metadata.

These providers should be implemented behind a common PAX discovery interface so that adding providers later does not require changing the CLI or paper model.

---

### 2.3 Paper inspection

Inspect a search result before adding it.

```bash
pax show <result>
```

Example:

```text
Title:       On Computable Numbers...
Authors:     Alan M. Turing
Year:        1936
DOI:         10.1112/plms/s2-42.1.230

PDF sources:
  ✓ Institutional repository
  ✓ Archive
  ✗ Publisher

Status:
  Not in library
```

---

### 2.4 Add paper to the library

Add a discovered paper to the declarative library.

```bash
pax add <result>
```

The operation should:

1. Resolve canonical metadata.
2. Identify an accessible PDF source.
3. Determine the artifact's content hash.
4. Generate the paper declaration.
5. Add it to the library specification.
6. Make it available to Nix for reproducible acquisition.

The MVP should distinguish between **declaring a paper** and **materializing its PDF**.

---

### 2.5 Lazy artifact acquisition

Adding a paper should not necessarily require immediately materializing the PDF.

```bash
pax add 42
```

can result in:

```text
✓ Metadata resolved
✓ Paper declared
✓ PDF source recorded

PDF: not downloaded
```

Materialization should be explicit:

```bash
pax fetch <paper>
```

or:

```bash
pax open <paper>
```

if opening a non-materialized paper implicitly requires fetching it.

---

### 2.6 Nix integration

PAX should use standard Nix mechanisms rather than implementing its own artifact store.

Responsibilities of PAX:

```text
paper identity
metadata
source resolution
declarations
hashes
library management
```

Responsibilities of Nix:

```text
fetching
hash verification
content-addressed storage
caching
reproducibility
garbage collection
```

The CLI should expose:

```bash
pax fetch <paper>
pax check
pax sync
```

#### `pax fetch`

Materialize declared papers through Nix.

#### `pax check`

Verify that declared artifacts remain reproducible.

#### `pax sync`

Reconcile the local library with its declarative specification.

---

### 2.7 Local library listing

List papers already declared in the library.

```bash
pax list
```

Support basic filtering:

```bash
pax list --author Turing
pax list --year 1936
pax list --tag computability
```

---

### 2.8 Local search

Search the local metadata without querying external providers.

```bash
pax search --local "computability"
```

Search fields should include:

* Title
* Authors
* DOI
* Year
* Venue
* Tags
* Citation key

---

### 2.9 Paper metadata

Inspect a paper already in the library.

```bash
pax show turing1936
```

Display:

```text
Title
Authors
Year
Venue
DOI
Citation key
Tags
PDF status
PDF source
Nix artifact status
```

---

### 2.10 Metadata editing

Allow modification of local metadata.

```bash
pax edit turing1936
```

At minimum:

* Tags
* Citation key
* Notes
* Metadata corrections

The original external metadata should not be silently overwritten without user intent.

---

### 2.11 Open papers

Open a materialized paper using the user's configured PDF viewer.

```bash
pax open turing1936
```

If the artifact is not materialized:

```text
Paper is not available locally.

Fetch now? [y/N]
```

This reinforces the lazy workflow.

---

### 2.12 Remove papers

Remove a paper from the declarative library.

```bash
pax remove turing1936
```

This should remove the declaration and metadata, but should not directly manipulate the Nix store.

Nix remains responsible for determining when unused artifacts are garbage-collected.

---

### 2.13 BibTeX export

Export the local library as BibTeX.

```bash
pax export bibtex
```

Also support:

```bash
pax export bibtex > bibliography.bib
```

The generated citation keys should be stable and deterministic.

---

## 3. Declarative Library Format

The exact format can evolve, but the MVP should maintain a clear separation between:

### Paper identity

```text
DOI
title
authors
year
```

### Artifact identity

```text
source URL
content hash
```

### Local organization

```text
citation key
tags
notes
```

A paper should therefore conceptually look like:

```text
Paper
├── Identity
│   ├── DOI
│   ├── title
│   ├── authors
│   └── year
│
├── Artifact
│   ├── source
│   └── hash
│
└── Local metadata
    ├── citation key
    ├── tags
    └── notes
```

This distinction is important for reproducibility.

---

## 4. CLI Command Set

The MVP should aim for the following command surface:

```text
pax init

pax search <query>
pax show <paper-or-result>

pax add <result>
pax remove <paper>

pax list
pax edit <paper>

pax fetch <paper>
pax sync
pax check

pax open <paper>

pax export bibtex
```

Optional aliases can be added later.

---

## 5. MVP Non-Goals

The following should explicitly remain outside the MVP:

* TUI
* `lazypax`
* Plugin system
* Web interface
* PDF annotation
* Full-text PDF indexing
* Citation graph
* Related-paper recommendations
* AI summaries
* Automatic literature reviews
* Neovim integration
* Zotero synchronization
* Cloud synchronization
* Embedded Nix evaluator
* Custom package/artifact store
* Dozens of metadata providers

The MVP should prove the core architecture before expanding the feature set.

---

## 6. MVP Success Criteria

PAX should be considered successful when this workflow works end-to-end:

```bash
pax init

pax search "actor model"

pax add <result>

pax list

pax fetch <paper>

pax open <paper>

pax export bibtex
```

And the resulting library should be reproducible:

```bash
git clone <research-library>

cd <research-library>

nix build
```

A second machine should be able to reconstruct the same declared paper artifacts without depending on PAX's external discovery process.

---

## 8. MVP Philosophy

The CLI should remain:

* Local-first
* Reproducible
* Scriptable
* Unix-friendly
* Declarative where possible
* Lazy about materializing artifacts
* Independent of a graphical interface
* Useful without the future TUI

The future `lazypax` interface should consume the same PAX functionality rather than becoming a separate implementation.

