# ORDER: cpu-training-pilot, deepseek-247 (real ask-offset checkpoint regression)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `68a60425f`.

## Goal

Turn deepseek-245/246's one-off Claude-330 checkpoint audits into a repeatable,
hash-pinned host regression:

1. Add a manifest-driven Rust runner that audits multiple trained ask-offset
   checkpoints in parallel, compares every observed summary with an expected
   result, and writes one deterministic report only to a new path.
2. Check in the eight real B2-arm observations already measured: B1-a,
   B1-a0, B2-a and B2-aq at seeds 1 and 2, with the sealed archive and
   checkpoint hashes that produced them.
3. Re-run the real checkpoints on the workstation and retain a compact JSON
   report and resource ledger. Do not commit checkpoints, tarballs or model
   binaries.

The result is an instrument for future ask/carry changes to reproduce the
known selection split without a GPU. It is not a new B2, carry or capability
result.

## Claim

`uor-harness loop claim cpu-training-pilot --lab-topic real-ask-regression`
refused because `crates/uor-r4-cpu-train/**` overlaps another open
one-model-end-to-end order's broad CPU glob. That order and this one touch no
common file. This hand-written ORDER uses the existing umbrella exception and
the actual changed-path policy gate proves the narrower delivery.

## Cancel probe

- `TrainedAskOffsets` already exposes the exact relation and cell readings.
- `cpu-ask-offset-audit` already stores one checkpoint's report.
- No runner currently consumes multiple real checkpoint arms, compares
  hash-pinned expected summaries, parallelizes those independent audits or
  fails when a future code change changes the known split.

If any claim is false, stop and re-scope before changing product code.

## Plan and files in scope

1. Add `crates/uor-r4-cpu-train/src/ask_offset_regression.rs` with a strict
   manifest schema, checkpoint validation, deterministic aggregation and
   expected-field comparison.
2. Add `crates/uor-r4-cpu-train/src/bin/cpu-ask-arm-regression.rs` with
   bounded workers, create-new output and nonzero failure on any mismatch.
3. Add unit and binary tests for valid multi-arm execution, hash refusal,
   missing checkpoint data, duplicate identities, wrong expected values,
   deterministic output and argument refusal.
4. Add the hash-pinned real-arm manifest and a compact retained report under
   this task folder; measure wall time and peak RSS.
5. Add one notebook row and a resource ledger.

Files:

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/ask_offset_regression.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-arm-regression.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-247/**`
- `docs/labs/cpu-training-pilot-real-ask-regression-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Targeted tests and mutations fail on the checked regression.
2. The real eight-checkpoint run passes against the manifest and reproduces
   the known 64/64, 9/64, 1/64, 0/64, 3/64 and 64/64 target-nearest counts.
3. The runner uses more than one CPU worker and reports bounded worker count.
4. REPORT, report check, local policy, full gates and a fresh exact-head
   review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, PR #3882, serving schemas,
the integer engine, GPU state or another lab's work. Do not treat this
regression as B2, carry, capability or serving evidence.
