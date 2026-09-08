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
- [ ] `pax open <key>` — launch configured PDF viewer, prompt-to-fetch if not materialized
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

`fetch` and `check` are done, both built on `nix::prefetch_file` — `sync` and `open` are
next. Everything else on this list is a smaller, independent gap (DBLP, display fields,
filters, local search, edit corrections) that doesn't block or get blocked by anything
else.
