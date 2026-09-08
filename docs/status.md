# PAX — Implementation status

A checklist of everything `docs/mvp.md` specifies, tracking what's implemented against
what's still missing. Update this alongside any change that closes or reopens an item —
it should stay accurate rather than aspirational.

## Commands

- [x] `pax init` — scaffolds `research/flake.nix` + `research/papers.nix`
- [x] `pax search <query>`
- [ ] `pax search --author "..."` / `--doi "..."` flags
- [ ] `pax search --local "query"` (local-only search, no external calls)
- [x] `pax show <provider:id>` — unresolved candidate
- [x] `pax show <citation-key>` — declared paper
- [x] `pax add <provider:id>` — declares metadata + resolved PDF source URL
- [x] `pax list` — plain listing
- [ ] `pax list --author` / `--year` / `--tag` filters
- [x] `pax edit <key>` — tags, notes
- [ ] `pax edit <key>` — citation-key rename
- [ ] `pax edit <key>` — Identity corrections (title/authors/year/doi)
- [x] `pax remove <key>`
- [x] `pax fetch <key>` — materialize via `nix store prefetch-file`, write `hash`
- [x] `pax check` — verifies every declared artifact against its recorded hash, exits
      non-zero on any mismatch/error, without writing or materializing anything
- [ ] `pax sync` — reconcile `research/` against `papers.nix`
- [x] `pax open <key>` — resolves via `nix build` on `research/flake.nix` and launches
      `$PAX_PDF_VIEWER` (default `xdg-open`); fetches automatically if not yet
      materialized (deliberately no interactive prompt — see below)
- [x] `pax export bibtex`

## Providers

- [x] OpenAlex
- [x] Crossref
- [x] Semantic Scholar
- [x] arXiv
- [ ] DBLP — no `ProviderId` variant or client module yet

## Search-result / show display fields (docs/mvp.md §2.2/§2.3/§2.9)

- [x] Title, Authors, DOI
- [x] PDF source
- [ ] Venue — no `venue` field exists anywhere in `Identity`
- [ ] Abstract
- [ ] "Whether already in library" flag on search results
- [ ] `pax search` list view (`candidates()` in `sink.rs`) shows only title/id/doi —
      missing authors/year/venue/PDF-availability entirely, unlike the single-item
      `show` view
- [ ] Nix artifact status (fetched / not fetched) on `show`

## Declarative library format (docs/mvp.md §3)

- [x] Identity (doi, title, authors, year)
- [x] Artifact (source_url, hash) — both populated (`add` resolves `source_url`, `fetch` computes `hash`)
- [x] Local (citation_key, tags, notes)
- [x] Round-trips through `research/papers.nix` (tested)

## Reproducibility proof (docs/mvp.md §6 — the actual "done" bar)

- [x] Partially confirmed: `init → add → fetch` in a scratch dir, then `nix build
      .#<key>` against the generated `research/flake.nix` reproduces the exact
      artifact from the declared hash, with no PAX involved in the build step.
      (This also caught and fixed a real bug: the flake template read `paper.url`,
      but the schema `library.rs` writes is `source_url` — `nix build` silently
      couldn't have worked for *any* declared paper until this session fixed
      `templates/flake.nix.template` and `research/flake.nix`.)
- [ ] Full command chain run for real: `init → search → add → list → fetch → open → export bibtex`
- [ ] `git clone` a generated `research/` onto a **second** checkout/machine and confirm
      the same `nix build` reproduces it there too (the scratch-dir run above proves the
      mechanism works, not cross-machine reproducibility specifically)

## Non-goals — explicitly not required (docs/mvp.md §5)

TUI, `lazypax`, plugin system, web UI, PDF annotation, full-text indexing, citation
graph, recommendations, AI summaries, auto lit-reviews, Neovim/Zotero integration,
cloud sync, embedded Nix evaluator, custom artifact store, dozens of providers.

## Critical path

`fetch`, `check`, and `open` are all done — `sync` is the only piece left on the
docs/mvp.md §4 command surface. Everything else on this list is a smaller, independent
gap (DBLP, display fields, filters, local search, edit corrections) that doesn't block
or get blocked by anything else.

## Design decisions worth remembering

- **`open` never prompts interactively.** docs/mvp.md §2.11 suggests a `Fetch now?
  [y/N]` confirmation, but this codebase already has a standing precedent against
  interactive stdin capture (`CandidateId` exists specifically so `add`/`show` never
  need to ask "which one did you mean?" — see `CLAUDE.md`'s "Reference types"
  section). A manual "go run `pax fetch` yourself" redirect has the same problem in a
  different shape — still a forced, decoupled second step. `open` instead calls
  `fetch` automatically and silently when a paper isn't materialized yet (docs/mvp.md
  §2.5 explicitly allows this as an alternative). Only a genuinely unrecoverable case
  — no `source_url` at all — is a hard error.
- **`nix build` needs `--no-link`.** Without it, every `pax open`/any future
  Nix-build-based command drops a `./result` symlink in the caller's CWD. Confirmed
  this leaking into this repo's own root during development (harmless — `result/` is
  gitignored — but worth remembering for any future code that shells out to `nix
  build`).
- **`nix store prefetch-file` vs. `nix build` are not interchangeable.**
  `prefetch-file` (what `fetch`/`check` use) always re-hits the network to discover
  the *current* hash — measured at ~11s even when the file's already in the local
  store, which is correct for freshness-checking but far too slow for `open`, which
  should be instant for an already-fetched paper. `nix build` against
  `research/flake.nix` (uses the hash already recorded in `papers.nix`) reuses the
  local store with no network call at all — measured at ~0.5s.
