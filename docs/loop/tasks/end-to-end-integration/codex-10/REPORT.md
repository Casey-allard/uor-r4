# REPORT: end-to-end-integration, codex-10

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic read-index-wire` was refused because
  `crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. This unit changes the stack
  and its tests, adds no dependency and does not change `Cargo.lock`.
- **Only the `Mean` rule on copy key 1 is indexed for the copy heads.** The latest and longest
  rules select a different way and keep the scan; a pair head falls back to the scan; the forced
  replay keeps the scan.
- **The type/ask heads are indexed** (their keys are single-code); the ask heads share the type
  heads' keys, so one index serves both.
- **The equality test is 96 steps on a 128-context synthetic trunk**, not 32k. It proves the index
  and the scan serve the same answer; the 32k work count is codex-8's measurement, not repeated.
  Its Hamming-trace leg is computed by `record`, which scans for both sessions, so the meaningful
  equality is `step_native`; the full position order is covered by the unit test
  `the_indexed_copy_head_matches_the_scan_on_a_long_stream`.
- **No timing here.** The switch exists for equality, not for a benchmark.
- **The pair key is not indexed.** Copy key 2 falls back to the scan in `choose_indexed`.
- The `HammingIndex` is `pub` only so the `pub` read methods can name it; the module is private, so
  it is crate-internal in practice.
- No model or quality claim.

## What changed

- `crates/uor-r4-integer/src/stack/hamming.rs`:
  - `HammingIndex` (one `CodeIndex` per copy and type head), `ServedHamming::indexes`, `clear`.
  - `CopyHeads::mean_at` and `TypeHeads::mean_by` take an optional index; `TypeHeads::choose_indexed`.
  - `stream`/`readout` take the read indexes and pass the read's own to the means.
- `crates/uor-r4-integer/src/stack/session.rs`:
  - the session field, built from the model's Hamming reads; `clear` on `reset` and `restore_state`;
    `set_hamming_index(bool)`; the four read call sites pass the indexes (or an empty slice).
- `crates/uor-r4-integer/src/stack/tests.rs`: the call sites gain `None` / `&mut []`.
- `crates/uor-r4-integer/tests/hamming_index_parity.rs` (new): the on/off session equality.

## Measurements

- Library: **674 passed**, 1 ignored.
- On/off parity: 96 steps on a read-bearing synthetic trunk, tokens and Hamming trace identical at
  every step, and again after `reset`.
- Hamming tests run inside the library suite; no existing assertion changed except the added
  parameters.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 674 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  hamming_index_parity` — 1 passed.
- `rustfmt --check` — the added lines are clean (the files keep their pre-existing diffs).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib`
  — clean, no warning in the changed files. `--lib --tests` fails on a pre-existing deny in
  `tests/openai_loopback.rs:1658`, outside this diff.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the exchange is proven identical on the served path, with a switch that
  forces the old path, before any cost claim is made.
- `scope-guard`: the pair key, the context and the schema are untouched; only the read plumbing
  changed.
- `test-freeze`: adds a test file and call-site parameters; no existing assertion is weakened.

## Still open

- The pair-key index (or leaving it on the scan by design).
- A 32k session measurement once a 32k-context artifact exists.
- I2 context wiring; I3/I5 deferred; I4 withdrawn.

## Plain summary

The served Hamming read now selects from the exact per-head key index by default, and a session
switch runs the scan instead. Over 96 steps and after a reset, the two serve the same tokens and the
same choices. The pair key and the non-mean rules still scan.
