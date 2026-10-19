# Repository renormalization — 19 October 2026 (claude)

**Owner direction (Casey, 19 Oct 2026, Claude chat).** The repository has become unreadable: about 9,300 files under `docs/`, roughly 720 KB of process text, a README written as a running journal, and mechanisms filed under several names each. Nobody can quickly say which mechanisms are alive or what is under test. The owner asked for a complete audit, reorganization and illustrated rewrite. The research is kept in full; what changes is how it is filed, so that a newcomer can understand the project and an expert can still audit every result.

## Decisions the owner made for this work

1. **Short merge freeze for the cutover.** It is announced on #2029. Compute keeps running; the freeze covers only the moved paths.
2. **Side projects get their own folder.** They live under `projects/` in this repository, one README each. Local research code moves into `projects/`; large data goes to the private HF store with checksums.
3. **Distill, then archive, in the repository.** Each mechanism and research line gets an illustrated "what we learned" page. Dated records move **byte-identical** into a top-level `archive/` with a generated catalog. **Nothing is deleted.** `uor_standards/` is archived once the build is confirmed not to read it, and the docs link upstream instead.
4. **One short charter.** Claude drafts it, and the owner approves it before it replaces the `AGENTS.md` content and the in-force D-entries. The old text is archived.
5. **D22 stays.** Mechanism statuses are ACTIVE, READY or PARKED; there is no terminal state. Every PARKED mechanism states what would reopen it.
6. **Docs publishing.** The docs are GitHub-native markdown and an mdBook site built from the same files on the Pages URL. The Studio moves to `/studio`. This amends D36 for the docs site only.
7. **Web client.** The D11 integer model runs locally in WASM, inside the existing Studio shell.
8. **Desktop client.** A Tauri app shares the web UI and runs the integer model natively, with D19 memory, offline use and M1-class laptops as the target. This work writes its spec only.
9. **Two checkpoints.** CP1 comes after the audit: charter, mechanism board, target tree and synthesis. CP2 comes before the final merge: a preview of the result.

## This unit: tooling, before anything moves

Four `xtask` tasks, all in Rust (D27). They make the move checkable and the archive tamper-evident.

| Task | What it does |
|---|---|
| `cargo run -q -p xtask -- links [--json F] [--baseline F] [--renames-from REV] [--simulate MAP.tsv]` | Checks every relative, reference, HTML and `blob/main` link in tracked markdown. `--simulate` scores a proposed move map without moving anything, and `+path` lines add virtual pointer stubs. `--renames-from` takes the map from git's `R100` renames. |
| `cargo run -q -p xtask -- docs-inventory --out F` | Lists every `docs/` file with its size, SHA-256, date, `SHA256SUMS` coverage and inbound/outbound link counts. It also records every by-string code reference to a doc path, including `include_bytes!("../../../docs/…")`. |
| `cargo run -q -p xtask -- docs-move --map MAP.tsv [--apply]` | Renames only, through `git mv`. It validates overlaps and existing destinations, and is a dry run unless `--apply` is given. |
| `cargo run -q -p xtask -- archive-verify [--json F] [--baseline F] [--base REV]` | Re-checks every `SHA256SUMS` manifest. With `--base`, it fails if any file under `archive/` that existed at REV was modified or deleted. New files are allowed. |

`.github/workflows/docs.yml` runs these on docs PRs, comparing the PR head against its base. It is not a required check.

## Measurements at `a15435627` (origin/main)

These are the baseline the cutover is judged against.

- **Links:** 4,388 markdown sources, 14,255 repository links checked, **552 already broken**, and 6,722 external links not checked.
- **Manifests:** 61 `SHA256SUMS` manifests; 744 entries `ok`, 0 mismatched, and 537 listing files kept off-repo (`missing`, as expected).
- **Code references:** 9,336 `docs/` files, of which 457 are covered by a manifest. There are 3,450 by-string code references to doc paths, and 112 `docs/labs/<dir>` records are named by code, scripts, CI or the harness.

**Move-map simulations.** Nothing was moved; each figure is the number of new broken links against the baseline.

| Variant | Entries | New broken | Note |
|---|---|---|---|
| Equal depth: `docs/X` → `archive/X`, labs records that code names stay live | 1,297 | 2,010 | 449 come from live labs records linking to moved siblings |
| Same, but loose results under `archive/lanes/` (one level deeper) | 1,297 | 2,558 | Equal depth is better |
| All `docs/labs` stays | 936 | 1,449 | History's links into `labs/` break instead |
| Move all records older than two days, plus the old curated docs | 1,314 | 1,648 | 420 of the breaks resolve to just three hub files |

The 420 breaks in the last variant all point at three hub files: `archive/integration/{DECISIONS,project-track,current-state}.md`. A small set of pointer stubs, together with snapshot copies of the referenced figures, removes most of the remaining breaks. Live documents are rewritten anyway. Archived files are never edited.

## Next

- The audit fleet is running: about 1,400 records classified, the history before 13 October reconstructed, and rules, folders, products and code inventoried.
- Then checkpoint 1: the charter draft, the mechanism board and the move map, presented to the owner.

References #2028.

## Review of the tooling (PR #2787)

Two fresh-context reviews found seven defects, fixed before merge. Every fix of a hole that could let a change through has a test, and each test was confirmed to fail without its fix:

1. Manifest entries were keyed by basename, so same-named manifests masked each other. They are now keyed by full mapped manifest path.
2. A manifest entry that disappeared was not counted. It now counts as `vanished`.
3. The workflow skipped the `--base` immutability check whenever it lacked a baseline. It now always runs on PRs.
4. Mode changes and symlink targets under `archive/` were not checked. They are now compared.
5. An unclosed code fence hid every later link. Such fences are now rescanned as ordinary text.
6. Code references written as manifest-relative `../` chains were resolved from the wrong directory. The chain-stripped path is now recorded as well.
7. Move validation allowed one entry's destination to land inside another entry's source. This is now rejected.

A further CI defect was also fixed: the base and head builds shared one cargo target directory, so the head steps ran the base binary. The base now builds in its own directory.

Known, documented limit: a manifest that is moved and edited in the same change shows up as `vanished` (a loud false failure, never a silent pass), because only byte-identical renames are mapped.
