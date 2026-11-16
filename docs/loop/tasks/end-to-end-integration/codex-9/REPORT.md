# REPORT: end-to-end-integration, codex-9

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic read-index-prod` was refused because
  `crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. This unit adds production code
  and docs, no dependency and no `Cargo.lock` change.
- **It is not wired into serving.** The session still calls the O(t) scan. `CodeIndex` and
  `choose_indexed` are `pub(crate)` with `#[allow(dead_code)]`; the next unit passes the index into
  `mean_at`/`stream`/`readout` and the session state. This unit makes that plumbing safe by proving
  the offsets and the tie rule first.
- **Copy key 1 only.** A pair head (`pair[h].is_some()`) falls back to the shipped scan inside
  `choose_indexed`, so a caller that does not check the pair still gets the right answer; the
  type/ask heads are not indexed.
- **The index stores positions** (`O(t)`), so it saves work, not memory; the token vector already
  holds them. `clear` is not yet called by a reset or a restore; the wiring unit must call it.
- `cargo fmt` would rewrite the whole pre-existing `hamming.rs`; the file was reverted and only the
  formatted lines inserted, so the committed diff is purely additive. At this head
  `cargo fmt --check` does not flag the file, and standalone `rustfmt` flags only pre-existing
  lines.
- No behavior change, no model or quality claim, and no timing here (codex-8 measured 32k).

## What changed

- `crates/uor-r4-integer/src/stack/hamming.rs`:
  - `CodeIndex` in production: `new`, `clear`, `catch_up(tokens, t, code)`, `nearest(query, kept)`,
    and `built` so a session appends only what is new.
  - `CopyHeads::choose_indexed(h, tokens, index)` for copy key 1.
  - one test over 4,096 rows, compared to `choose` at every row and after a `clear`.
- No existing test, serving line, schema or dependency changed.

## Measurements

- 4,096-token fixture stream (4-code vocabulary, `kept = 16`): the indexed selection equals
  `choose` at every non-trivial row, 1..4,095 (row 0 is empty on both sides).
- A cleared index rebuilds the same answers at row 2,048.
- A pair head falls back to the scan and matches `choose` at every row of a 256-token stream.
- Hamming suite: **37 passed**.
- Exactness at 256 codes and 32k is codex-8's measurement; this unit keeps the same `nearest`.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm the_indexed_copy_head` — 1 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm hamming` — 37 passed.
- `rustfmt --check` — the added 114 lines are clean (the file's pre-existing diffs remain, below the
  new code).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in `hamming.rs`.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the production index is checked against the shipped `choose` at every row
  before any session plumbing, and the limits (pair head, memory, reset) are stated.
- `scope-guard`: one file, one test; no serving line changes; the whole-file reformat was reverted.
- `test-freeze`: adds a test and changes none; the commit carries a `TEST-CHANGE:` line.

## Still open

- The session plumbing: `HammingIndex` per read, `mean_at`/`type_mean`/`stream`/`readout` parameters,
  `clear` on reset/restore, and a long-session equality test with the index on and off.
- The type/ask heads and the pair key.
- I2 context wiring; I3/I5 deferred; I4 withdrawn.

## Plain summary

The exact index is now production code with a catch-up for a session, and the copy head can select
from it; it matches the scan at every row of a 4,096-token stream and after a reset. It is not wired
into serving yet — that is the next unit.
