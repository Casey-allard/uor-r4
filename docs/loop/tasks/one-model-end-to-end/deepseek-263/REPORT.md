# REPORT: one-model-end-to-end, deepseek-263

Lab: deepseek. Base: `d4154488f`. Order: [ORDER.md](ORDER.md). This unit changes the
reply-panel instrument only. It trains nothing and uses no GPU.

## What did not work

The first push was refused by the repository policy hook because this REPORT
did not yet have its required headings. That was a documentation-format
failure; no implementation or test failed. The sections below now carry the
required report.

`main` then advanced through claude-341, which corrected the older ask test
while retaining the CPU refusal. The merge forward keeps claude-341's parity
intent and supersedes its stale refusal with this unit's carried generation and
rank controls.

The real-artifact arm did not run: claude-339 is still an open draft and has
not published an exported language-run artifact. The earlier claude-335 test
archive contains checkpoints but no `UORLUT01`. The synthetic comparison is
therefore the only completed evidence in this unit.

## What changed

`served-reply-panel` no longer refuses an asking or carried Hamming artifact on
`--engine cpu`. `CpuFbsForward` already executes both the ask table and the
readout carry; the refusal was the stale guard from claude-338, when that was
not yet true.

The remaining engine refusals are unchanged in meaning:

- `--content-readout` is still integer-only, because that sidecar is applied by
  `IntegerStackSession::serve_step`, the product path.
- `--teacher-forced 1` and `--paired-first 1` still require the CPU engine.

The module documentation now says both engines read asks and carry.

## Tests

The panel's existing ask fixture has ask tables at depths 1 and 3 and a
three-row readout carry. The new tests use deepseek-262's
`drop_hamming_carry_uorlut01` to produce the ask-preserving ablation.

1. `both_engines_read_the_ask_heads_and_carry`:
   - CPU and integer generation are equal on the carried artifact.
   - CPU and integer generation are equal on the ablated artifact.
   - Carried generation differs from ask-preserving ablated generation on both
     engines, so the readout carry is live in the free-running path.
   - CPU carried generation differs from CPU generation on the plain artifact,
     so the ask path is non-vacuous.
2. `cpu_rank_modes_use_the_carried_question`:
   - teacher-forced ranking differs between carried and ablated artifacts in a
     row after the first; this exercises the multi-row carry, not only the
     current token's ask table.
   - paired-first ranking differs between the same two artifacts.
3. `each_engine_refuses_what_it_would_score_silently_wrong` now accepts CPU
   asking artifacts while retaining the integer rank-mode and CPU content-readout
   refusals.
4. claude-340's `the_artifact_is_parsed_once_before_every_timer` still passes.

## Measurements

- Panel binary: **29 passed, 0 failed, 0 ignored**.
- CPU serving library: **483 passed, 0 failed, 5 ignored**.
- Targeted CPU-versus-integer ask parity: **1 / 1 passed**.
- Targeted CPU-versus-integer carry parity: **1 / 1 passed**.
- Carry effect: at least one teacher-forced rank after the first row changed
  between the carried artifact and the ask-preserving ablation; paired-first
  ranks also changed, and free-running generation changed on both engines.
- Real language artifact: **NOT_RUN**.

## Checks run at the head

- `cargo test --offline --locked --features cpu-parity --bin served-reply-panel`:
  **29 passed, 0 failed**.
- CPU Hamming parity against the integer session:
  - `ask_heads_at_both_hooks_match_serving_without_carry`: **1 passed**.
  - `readout_ask_carry_matches_serving_over_a_multi_row_window`: **1 passed**.
- Full `uor-r4-cpu-train` library: **483 passed, 5 ignored, 0 failed**.
- `cargo check --offline --locked --features cpu-parity --bin served-reply-panel`:
  passed.
- `cargo clippy --offline --locked --features cpu-parity --bin served-reply-panel -- -D warnings`:
  passed; dependency warnings shown by Cargo are not diagnostics in this crate.
- `rustfmt --edition 2021 --check
  docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:
  passed.
- `UOR_LAB=deepseek uor-harness loop gates --full`: **11 / 11 green**.
- `UOR_LAB=deepseek uor-harness check`: **0 errors, 327 warnings**; the
  warnings are the repository-wide advisory backlog.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: passed with the known
  claims-scope and filemap advisories.

The tests that need `UOR_PANEL_TOKENIZER` report that they skip when the
external tokenizer is absent. The ask, ablation, generation and rank tests above
do not require it.

## Skills used

No Codex skill was loaded for this unit. The work used the repository's source,
ORDER, harness rules and existing test suites.

## Still open

1. Run the same panel comparison on a real ask/carry `UORLUT01` once claude-339
   publishes one.
2. Re-run the panel with `UOR_PANEL_TOKENIZER` set for the tokenizer-dependent
   report and run-path tests.
3. The integer engine still does not host the CPU teacher-forced and
   paired-first rank modes; this unit intentionally keeps them on CPU.

## Plain summary

The reply panel no longer treats ask and carry artifacts as integer-only. Its
CPU decoder and both CPU rank modes now exercise the same ask table and
multi-row carry that the CPU forward already matched against integer serving.
The synthetic carried and ask-preserving `−C` artifacts agree across CPU and
integer generation, while the carry changes later ranking. No model was trained
or evaluated, and the real-artifact run remains open.

## Scope discipline

- No artifact schema, serving arithmetic, trainer objective, model weight or
  bar changed.
- No Claude branch, pod, lease, checkpoint or sealed artifact was touched.
- The completed deepseek/carry-ablation claim is removed from the committed
  claims store in this PR.
