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

## Cutover tools (second unit)

Four more `xtask` tasks. They make the move itself mechanical:

| Task | What it does |
|---|---|
| `docs-relink --map MAP.tsv [--apply]` | Rewrites only the link targets in **live** markdown files that point at moved paths. It never touches archived, SHA256SUMS-covered, pinned FIT-sheet or dated lab-record files. Run before the move. |
| `docs-rewrite-refs --map MAP.tsv [--exclude P] [--apply]` | Rewrites `docs/…` path literals in live code, including `include_bytes!` `../` chains, to `archive/…`. A literal it cannot place exactly is reported `UNREWRITTEN`. The cutover tools' own sources are excluded, because their tests use synthetic paths. |
| `docs-catalog [--check]` | Renders `archive/CATALOG.md` and one page per research line under `archive/catalog/` from `archive/catalog.jsonl`. |
| `docs-stubs --list FILE [--apply]` | Writes pointer stubs, or copies figures, at the `archive/` positions where archived records link to files that stay live. |

**Rehearsal.** I rehearsed the full cutover in a throwaway clone of this branch: relink, rewrite references, move, add stubs, then build the catalog. The results:
- 1,007 links rewritten in 75 live documents, with 0 left unmatched.
- 8,733 files moved, every one an R100 rename.
- 104 stubs and copies written.
- 1,173 records catalogued.
- All 405 pre-existing `archive/` files unchanged, and 0 manifest regressions.

The move leaves 365 links newly broken, and none of them are in live content:
- 318 are inside archived records, mostly absolute `blob/main` URLs that cannot be edited.
- 44 are in SHA-pinned FIT sheets.
- 3 are in records dated 18–19 October, which resolve again once those records are archived at equal depth.

The rehearsal also caught problems in the cutover inputs, all fixed before any real run:
- The tools' own tests had been rewritten.
- The harness's tests name six real records and rely on `docs/labs/` semantics. Those records now stay live, and the harness and claim-wording code are excluded from rewriting until the path registry lands.

**Review and compile footprint.**
- A fresh-context review found one policy gap. `docs-rewrite-refs` would have edited code inside three live dated records, including a verifier whose `git show {rev}:docs/…` reads are correct only at the old path. Records, pinned prep sheets and SHA256SUMS-covered files are now never edited, and each such literal is reported instead. A test pins the rule.
- One more record, `rung0-pilot-release-2026-10-14`, stays live because a live verifier reads it from the working tree.
- I compared the rehearsed tree with unmodified `main`, package by package, for every crate whose code the cutover rewrites: `cargo check -p … --all-targets` and `--keep-going` full error sets. They are identical. For example, `uor-r4-core`, `uor-r4-integer` and `uor-r4-router` show the same 1,156 pre-existing messages on both sides, so the cutover adds no compile error.
- The discrete crate checks clean with and without CUDA (stub `nvcc`), and its 198 library tests pass.
- The harness keeps only its one failure, which `main` already has.

## Cutover (third unit): the move

