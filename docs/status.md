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
- [ ] `pax add` step 3: compute the artifact's content hash (currently always `null`)
- [x] `pax list` — plain listing
- [ ] `pax list --author` / `--year` / `--tag` filters
- [x] `pax edit <key>` — tags, notes
- [ ] `pax edit <key>` — citation-key rename
- [ ] `pax edit <key>` — Identity corrections (title/authors/year/doi)
- [x] `pax remove <key>`
- [ ] `pax fetch <key>` — materialize via Nix (`nix-prefetch-url`/`nix build`), write `hash`
- [ ] `pax check` — verify declared artifacts still reproduce, without materializing
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
- [x] Artifact (source_url, hash) — shape exists; `hash` never populated yet
- [x] Local (citation_key, tags, notes)
- [x] Round-trips through `research/papers.nix` (tested)

## Reproducibility proof (docs/mvp.md §6 — the actual "done" bar)

- [ ] Full command chain run for real: `init → search → add → list → fetch → open → export bibtex`
- [ ] `git clone` a generated `research/` into a scratch dir and confirm `nix build`
      reproduces the artifact without PAX, on a second machine/environment

## Non-goals — explicitly not required (docs/mvp.md §5)

TUI, `lazypax`, plugin system, web UI, PDF annotation, full-text indexing, citation
graph, recommendations, AI summaries, auto lit-reviews, Neovim/Zotero integration,
cloud sync, embedded Nix evaluator, custom artifact store, dozens of providers.

## Critical path

`fetch` is what unlocks `hash`, `check`, `sync`, `open`, and the final reproducibility
proof — everything else on this list is either already done or a smaller, independent
gap (DBLP, display fields, filters, local search, edit corrections).
