# #1856 — Entry scorer owns the boundary decision: 0/256 to 256/256 on the live path (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T20:09:35Z
- **Updated:** 2026-10-07T20:11:21Z
- **Closed:** 2026-10-07T20:11:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1856
- **Merged:** 2026-10-07T20:11:20Z
- **Merge commit:** a38bb57c8cb643a0bef1abe59269f2330d944b9b
- **Base / Head:** main / UOR-Foundation:deepseek/entry-scorer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

The entry position stops being decided by the mixed Copy+Generate pool and gets its own **learned, exported and reloaded** scorer. `mode = "entry_scorer_fit"`:

1. fits a feature-keyed table on one label-independent panel half,
2. writes it to `entry-scorer.json`, reads it back, and hash-checks the reloaded copy,
3. evaluates the whole panel **twice through the live path** — once with the pool owning the entry decision, once with the scorer owning it — and reports the two halves separately.

The artifact is data, not floats: per feature, the training counts of the first answer token. No optimizer step, no gate, no supplied token, no serving change. When the scorer has no row for a feature it falls through to the pool, and every decision records whether it applied (`entry_scorer.applied`, `token`, `pool_choice`) so the override is never silent.

## Result — through the live evaluation path

Sealed root `deepseek/entry-ceiling/scorer-live-attempt3/` on the shared volume, 456.6 s:

| entry decision | in-sample (256) | held-out (256) |
|---|---:|---:|
| pool (mixed Copy+Generate) | 0 / 256 | 0 / 256 |
| **scorer owns the boundary** | **256 / 256** | **256 / 256** |

`artifact_sha256` `d14412529a871bc734e896b5e3ef34797bb08b439674093662e85df9c1512e8d`, `distinct_features` 12.

This is the same path that produces the scoreboard's `entry_teacher_correct`, so the number is not a side-channel: fit → export → reload → serve moves the entry decision from 0 to 256/256 on held-out rows.

## What it does and does not show

- It shows the *mechanism shape* works end-to-end: a dedicated learned decision at the empty prefix, carried through export as bounded integer data, verified on reload, and read back on the held-out half. That is the shape the earlier measurements pointed to — the pool cannot decide this position (0/512 mixed, 2/512 even with Copy removed), and an additive bounded bonus cannot beat a Copy atom at the clip (0/256 in the mixed pool two commits ago).
- It does **not** show generalisation. `distinct_features` is 12 because the panel has twelve query signatures; a held-out row is a row whose *query* was seen. 256/256 here is memorisation of the query set, and the report's own scope field says so.
- The feature remains the open problem, and it is measured: on an unseen wording, prompt-keyed lookup has no coverage, record-metadata and context-shape lookups return 0 % despite full coverage, and query×field pair tables reach only 25–34 %. What is left is a comparison computed between the query's tokens and the record's tokens rather than a lookup over either.

## Verification

`cargo check -p uor-r4-training --example geometric-frozen-map-fit` clean, `cargo test -p uor-r4-training --example geometric-frozen-map-fit` **13 passed / 0 failed** (including the new scorer round-trip and tie-break test), touched-file `rustfmt --check` clean. One defect found and fixed by the check itself: the export/reload identity test hashed the pretty-printed file against the compact in-memory serialization, so it failed on a correct export; it now compares canonical serializations. That failure is retained as `scorer-live-attempt1/`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

