# #615 — feat(#531): parallel observe-text — teacher-force articles across a worker pool

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-11T00:53:03Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-11T01:11:50Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/615
- **Merged:** 2026-08-11T01:11:50Z
- **Merge commit:** 06fdc17a8557cb02ac481f1fe11ed377888ea193
- **Base / Head:** main / UOR-Foundation:r5/parallel-observe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #531 — parallelize the corpus-scale observe

The observe was single-threaded: one teacher instance stepped one article at a time, so a ~2M-record run pinned one core (~28h measured on an M1). `observe-text` is teacher-forced and the sampled token is discarded, so every recorded field is a deterministic function of `(article tokens, model)` — articles are mutually independent. This parallelizes them, with **byte-identical output**.

- Extracted a pure `produce_article_records` (teacher-force one article → its ordered records) and a `commit_article` (append + checkpoint). The single-threaded path routes through them unchanged — it threads `checkpoint.rng` exactly as before, so its shards and committed checkpoint are byte-for-byte identical.
- `observe_text_corpus` now takes an oracle pool `&mut [Box<dyn TeacherOracle + Send>]`: it gathers a batch of up to `pool.len()` not-yet-committed articles, teacher-forces them in parallel (one `std::thread::scope` worker per oracle, each with a local throwaway rng — sound because no recorded field depends on it), then commits the batch on the main thread in ascending ordinal. The writer appends in call order, so shard bytes match a single-oracle pass exactly; resume/checkpoint semantics are unchanged.
- `graph-cli observe-text` gains `--workers N` (default 1): builds N teacher instances (each holds its own weights, so RAM scales with N) and passes the pool through.

### Determinism guard
A new test observes the same fixture corpus with **1 vs 3 workers** and asserts the merged stream, its κ, every shard file, and `stories.jsonl` are byte-identical.

Verified: default + `--all-features` build, full `cargo test --workspace --no-run` (incl `tests/bdd.rs`), `clippy -D warnings`, `fmt`, and `check-model` / `audit-deferral` / `audit-limits`.

Refs #531.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

