# ORDER: cpu-training-pilot, deepseek-143 (local continuation-horizon regressor)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `7dfd96644`.

## Goal

Test a continuation objective rather than another terminal-class density gate.
For every generated modulo-256 content state, predict how many reference
tokens remain from the local signature trajectory and stop only when the
predicted remaining horizon is exactly zero.

Deepseek-142's class-balanced density gate recalled only 19/84 terminal rows
and its shuffled class control recalled the same count. This unit uses the
ordinal horizon (`reference_length - generated_offset`) as the training
signal, keeps near-terminal positive horizons, and uses an exact-offset
neighbourhood. It does not retune density, prototypes, writer admission or
content targets.

This is D84 CPU development research. It does not change integer serving,
`UORLUT01`, CUDA, float training or Claude-owned files.

## Fixed construction

Use the deepseek-137 98/103 split, memory and positive code readout. Generate
every training reply with that arm and retain each decision's complete
2,048-bit signature, generated offset and signed horizon.

- Candidate neighbourhood sizes are 1, 2, 4 and 8.
- Neighbours must have the same generated offset; there is no offset radius.
- Rank by Hamming distance and then signature bytes. Exclude the query row
  during fitting.
- Predict the lower median horizon of the k nearest rows.
- Emit BOS only when the prediction is exactly zero, before the unchanged
  content token. Negative predictions mean the state is past its reference
  and continue without a stop.
- Select k leave-one-row-out by maximum terminal recall, then fewer false
  stops, lower horizon mean absolute error, and finally smaller k.
- The control rotates horizon values within each exact-offset group,
  preserving every group's size, offsets and horizon multiset while changing
  their signature association.

All ungated choices must remain the code arm's content tokens.

## Cancel probe

The tested alternatives were terminal/nonterminal prototypes (deepseek-132,
139), class density with missing-neighbour penalties (deepseek-142),
teacher-forced stop coordinates (deepseek-131), writer-cell identity
(deepseek-120), radius-only matching (deepseek-118), and post-reference
content rows (deepseek-138). None trained an ordinal remaining-horizon
objective over generated rows with within-offset horizon permutation.

## Preflight

Run `--select-only` on both seeds with `--stop-horizon-neighbors 8`.
Proceed only if each seed has:

- exactly 84 zero-horizon rows, 1,484 nonterminal rows and 1,568 total rows;
- true leave-one-out terminal recall at least 26/84;
- true false stops at or below the 1% nonterminal limit;
- true horizon MAE strictly below the within-offset shuffled control;
- different true and shuffled gate hashes and changed per-row predictions;
- a selected k in `{1,2,4,8}`; and
- all 1,568 generated training rows retained.

The preflight and its resource reading are preserved even when it stops.

## Arms and bar

Run no-gate code, true horizon and within-offset horizon-permuted arms over
complete 16-token replies. Both seeds must pass:

- true training recall at least 26/84 with false stops at or below 1%;
- true horizon MAE below shuffled;
- at least 9/103 replies stop at BOS, at least 8 no later than the reference
  length, and at least 8 exactly match the reference and stop at BOS;
- true BOS completion beats code and shuffled by the prompt-level rule
  `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- true top-1 and matching-prefix depth are not below code;
- every fired decision is BOS and every ungated token equals code;
- on zero-horizon training rows, true prediction beats shuffled by
  `candidate_only >= max(2, ceil(2 * sqrt(control_only)))`;
- true and shuffled gate hashes and per-row predictions differ; and
- a one-worker seed-1 repeat matches every non-timing report field and
  generated JSON artifact from the 32-worker seed-1 run.

Also report every k's LOO recall, false stops and MAE; selected k; full
horizon error; BOS exact/shorter/longer stops; top-1/depth; openings; distinct
and repeated replies; workers; wall, RSS and hashes.

## Instrument gates

- Source and all five public inputs match.
- Horizons equal normalized reference length minus generated offset, including
  negative values after the reference.
- Exact-offset selection, deterministic ties, lower-median prediction and
  leave-one-out fitting are tested.
- The control preserves each offset group's horizon multiset and changes the
  effective prediction mapping.
- Fired decisions are BOS and ungated decisions equal code.
- The 1-worker and 32-worker generated evidence matches.
- Full CPU release tests, scoped Clippy, rustfmt and evidence manifests pass.

## Stop rules

Stop on source/input/split drift, missing classes, a preflight bar failure,
vacuous permutation, ungated token mismatch, worker nondeterminism, resource
breach, or any need to change integer serving, CUDA, float training or
Claude-owned source.

## Decision consequence

A pass makes local continuation-horizon prediction a supported CPU
termination candidate and justifies artifact-bound serving integration. A
fail closes this exact-offset lower-median configuration and requires a
different trajectory state, decode objective or content mechanism. Neither
result alone establishes useful chat.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-continuation-horizon-2026-11-04.md`
- `docs/labs/cpu-training-pilot-continuation-horizon-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-143/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The crate and driver expose the horizon arm without replacing earlier arms.
2. Tests cover horizon derivation, exact-offset selection, deterministic ties,
   lower median, LOO fitting, within-offset permutation and BOS-only firing.
3. FIT and fresh GO precede the preflight and full run.
4. Evidence preserves the result and classifies the mechanism.
5. Exact-head review, protected merge and fresh-main verification complete.
