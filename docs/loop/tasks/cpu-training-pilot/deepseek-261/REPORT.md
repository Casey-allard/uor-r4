# REPORT: cpu-training-pilot, deepseek-261

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The handoff proposed `deepseek-254`, but open PR #3889 already owns that
  number for unrelated carry-hold checkpoint work. This unit was renumbered to
  the next free `deepseek-261`; its files do not overlap PR #3889.
- The first draft push was blocked because the ORDER landed before its REPORT.
  The REPORT skeleton was added before the draft was published; no product
  code existed at that point.
- The first compile exposed a borrow mismatch in the CPU carry bound:
  `reach.min(t)` saw `reach` as `&usize`. The code now uses
  `(*reach).min(t)`; the focused test covers the corrected bound.
- A workspace-wide `cargo fmt --all` formatted unrelated clean files. Those
  formatter-only changes were reverted by exact path, and final formatting was
  restricted to the two ordered Rust files.
- Strict `clippy -D warnings` over every package target is blocked by ten
  pre-existing lints in unrelated CPU files. The final library/test run with
  only those known lint classes allowed passes with no warning in this unit.
- The initial ORDER cited roadmap issue #3472, which is the width/data study.
  The actual CPU training item is #3506; the ORDER and PR body now use #3506.
- This unit establishes CPU/integer execution parity only. It does not train a
  model, qualify a GPU bar, or show that an ask-selected value produces a
  useful reply.

## What changed

- The independent CPU Hamming type heads now retain the key rotation and the
  optional exported `AskImage`.
- When ask data exists, type queries are formed exactly as the integer session
  does: `(table + ask)[x_t]` at Q28, plus each readout-carry gain times the
  corresponding row, then the key rotation and sign code. There is no
  intermediate rounding.
- The stream uses ask heads without a carry; the readout uses the same rule
  with its declared carry. Artifact validation restricts carry data to the
  readout, which this maps one-to-one.
- Served math and Hamming tracing share the same ask-query selector. CPU FBS
  trace calls now receive the writer table needed for the question sum.
- Tests add a direct four-lag Q28 carry oracle and CPU/integer parity over two
  hooks and a 16-row window. Every step compares the full Q12 query, selected
  token and kept cells; every trace compares depth, query, key and kept
  positions. Plain-versus-ask and ask-versus-carry controls must both change a
  later result.
- Existing no-ask and mixed copy-key paths remain green.
- Size before this REPORT was 327 Rust lines added and 54 removed. The ORDER
  gave no line estimate, so no ratio applies. `loop pr-check` supplies the
  final by-class size.

## Measurements

The focused `ask` library filter passed **28 tests**, with **1 ignored**.
The two new parity tests ran 16 rows through both Hamming hooks:

- ask heads replaced the hook-point queries in the plain-versus-ask control;
- the readout carry changed at least one later row in the ask-versus-carry
  control;
- every intervening CPU step and Hamming trace matched `IntegerStackSession`.

The direct Q28 arithmetic test covers carry lags 1 through 4 and checks that
the no-carry arm contains only the current asked row.

The final full CPU library suite on the AMD Ryzen 9 9950X passed **483 tests**
with **5 ignored** in **48.98 s**. The timed process wall was **51.80 s** at
**1965%** CPU and **1,014,568 KiB** peak RSS.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib ask`:
  28 passed, 1 ignored.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  hamming::tests::asked_point_carries_every_lag_at_q28_without_rounding --
  --exact`: 1 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 483 passed,
  5 ignored in 48.98 s; timed process wall 51.80 s.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- Scoped library/test Clippy with the six pre-existing lint classes allowed:
  passed. Strict all-target Clippy remains blocked by pre-existing unrelated
  lints.
- `rustfmt --edition 2024 --check` on `fbs.rs` and `hamming.rs`: passed.
- `git diff --check`: passed.
- `uor-harness loop pr-check --local`: passed; scope, banned presence,
  commit order, REPORT, owner paths, rule sources, rendered lock and read path
  were green, with only the existing FILEMAP advisory.
- `uor-harness loop gates --full`: all 11 gates green in 18.76 s.
- `uor-harness check`: 371 rules, 0 errors and 336 pre-existing warnings in
  96.45 s. The branch-name warnings include stale cached PR state outside this
  unit.

## Skills used

No system-provided skill was used. The repository's CPU-research and loop
delivery instructions were followed directly.

## Still open

- Run the completed CPU training/evaluation drivers against the real
  Claude-336 ask/carry checkpoints when they are published; this unit changes
  the oracle they use but produces no checkpoint reading.
- Treat any future CPU/model result as D84 development evidence. GPU remains
  the qualification authority under the charter.

## Plain summary

The CPU copy of the model previously ignored the exported question table. When
it audited or trained an asking model, it asked from the ordinary hidden point
instead. This change makes the CPU copy perform the same integer question sum
and short readout carry as the real serving engine. Tests put identical tokens
through both engines and require the same query, token, kept cells and trace at
every step. That removes a silent mismatch from future CPU experiments; it does
not yet say whether the model gives good answers.
