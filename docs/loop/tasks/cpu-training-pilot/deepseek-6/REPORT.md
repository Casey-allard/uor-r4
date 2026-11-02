# REPORT: cpu-training-pilot, deepseek-6

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `a2d318159`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The reported Hamming off-by-one was an audit error, not a serving
  defect.** At current zero-based row `r = tokens.len() - 1`, the integer and
  CPU callers pass `r` to an exclusive `1..r` scan. Candidate `r - 1` carries
  the immediately preceding payload; row 0 has no predecessor and current row
  `r` is excluded. The float predicate independently enforces the same causal
  rule with `s >= 1 && s < r`.
- **No serving kernel was repaired.** Changing either Hamming kernel would
  have introduced an actual range defect. This unit removes the mistaken
  narrative from the served-forward contract instead.
- **The public artifacts still do not contain Hamming records.** This unit's
  direct predicate tests prove the range, while trained full-path behavior
  remains the next fixture gap.
- **The first exact-head review found avoidable formatting churn.** A
  whole-file formatter pass rewrote hundreds of unrelated lines in the integer
  and float Hamming files. Those files were restored to the base and only the
  focused tests were reapplied.

## What changed

- Added independent candidate-range tests in the integer Hamming kernel, the
  independent CPU oracle, and the float `is_candidate` predicate. At current
  row 4 over five stream tokens, each keeps payload positions `[1, 2, 3]`,
  including immediate predecessor 3 and excluding row 4.
- Corrected the exact-FBS contract wording: candidates are `1 <= s < r`, where
  `r` is the current zero-based row, not the stream length.
- Added deterministic corrected public-seed audit receipts without rewriting
  the merged PR #3560 evidence.

## Measurements

The new audit receipts both remain exact on 64 positions: zero mismatches over
16,384 Q12 coordinates, 64 tokens and 512 kept cells per seed. They were
generated with `--no-timing`; a repeat of seed 1 was byte-identical.

- Seed 1 corrected receipt: SHA-256 `6088baebaad7e36991fa4b9843a7391d4491665da64698f8b590df6b91c1e5bf`.
- Seed 2 corrected receipt: SHA-256 `fef554db08d0063ed7a7da302ad8fa035d14d159702b51f74e2dfbe972cb7b17`.
- Audit binary: SHA-256 `5b2e0f4a0f205580698be2d0b9d20e9bbf5a96e45ad129f6e0d24f13d56f3b9f`.

## Checks run at the head

- Integer: `cargo test -p uor-r4-integer --release --features
  serving-wasm,fbs-synthetic --lib
  candidates_include_the_immediate_predecessor` — 1 passed.
- CPU: `cargo test -p uor-r4-cpu-train --release
  candidates_include_the_immediate_predecessor` — 1 passed.
- Float: `cargo test -p uor-r4-float-train --release
  candidates_include_the_immediate_predecessor` — 1 passed.
- Corrected evidence SHA-256 inventory verified.
- CPU-crate formatting and all-target Clippy pass. The integer and float files
  are not formatted wholesale because their base revisions have unrelated
  formatting drift and their crate-wide Clippy reports pre-existing debt in
  files such as `integer/src/codec.rs`, `float-train/src/fbs_export.rs` and
  `in_space_long.rs`. The applied tests preserve each file's local style.

## Skills used

No repository skill was invoked. The unit used direct source tracing and
independent host controls across the three implementations.

## Still open

- The next sequence unit integrates the deterministic served-forward audit at
  artifact export/release time.
- A retained trained artifact carrying Hamming, turn memory and the point key
  is still needed for full-path public evidence.
- Training through exact dynamic quantization and discrete selection remains
  unresolved.

## Plain summary

We checked the suspected Hamming bug from end to end. It was our description
that was wrong, not the model: the implemented range is causal and includes the
immediately earlier position. Three independent tests now pin that boundary,
and corrected audit receipts replace the mistaken interpretation for future
work.
