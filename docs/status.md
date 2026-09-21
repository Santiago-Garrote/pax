# PAX — Implementation status

A checklist of everything `docs/mvp.md` specifies, tracking what's implemented against
what's still missing. Update this alongside any change that closes or reopens an item —
it should stay accurate rather than aspirational.

## Commands

- [x] `pax init` — scaffolds `research/flake.nix` + `research/papers.nix`
- [x] `pax search <query>`
- [x] `pax search --author "..."` — real field-scoped search on arXiv/OpenAlex;
      Crossref/Semantic Scholar fall back to plain full-text search (see below)
- [x] `pax search --doi "..."` — direct resolution on OpenAlex/Crossref/Semantic
      Scholar (all three confirmed to resolve the same paper for the same DOI);
      arXiv reports "not supported" (no DOI-based lookup in that crate at all)
- [x] `pax search --local "query"` (local-only search, no external calls; matches
      title/authors/doi/year/venue/tags/citation-key)
- [x] `pax show <provider:id>` — unresolved candidate
- [x] `pax show <citation-key>` — declared paper
- [x] `pax add <provider:id>` — declares metadata + resolved PDF source URL
- [x] `pax list` — plain listing
- [x] `pax list --author` / `--year` / `--tag` filters — combine with AND
- [x] `pax edit <key>` — tags, notes
- [x] `pax edit <key>` — citation-key rename (`--rename`), validated against
      `Library`'s own identifier grammar and checked for collisions before
      anything is written — an unvalidated rename would corrupt `papers.nix`
      (citation keys are unquoted Nix identifiers, not strings)
- [x] `pax edit <key>` — Identity corrections (`--title`/`--author`/`--year`/`--doi`);
      `--author` replaces the whole list, not incremental like tags; no way to
      clear `year`/`doi` back to null (out of scope — see `docs/status.md` design
      notes)
- [x] `pax remove <key>`
- [x] `pax fetch <key>` — materialize via `nix store prefetch-file`, write `hash`
- [x] `pax check` — verifies every declared artifact against its recorded hash, exits
      non-zero on any mismatch/error, without writing or materializing anything
- [x] `pax sync` — batch `fetch`: materializes every declared paper without a hash
      yet, one paper's failure doesn't stop the rest, exits non-zero on any failure
- [x] `pax open <key>` — resolves via `nix build` on `research/flake.nix` and launches
      `$PAX_PDF_VIEWER` (default `xdg-open`); fetches automatically if not yet
      materialized (deliberately no interactive prompt — see below)
- [x] `pax export bibtex`

## Providers

