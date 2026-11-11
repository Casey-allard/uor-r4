# ORDER: one-model-end-to-end, deepseek-273 (aligned-corpus GPU smoke)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `7a450a88f`.

Scorer/control source: `4add144ebdc9087e47fe3fa830fb21116278644e`; execution
pin and script hashes: `launch-pins.env`.

## Goal

Train the current geometric FBS/Hamming model on the aligned scenario TRAIN for
one bounded GPU smoke, export the checkpoint through the real integer artifact
path, and score the same 1,024-row development and Fresh panels through
`cpu-scenario-grade`, beside the same-evaluator previous-token count table and
the existing arm-r artifact.

This first aligned-corpus GPU probe asks whether the current model turns a fact
stated in context into a held-out answer. It is not a CPU proxy or capability
claim.

## Cancel probe

Before any pod is rented:

1. Seal the whole corpus and base artifact in HF and read them back.
2. Check every data, label, manifest, tokenizer and trainer-`ft/dev` hash.
3. Score the count-table and arm-r controls on the sealed Fresh rows.
4. Confirm scenario TRAIN is `--train-split` while tokenizer and `ft/dev`
   satisfy `--data-root`.
5. Confirm trainer, snapshot, bound exporter and scorer exist at the pushed
   head.

On the pod, a 64-step smoke must end with a checkpoint before the 1,024-step
run starts. Smoke readings are never used as model results.

## Files in scope

- `docs/harness/prep/one-model-end-to-end-scenario-gpu-smoke-2026-11-11.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-273/**`
- `docs/research/notebook/data-tokenizer.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `conformance/stack/fbs-deepseek-scenario-smoke.toml`
- `conformance/generated/stack-contracts.json`
- `docs/model/STACK-CONTRACT.md`
- `crates/uor-r4-cpu-train/src/scenario_grade.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-scenario-grade.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`

A defect may add the smallest needed trainer, exporter or readout file only
after this ORDER is updated. Ignored jobs, inputs, checkpoints, snapshots,
artifacts and reports stay under `local/deepseek/scenario-gpu-smoke-2026-11-11/`
and HF.

Claude's `claude/e8-address` and `claude/place-read` branches and worktrees are
out of scope.

## Done when

1. The pre-registration has a fresh non-author FIT and passed pre-run review.
2. The pod trains at the exact pushed head with `--features cuda --locked`,
   `--train-as-served 1`, and no dense transformer, softmax, or matmul.
3. The smoke produces a checkpoint; the bounded run produces
   `models/step-001024`.
4. `fbs-snapshot` produces a snapshot and `fbs-export` produces an integer
   artifact that parses and carries the new stack-contract and tokenizer
   identities.
5. `cpu-scenario-grade` scores all 1,024 development and 1,024 Fresh cases from
   that artifact, with asked-value, distractor, abstention, exact-reference,
   stop, and distinct-reply counts by archetype. Each report also carries the
   previous-token count-table control on the identical selected cases.
6. Arm-r and count-table controls are read before the result is classified.
7. The REPORT applies the sheet's rows in order and records negatives first,
   exact commands, hashes, timings, cost, and what remains unverified.
8. Results are sealed/read back, the pod is destroyed, and all delivery checks
   pass.

## Stop and ask if

- any corpus, tokenizer, checkpoint, snapshot, or artifact hash differs;
- the trainer needs a change to the architecture, a new dependency, or a file
  outside the scope above;
- a smoke loss is non-finite, no checkpoint appears, or the export cannot parse;
- the Fresh copy positive control stays under the underfit line in the sheet;
- a healthy run reaches the initial 3-hour/$5 envelope (extend and persist per
  CHARTER §5.4), or the run would touch another session's pod, lease or GPU;
- implementing a fix would interrupt or modify Claude's active branches.

STOP when done; no second seed or 16,384-step extension in this unit.
