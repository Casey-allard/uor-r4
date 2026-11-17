# ORDER: cpu-training-pilot, deepseek-254 (grouped carry-hold checkpoint audit)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b0961dd91`.

## Goal

Make the CPU carry-hold audit reproduce the token-group readings Claude-334's
run records, and make several arm/seed checkpoints auditable in one strict,
deterministic pass. This prepares the CPU-side reading of Claude-334's upcoming
`B2-aqH` and `B2-aH` seals without waiting on, modifying or spending compute on
Claude's active run.

1. Add the two-class task's grouped hold-gain readings to the checkpoint parser:
   the 16 question tokens, four header tokens, 512 value tokens and the mean of
   all other tokens.
2. Add each ask depth's `|w_y|` mean to the hold summary so the independent CPU
   reading is comparable with Claude-334's `hamming_type_weight_mean_abs`.
3. Extend `cpu-carry-hold-audit` with a manifest mode that:
   - binds every checkpoint by model and Hamming SHA-256;
   - requires relative, contained checkpoint paths and unique arm, seed, path
     and checkpoint-hash identities;
   - audits arms in parallel with a bounded worker count while preserving
     manifest order;
   - writes one deterministic report only to a new path.
4. Add synthetic controls for the grouped statistics, per-depth weights,
   malformed manifest identities, path traversal, worker bounds, deterministic
   repeats and output replacement.
5. If Claude-334 seals real held checkpoints before this unit closes, audit
   them; otherwise mark the real arm run `NOT_RUN`.

This is a read-only checkpoint instrument. It does not prove B2, carry,
capability, export or serving.

## Claim

`uor-harness loop claim cpu-training-pilot` refuses this CPU scope on the
known umbrella-scope false overlap with Claude's open one-model order. The
files below overlap none of PR #3886. Use a hand-written ORDER and let local
policy prove the changed-path scope.

## Cancel probe

`cpu-carry-hold-audit` currently reports whole-vocabulary gain statistics and
one depth-2 type-weight mean, while Claude-334 records gain by question,
header, value and rest groups plus one type-weight mean per depth. It accepts
only one checkpoint, so a two-seed/two-arm audit requires hand collection.

## Files in scope

- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-254/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. The single-checkpoint schema reports question/header/value/rest gain and
   every present ask depth's type-weight mean absolute value.
2. Batch mode binds hash, row count and safe checkpoint path, rejects duplicate
   identities, preserves source order across parallel workers and emits a
   byte-identical repeat.
3. Targeted tests and all full loop gates pass at the exact head.
4. Real Claude-334 held checkpoints are audited if sealed; otherwise the REPORT
   marks them `NOT_RUN` and gives the exact manifest-mode next command.
5. A fresh exact-head adversarial review approves the PR before it merges.

## Stop and ask if

Do not modify Claude-owned float-training files, PR #3886, serving or export
schemas, GPU state, pods or another lab's work. Do not interpret a moved gain
as carry, B2 or language evidence.
