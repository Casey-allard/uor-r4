# ORDER: cpu-training-pilot, deepseek-248 (repair exact-head review findings)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `380ad4c32`. Parent ORDER: [deepseek-247](../deepseek-247/ORDER.md).

## Goal

Repair the five material findings from the fresh exact-head review of deepseek-247:

1. Correct the README's `uor-store get` root. The store client preserves the
   full remote path, so extraction must start from the downloaded `runs-2`
   directory.
2. Make checkpoint identity unique. Reject two arms that name the same
   checkpoint path or the same `(model_sha256, hamming_sha256)` pair.
3. Separate source failures from regression mismatches. Missing or corrupt
   checkpoint evidence remains visible in the report but exits `1`; an
   observed summary that differs from its expectation exits `2`.
4. Scope the notebook conclusion to B2-a. B1-a's found-copy pull at scored rows
   does select the target, so "scored-row training" is too broad.
5. Complete the resource ledger with the downloaded archive bytes and measured
   extracted checkpoint-tree bytes, alongside retained text and build storage.

## Files in scope

- `crates/uor-r4-cpu-train/src/ask_offset_regression.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-arm-regression.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-247/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-248/**`
- `docs/labs/cpu-training-pilot-real-ask-regression-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Tests refuse duplicate checkpoint paths and duplicate checkpoint hash
   pairs.
2. A source-error report has zero regression mismatches but is classified as
   an exit-1 source failure; a summary mismatch remains exit 2.
3. The reproduction instructions agree with the store client's directory
   semantics.
4. The notebook claim and storage ledger are corrected.
5. Targeted checks, report policy, full gates and a second fresh exact-head
   review pass.

## Stop and ask if

Do not change the accepted checkpoint metrics, archive/checkpoint hashes,
Claude-owned files, GPU state, serving schema or another lab's work.