The owner approved the move map at checkpoint 1. The run used the rehearsed script `cutover/cutover.sh` on a fresh branch from `97e0eec43`, tagged `pre-renorm-2026-10-19` (freeze notice: #2029). It ran in four commits:
1. **Live links and code paths.** 1,003 link targets in 75 live documents and 380 path literals in live code now point at `archive/`. Three literals inside protected records were deliberately not edited: two `git show {rev}:` reads that are correct at their revision, and one comment.
2. **The move.** 8,710 files, all `R100` renames: `docs/history`, `docs/evidence`, `docs/r4_*_evidence`, every `docs/integration` file except the four hubs, the loose result files in `docs/`, and dated lab records older than 2026-10-18. The map is `cutover/move-map.tsv`, which is also the old→new path table. Seven records stay live because live code or the harness names them; see `PINNED_LABS` in `cutover/make-map.sh`.
3. **Pointer stubs.** 106 stubs and figure snapshots, listed in `cutover/stubs.txt`, so links inside archived records to the four hubs, the harness, the mechanism briefs and the figures keep resolving.
4. **The catalog.** `archive/catalog.jsonl` holds 1,173 records, 360 of them verified by a full read. `archive/CATALOG.md` and the 20 per-line pages are generated from it by `cargo run -q -p xtask -- docs-catalog`.

**Gates at the branch head**
- The move commit contains only `R100` renames.
- `archive-verify` against the base: all 405 pre-existing `archive/` files unchanged, 0 manifest regressions (744 ok).
- `links` against the base: 0 new broken links in live content. 367 permanent breaks remain, all inside protected files and listed in `cutover/permanent-link-breaks.txt`:
  - 318 inside archived records, mostly absolute `blob/main` URLs that cannot be edited;
  - 42 in SHA-pinned FIT sheets;
  - 7 in records dated 18–19 October, which resolve once those records are archived at equal depth.
- Compile and test results are in the PR evidence.

**Running an archived script.** Check out `pre-renorm-2026-10-19` to run any archived script with its original `docs/…` paths.

## Charter adoption (fourth unit)

The owner (Casey) adopted the charter at checkpoint 1 on 19 October 2026, with these eight answers:

| # | Answer |
|---|---|
| Q1 | D24 covers research compute; on-device D11 serving is the product. |
| Q2 | A FIT pre-registration is needed only for GPU training or evaluation and for runs that milestone claims rest on. |
| Q3 | One typed release per sheet. |
| Q4 | One notebook per research line. |
| Q5 | Owner plus one PM seat; the council is retired. |
| Q6 | The M1 bar becomes "beat both count tables on two seeds, with tokens/s". |
| Q7 | Lean harness. |
| Q8 | The legacy crates are repaired, not archived. |

Process: the draft was revised with the answers, then reviewed in a fresh context against all 258 inventoried rules. That review found three major and six minor issues, all fixed, including restoring the matmul ban's "anywhere" scope and D30's pod nvcc builds.

This unit lands:
- `CHARTER.md`;
- a 67-line `AGENTS.md`;
- `docs/integration/DECISIONS.md` as an index that keeps all 44 original headings, so every `#dNN` anchor still resolves, and adds D42;
- `docs/rules/` (a plain-language guide, the D0–D42 → charter map, and a change log);
- the first research notebooks, `docs/research/notebook/`;
- a rewritten `CONTRIBUTING.md`;
- "superseded" banners on `session-goal.md`, `protocol.md`, `.kimi-code`, `.codex-lab` and the `.agents` skill.

The previous `AGENTS.md`, `DECISIONS.md` and `CONTRIBUTING.md` are archived byte-identical under `archive/governance/` and `archive/integration/DECISIONS-2026-10-19.md`. The harness gate changes for the notebook cadence land in their own PR.

## Byte-bound sources the cutover touched (fifth unit, 20 Oct 2026)

`docs-rewrite-refs` rewrote `docs/…` path strings in live code. Some of those files are themselves hashed by code that compares the hash against a sealed result, so changing a comment changes a bound identity. A census of every `include_str!`/`include_bytes!` target the cutover rewrote, with each path resolved exactly, found two cases that still matter:

- **The #846 compositional-planning result.** `crates/uor-r4-graph-certify/tests/compositional_planning_certification_846.rs` compares content IDs of six sources with `archive/compositional_planning_certification_846_result.json`. The cutover changed one comment or notes line in each, so `committed_result_binds_the_audit_inputs_and_negative_verdict` failed (#2793 surfaced it once certify built again).
  - **Fix:** this unit restores those six files byte-for-byte to `pre-renorm-2026-10-19`:
    - `compositional_reasoning.json`
    - `compositional_planning.rs`
    - `semantic_transitions.rs`
    - `plan_sections.rs`
    - `plan.rs`
    - `compositional_planning_measurement_843.rs`
  - **Verified on #2793's head:** 3 of 3 tests pass with the restore, and the binding test fails without it.
  - **Stale paths kept on purpose:** their comments again name `docs/…` spec paths that now live under `archive/`. The bytes are bound, so the stale paths stay. Never rewrite these six files.
- **The discrete driver identity.** `uor_r4_discrete::training::driver_source_sha256()` hashes 19 sources. The cutover rewrote `include_*!` paths to config files that moved: 1 in `controller.rs`, 2 in `completion_baseline.rs`, 1 in `redirect_admission.rs`. These edits were required, because the old paths no longer exist.
  - **The identity changed:** from `0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d` (at `fd409fcdf`, #2782, unchanged through `97e0eec43`) to `b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24` (from `d7b39d6a9` on). No compiled behaviour changed: the included file contents are identical.
  - **For the A8 dose:** the admission-preserving sheet binds its controller and finite qualification to `fd409fcdf`, and the consumer verifier record binds `0f8ce406…`. So the dose must be built at `fd409fcdf`, as bound. A binary built from later main reports `b13d4920…` and the qualification gates will refuse it, correctly.

The other rewritten files that hash themselves are historical targets that do not build today, or that report a hash without comparing it:
- the core `*_973` tests;
- the `joint-*` training examples;
- `chat-grade`'s `milestone_world_v2`;
- `uor-r4-model-source`;
- the core `addressed-lexical-bridge` and `realtext-sparse-recall` binaries.

Their old bytes are at the tag `pre-renorm-2026-10-19`.
