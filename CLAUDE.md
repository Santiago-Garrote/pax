# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

PAX is a Rust project for discovering, declaring, managing, and reproducibly acquiring academic papers, using Nix as the artifact backend. The design principle: **PAX understands papers, Nix understands artifacts.**

- PAX handles: academic search, metadata resolution, source resolution, paper declarations, local organization, bibliography export.
- Nix handles: fetching, hashing, verification, caching, storage, garbage collection.

Intended workflow: `Search → Select → Declare → Fetch → Manage → Reproduce` (see `docs/mvp.md` for the full command surface and design philosophy, and `docs/status.md` for which parts of it are actually implemented — `pax fetch`, `pax sync`, and `pax check` are still planned, not yet built).

**This repository's primary artifact is `pax-core`, a library — not a CLI tool that happens to have a core module.** The `pax` binary (`src/bin/pax.rs`) is a thin, optional client of that library, gated behind the `cli` Cargo feature; a future `lazypax` TUI (or any other adapter) is meant to be just another client of the same library, per the project's own README. `docs/mvp.md` is the forward-looking spec for the full command surface — don't assume a command or flag exists just because it's documented there. **`docs/status.md` tracks exactly which parts of that spec are implemented today vs. still missing** — check it before assuming something is done or not done; keep it updated when you close or reopen an item.

## Environment

The project uses Nix + direnv for the dev shell (`flake.nix`, `.envrc`). `.envrc` does `use flake`, so `direnv allow` sets up a shell with cargo/rustc/rustfmt/clippy/rust-analyzer and the OpenSSL env vars (`PKG_CONFIG_PATH`, `OPENSSL_DIR`) needed to build the native TLS dependencies.

Secrets/config load via `dotenvy` from `.env` at startup, but only in the `pax` binary (`dotenvy::dotenv().ok()` in `src/bin/pax/main.rs`'s `main()`) — the library itself never reads the environment directly; it's passed a `pax_core::Config` instead. Recognized vars: `SEMANTIC_SCHOLAR_API_KEY` (Semantic Scholar falls back to unauthenticated requests if unset) and `ARXIV_CONTACT` (arXiv's contact-email etiquette header, omitted if unset).

## Commands

```bash
cargo build                        # builds the library + the pax binary (cli is a default feature)
cargo build --no-default-features  # builds pax-core alone — proves the CLI deps (clap, dotenvy, tokio) aren't pulled in
cargo run --bin pax -- <args>      # e.g. cargo run --bin pax -- search "actor model"
cargo test                         # unit tests (library.rs round-trip, provider CandidateId parsing)
cargo test <substring>             # run a single test, e.g. cargo test candidate_id_rejects_bare_doi
cargo fmt                          # format (rustfmt is in the flake devshell)
cargo clippy                       # lint
```

## Architecture

```
src/
  lib.rs        # pax-core's public API: re-exports + search_all()
  provider/     # Provider trait, ProviderId, CandidateId, CandidateWork, ProviderError
    mod.rs      #   + the four provider implementations, one file each
  paper.rs      # Paper, Identity, Artifact, Local, PaperRef (citation key)
  library.rs    # Library::load()/save() against research/papers.nix
  nix.rs        # init_library(): scaffolds research/flake.nix + research/papers.nix from templates/
  error.rs      # PaxError, unifying ProviderError + io/parse errors
  bin/
    pax.rs      # clap CLI — argument parsing, dispatch, output formatting only
```

**Reference types — the load-bearing design decision in this codebase:** `add`/`show` and `search` operate on fundamentally different address spaces, so they use different types:
- **`CandidateId`** (`provider/mod.rs`) — a fully-qualified `provider:native_id` reference (e.g. `openalex:W2741809807`, `arxiv:2301.01234`) to a single, unresolved search hit. Its `FromStr` impl is deliberately strict: it rejects a bare DOI, a title, or free text (see `provider::tests` for the exact rejected/accepted cases). This makes "search returned two results, which one did you mean?" structurally impossible at the `add`/`show` argument boundary — disambiguation is `search`'s job (it already returns a list); `add`/`show` only ever accept a reference that is already unambiguous.
- **`PaperRef`** (`paper.rs`) — a citation key (e.g. `turing1936`) addressing the closed set of papers already declared in the local library. Used by `remove`/`edit` (implemented) and `open`/`fetch` (not yet — see `docs/status.md`).

**`Provider` trait** (`provider/mod.rs`) — `fn id()`, `async fn search(query)`, `async fn get(native_id)`. One implementation per provider module (`openalex.rs`, `crossref.rs`, `semantic_scholar.rs`, `arxiv.rs`), each also providing `From<ProviderNativeType> for CandidateWork`. `get()` is implemented for all four today (`OpenAlexClient::get_work`, `Crossref::work`, `SemanticScholar::get_paper`, `Arxiv::entry`).

**`pax::search_all(query)`** fans out to all four providers concurrently-in-sequence (awaited one after another, not via `join!`) and returns `HashMap<ProviderId, Result<Vec<CandidateWork>, ProviderError>>` — a failing provider doesn't fail the whole search, and the *caller* (the CLI) decides how to display an error, since the library itself never prints.

**`Library`** (`library.rs`) — `research/papers.nix` is Nix syntax, but it's a schema PAX itself invented and fully controls (a flat attrset of `citation_key = { ...fields... };` records; `research/flake.nix` only ever reads `url`/`hash` off each entry generically via `builtins.mapAttrs`, so extra fields are harmless to it). `Library::load`/`save` are a small hand-rolled tokenizer/parser and writer scoped to exactly that fixed shape — deliberately not a general Nix-language parser (no `rnix-parser` or similar dependency). `research/papers.nix` is the single source of truth; nothing else shadows it. `add`/`edit`/`remove` all mutate a `Library` today; round-trip correctness is proven by a unit test in `library.rs`.

**`pax init`** (`nix.rs::init_library`) materializes `research/flake.nix` + an empty `research/papers.nix` from embedded templates (`include_str!` from `templates/flake.nix.template` / `templates/papers.nix.template`). The `research/` directory is gitignored; the top-level `research/flake.nix` / `research/papers.nix` checked into *this* repo are a working example of that generated structure, not the templates themselves.

## Known gaps

See `docs/status.md` for the full, kept-current checklist of implemented vs. missing
pieces against `docs/mvp.md`'s target command surface. Briefly: `init`, `search`,
`show`, `add`, `list`, `remove`, `edit`, and `export bibtex` exist; `fetch`, `sync`,
`check`, `open`, and the DBLP provider don't yet — each is meant to be a small,
well-typed addition on top of `Provider`/`CandidateId`/`Library`/`PaperRef`, rather
than needing another architectural change first.
