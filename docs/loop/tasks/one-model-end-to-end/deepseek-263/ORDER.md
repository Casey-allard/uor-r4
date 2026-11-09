# ORDER: one-model-end-to-end, deepseek-263 (CPU ask and carry rank modes)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `d4154488f`.

## Goal

**Let the reply panel score ask and carry artifacts through the CPU port, including its
teacher-forced and paired-first rank modes.**

- Claude-338 made the integer engine read the ask heads and the carry. Its CPU port still refuses
  asking artifacts because the CPU Hamming port was not yet trusted there.
- The CPU port now reads both. `crates/uor-r4-cpu-train` compares its complete query, selected
  token, kept cells and Hamming choices against `IntegerStackSession` for ask-only and carried
  artifacts (`fbs.rs`, `ask_heads_at_both_hooks_match_serving_without_carry` and
  `readout_ask_carry_matches_serving_over_a_multi_row_window`).
- The panel's rank paths already use `CpuFbsForward::query_q12`, after the Hamming readout. They
  remain useful only if the refusal that guards the old CPU limitation goes away.

This unit removes that stale refusal, pins CPU-versus-integer serving on a carried ask artifact,
and exercises both rank modes on the same artifact. It does not train, change the artifact schema,
change a bar, touch Claude's branches, or use GPU compute.

## Cancel probe

1. `origin/main` contains claude-340 (`add60434a`) and deepseek-262 (`d4154488f`).
2. The CPU Hamming forward has ask-table and carry parity tests against the integer session.
3. `served-reply-panel` still refuses `--engine cpu` when any Hamming read has an ask.
4. No open PR owns this branch or the panel file.

**Result at claim:** pass.

## Design

- Remove only the panel's CPU ask/carry refusal. Keep refusing `--content-readout` on CPU,
  because that sidecar is applied by the integer product path.
- Keep `--teacher-forced` and `--paired-first` on CPU and keep `--engine integer` refusing them.
- Replace the stale test that proved CPU ignored asks with:
  - CPU carried decoding equals integer carried decoding;
  - CPU carried decoding differs from CPU ask-only decoding;
  - the ask-preserving `drop_hamming_carry_uorlut01` result is the no-carry control;
  - CPU and integer agree on both the carried and ablated artifacts.
- Exercise `teacher_forced_nearest_ranks` and `first_position_ranks` on the carried artifact and
  its ask-preserving ablation. The test must show the carry reaches ranking after multiple rows,
  not merely that the functions return.
- Use the same direct row helpers that the panel run path calls, with a built `CpuFbsForward`, so
  the test also preserves claude-340's no-per-row artifact parse.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-263/**`
- `docs/research/notebook/float-training.md`
- `docs/harness/claims.json`

## Done when

1. A carried ask artifact is accepted by `--engine cpu`, including teacher-forced and
   paired-first modes.
2. CPU and integer generation agree on the carried artifact and on its ask-preserving ablation.
3. A multi-row carry effect appears in CPU generation and ranking; removing the carry with
   deepseek-262's transformer changes those results while retaining every ask table.
4. Focused panel tests, the CPU Hamming parity tests, formatting for changed files, policy gates,
   required GitHub checks and a fresh exact-head review pass.
5. The PR merges and is verified on fresh `main`.

## Stop and ask if

Nothing waits on the owner. Do not change serving bars, model weights, training objectives,
artifact schemas, Claude branches or the integer product path. If CPU and integer serving disagree
on the carried fixture, report the first differing step and stop this unit rather than changing the
serving arithmetic.
