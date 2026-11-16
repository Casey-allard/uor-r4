# REPORT: end-to-end-integration, codex-6

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic read-index-probe` was refused because the item is
  still claimed by this lab's codex-3 worktree (PR #4015, queued behind the org runner pool). The
  claim is this lab's own; the unit adds a test module inside `hamming.rs` and docs, no dependency
  and no `Cargo.lock` change.
- **`cargo fmt` rewrote the whole pre-existing `hamming.rs`** (it is not format-clean on `main`):
  639 insertions and 105 deletions for a 128-line module. The file was reverted and only the
  formatted module appended, so the diff is **+128 lines**. The module itself is rustfmt-clean; the
  rest of the file is untouched and still not format-clean, as before.
- **This is a proof, not the wiring.** The session still calls the O(t) `nearest`; nothing is
  indexed in the serving path yet. The next unit must add per-head index state, update it on token
  push, and rebuild it on restore.
- **The index covers single-code keys only** — copy key 1, the type heads and the ask heads. Copy
  key 2 joins two token codes, so its bucket count can reach `vocab²`; it is not modeled here and
  still needs the scan.
- **It saves work, not memory.** Positions are still stored once per head (`O(t)`), which the token
  vector already provides; the index adds the code buckets (at most one per distinct code).
- **The 8.6× is at a 4,096-token stream** with a 256-code vocabulary. The ratio grows with context
  because the distance count saturates at the distinct-code count: at 32k the vocabulary cap gives
  ~128× (32768/256); using the measured 239.4 average it is ~137×. Both are extrapolations from the
  measured shape, not a 32k measurement.
- No model, quality or serving-time claim.

## What changed

- `crates/uor-r4-integer/src/stack/hamming.rs` gains a `#[cfg(test)] mod code_index_tests`:
  - `CodeIndex`: `code -> ascending positions`, built by inserting the key code of each eligible
    position as the stream advances;
  - `nearest(query, kept)`: distance to each distinct code, codes walked in ascending distance, and
    within one distance the first `kept` positions of each bucket are merged and sorted by position,
    giving exactly `(distance, position)` ascending;
  - a crafted tie test and a 4,096-token/256-code exactness test against the shipped `nearest`.
- No production line, existing test, schema or dependency changed.

## Measurements

- Exactness: the indexed selection equals `nearest` at **255 sampled rows** of the 4,096-token
  stream, with `kept = 16`; the crafted tie case yields `[(1, 0), (3, 0), (2, 1), (4, 1)]`, the
  read's `(distance, position)` order.
- Work: **239.4 distance computations** per query on average, against **2,047 eligible positions**
  — **8.6× fewer**. The count is the number of distinct codes, which is bounded by the vocabulary
  and does not grow with the stream.
- All 33 Hamming-read tests pass; 0.01 s for the module.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm code_index` — `2` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm hamming` — `33` passed.
- `rustfmt --check` on the appended module — clean (the file's pre-existing code is unchanged and
  still not format-clean).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in `hamming.rs`.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the index is checked against the shipped function on every sampled row, not
  against its own idea of the tie rule; the 8.6× is reported at its measured context and the 32k
  figure is labelled an extrapolation.
- `scope-guard`: the full-file reformat was reverted; the diff is the module only.
- `test-freeze`: the commit adds a test module and changes no existing test; the `TEST-CHANGE:`
  line says so.

## Still open

- **Wiring**: per-head `CodeIndex` state in the session, updated on push, rebuilt on restore, passed
  into `mean`/`ask_mean`, with the forced-replay path still on the scan. This is the unit that turns
  the 8.6× into a serving measurement.
- The pair-key head (copy key 2): either a two-code bucket key or leave it on the scan.
- A 32k-session measurement once wired, to replace the extrapolation.
- The `l = 2, 3` ladder, I2 context wiring, and I3 e-means remain queued.

## Plain summary

The Hamming read's selection can be indexed by key code instead of scanning every earlier position,
and the index reproduces the shipped selection exactly, including ties. On a 4,096-token stream it
examines 239 distinct codes instead of 2,047 positions — 8.6× fewer distance computations — and the
count is capped by the vocabulary, so the ratio grows with context. It is not wired into serving yet.
