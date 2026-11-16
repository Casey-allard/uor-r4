# REPORT: end-to-end-integration, codex-8

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic read-index-32k` was refused because the item is
  still claimed by this lab's codex-7 worktree (PR #4023, frozen for review). The claim is this
  lab's own; the unit adds one test and docs, no dependency and no `Cargo.lock` change.
- **The wall times are indicative, not a benchmark.** One scan and one indexed query in a debug
  test build; the numbers move with the machine and the build profile. The asserted result is the
  exact equality and the distance-count ratio; the timings are printed for context only.
- **The count is exact because the vocabulary is saturated.** At 32k the stream uses all 256 codes;
  a stream with fewer distinct codes is proportionally cheaper, and a stream with every code at one
  distance still merges at most `kept` positions per code, bounded by the vocabulary, not by `t`.
- **The index still stores positions** (`O(t)`), so it saves work, not memory; the token vector
  already holds them.
- **Single-code keys only.** Copy key 2 (pair) still scans; nothing is wired into the session.
- No model, quality or serving-time claim beyond the printed single-query context.

## What changed

- `crates/uor-r4-integer/src/stack/hamming.rs`: one test in `code_index_tests`,
  `the_index_at_32k_examines_codes_not_positions`, building the index for positions `1..t-1`,
  comparing to the shipped scan, counting distance computations, and printing both wall times.
- No production line, existing test, schema or dependency changed.

## Measurements

- 32,768-token stream, 256 codes, `kept = 16`, row `t = 32,767`.
- **256 distance computations vs 32,766 positions — 128.0x fewer.**
- One scan `780.791 us`, one indexed query `12.473 us` in the debug test build (indicative).
- The indexed selection equals the scan exactly.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm the_index_at_32k` — `1` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm hamming` — 34 passed.
- `rustfmt --check` — the added test's lines are clean (the file's pre-existing code still has its
  old diffs, below line 1666).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in the added lines.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: replaced the extrapolation with a measurement at the context the cost claim
  names, and kept the timing out of the assertions.
- `scope-guard`: one test and docs; no production line.
- `test-freeze`: adds a test to an existing module and changes none; the commit carries a
  `TEST-CHANGE:` line.

## Still open

- The wiring: per-head `CodeIndex` state in the session, updated on push, rebuilt on restore,
  behind `ReadOut`; that is I1b's remaining half.
- The pair-key head and the type/ask adapters.
- I2 context wiring; I3/I5 deferred; I4 withdrawn.

## Plain summary

At the 32k context the cost claim is about, the exact index examines **256 codes instead of 32,766
positions (128x fewer)**, matches the scan exactly, and completes one query in about 1/60th of the
scan's wall time in the debug build. It is still not wired into serving.