- [x] OpenAlex
- [x] Crossref
- [x] Semantic Scholar
- [x] arXiv
- **Skipped, deliberately — DBLP** (see "Design decisions worth remembering" below
  for why this isn't just "not implemented yet")

## Search-result / show display fields (docs/mvp.md §2.2/§2.3/§2.9)

- [x] Title, Authors, DOI
- [x] PDF source
- [x] Venue — `Identity.venue`, persisted through `papers.nix`; populated from each
      provider (OpenAlex `primary_location.source.display_name`, Crossref
      `container_title`, Semantic Scholar `venue`, arXiv `journal_ref` — often
      `None` for pure preprints)
- [x] Abstract — candidate-only (`CandidateWork.abstract_text`), not persisted to
      `Identity`/`papers.nix` (docs/mvp.md §3's declarative format never lists it,
      and it can be arbitrarily long)
- [x] "Whether already in library" flag on search results — `pax_core::known_dois` +
      `normalize_doi` (handles OpenAlex's full-URL DOI vs. the other three
      providers' bare-DOI format), shown as `[in library]` in `search`'s list view
- [x] `pax search` list view now shows authors/year/venue/PDF-availability
      (✓/✗) per result, matching `show`'s richer fields
- [x] Nix artifact status (`Fetched`/`Not fetched`) on `show` — reads
      `paper.artifact.hash` directly, no `nix` invocation (that's `check`/`open`'s job)

## Declarative library format (docs/mvp.md §3)

- [x] Identity (doi, title, authors, year)
- [x] Artifact (source_url, hash) — both populated (`add` resolves `source_url`, `fetch` computes `hash`)
- [x] Local (citation_key, tags, notes)
- [x] Round-trips through `research/papers.nix` (tested)

## Reproducibility proof (docs/mvp.md §6 — the actual "done" bar)

- [x] Full command chain run for real, in a scratch dir treated as its own git repo
      (matching how a real user's research library would actually be set up):
      `init → search → add → list → fetch → open → export bibtex`, all succeeded.
- [x] `git clone` onto a second checkout, `nix build` there with zero PAX code
      involved, reproduced the exact same store path
      (`/nix/store/izsx5favdpayifwp9lrdbh4pmlinr5sn-vaswani2017`) purely from the
      git-tracked `flake.nix`/`papers.nix`. **Caveat, stated plainly rather than
      glossed over:** this was a second checkout, not a second physical machine —
      same local Nix store both times, so cache reuse from earlier builds in this
      session can't be fully ruled out. A true cross-machine/cross-store run is
      still open if that distinction matters later.
- **Real Nix behavior surfaced by this test, not a PAX bug:** a flake's files must
  be tracked by git before `nix build` will evaluate them at all — an untracked
  `research/flake.nix` fails with "not tracked by Git" (with the exact `git add`
  command suggested). This means a real user's workflow needs `git add research/`
  (staging is enough, commit not required) before the first `pax open`/any
  nix-build-based command, in a library that's already a git repo. Worth a
  `CLAUDE.md`/README callout for anyone documenting the user-facing workflow.

## Non-goals — explicitly not required (docs/mvp.md §5)

Embedded Nix evaluator, custom artifact store — these would cross PAX's core
division of responsibility with Nix. Everything else once listed here was
either built (`lazypax`) or moved to the
[org idea backlog](https://github.com/pax-project/.github/blob/main/docs/ideas/) —
see `docs/mvp.md §5` for the full reasoning.

## Critical path

**The full docs/mvp.md §4 command surface is now implemented, and so is docs/mvp.md
§6's success criteria** — all 12 commands exist, the full command chain
(`init → search → add → list → fetch → open → export bibtex`) runs end-to-end for
real, and a `git clone`d second checkout reproduces the same artifact via `nix
build` with zero PAX involvement (same-machine caveat noted above). DBLP — the
one remaining item from docs/mvp.md's provider list — is **deliberately skipped**,
not merely unimplemented; see below for why. Nothing else is outstanding.

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
- **Providers don't agree on DOI format.** OpenAlex returns a full URL
  (`"https://doi.org/10.xxxx"`); Crossref, Semantic Scholar, and arXiv all return a
  bare DOI (`"10.xxxx"`). Any code comparing two DOIs for equality (e.g. `known_dois`'
  "already in library" check) needs `pax_core::normalize_doi` first, or it'll silently
  miss real matches depending on which provider each one came from.
- **OpenAlex needs a `doi:` prefix for id-based lookup, not a bare DOI.**
  `get_work`'s `id` param is inserted raw into the URL path (`/works/{id}`); a bare
  DOI contains `/` (e.g. `10.1145/358141.358147`), which reads as extra path
  segments and 404s. Confirmed against the live API — `get_by_doi` prefixes with
  `doi:` before calling `get()`.
- **The `crossref` crate's `FieldQuery` doesn't work against the real API.**
  `FieldQuery::author(name)` sends a bare `author=...` param; Crossref's actual API
  requires `query.author=...` and rejects the bare form as a validation failure.
  Confirmed against the live API on 0.2.2, the latest available — a crate defect,
  not a usage mistake. `search_by_author` doesn't override the trait default for
  Crossref (falls back to plain search) for this reason, same as Semantic Scholar.
- **Pre-existing, unrelated to any work this session: Crossref search sometimes
  fails to deserialize.** `pax search "Carl Hewitt"` (plain query, no filters) hits
  `invalid serde: missing field 'family'`; other queries hit `missing field
  'title'`. Confirmed this predates the `search`/`list`-filters work — the
  `crossref` crate's response types appear to assume fields the live API doesn't
  always send (e.g. an institutional/anonymous contributor with no `family` name).
  Not fixed here — out of scope for this task, noted for whoever picks up
  Crossref-related work next.
- **Citation keys are unquoted Nix identifiers, not strings.** `write_entry`
  (`library.rs`) writes `{key} = { ... };` with the key bare, not `"{key}"`. Any
  code that lets a citation key be user-supplied (currently only `edit --rename`)
  must validate it against `library::is_valid_citation_key` first, or it can write
  a `papers.nix` that `Library::load` can no longer parse back — a self-inflicted
  corruption with no warning until the next read.
- **DBLP is skipped deliberately, not just "not implemented yet."** Confirmed live
  (not assumed) that both `dblp.org` and the `dblp.uni-trier.de` mirror now serve
  every API request — search and per-record alike, regardless of User-Agent —
  from behind **Anubis**, a proof-of-work anti-bot wall, instead of real data:
  ```
  {"rules":{"algorithm":"fast","difficulty":16}, ...}
  ```
  So this was never "one more provider module like the other four" — there's no
  crate to wrap (unlike OpenAlex/Crossref/S2/arXiv), and unlike a normal from-scratch
  HTTP+JSON integration, a plain client gets the challenge page back on every
  request, not data. The real options, none of them cheap:
  - Implement an Anubis proof-of-work solver so `pax` can pass the challenge before
    every request — genuine extra engineering, and fragile by design: Anubis exists
    specifically to keep evolving against this kind of automated bypass, so it can
    silently break again on any DBLP-side update.
  - Switch to DBLP's bulk XML data dump (the whole database, several GB, updated
    periodically) and build a local search index instead of live per-query search —
    a fundamentally different architecture from the other four providers, not a
    same-shape addition.
  - Or what this project has chosen: skip it. DBLP was never required by
    docs/mvp.md §6's actual success criteria (that's about the core workflow, not
    provider count), and deliberately automating around a bot-detection wall is a
    step beyond normal API integration worth not taking without a specific reason
    to. Revisit if DBLP's public access story changes, or if a specific paper only
    findable there is actually needed.
