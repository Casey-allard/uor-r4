# ORDER: cpu-training-pilot, deepseek-251 (CPU carry-hold checkpoint audit)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `0895aa65f`.

## Goal

Prepare CPU-side validation for Claude-334's held ask write without waiting for
or modifying its active B2 run.

1. Extend the checkpoint parser to recognize the merged `hold_rows` and
   `d2.hold` schema without changing non-held checkpoint behavior.
2. Add `cpu-carry-hold-audit`, a read-only binary that:
   - binds model and Hamming file hashes;
   - requires the stream-only hold row count and gain;
   - validates `d2.hold` shape/finiteness and row count;
   - reports gain mean, min, max, mean absolute value and nonzero coordinates;
   - reports type-write row `w_y` magnitude/nonzero counts;
   - reuses the 16-relation ask selection audit and records target-nearest
     counts beside the hold measurements.
3. Add synthetic controls for missing/malformed/nonfinite/readout-depth holds,
   deterministic output, hash refusal and expected row-count refusal.
4. If Claude-334 seals held checkpoints before this unit closes, run the binary
   on B2-aqH and B2-aH at both seeds. Otherwise record real checkpoint
   measurements as `NOT_RUN`.

This is checkpoint diagnosis only. It does not prove carry, B2, serving or
capability.

## Cancel probe

`uor-harness loop claim cpu-training-pilot --lab-topic carry-hold-audit` refused
on the known umbrella-scope false overlap with an unrelated open
one-model-end-to-end order. No file overlaps that order; use a hand-written
ORDER and let local policy prove the changed-path scope.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-251/**`
- `docs/labs/cpu-training-pilot-carry-hold-audit-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Existing non-held ask audits remain byte-identical.
2. Held-checkpoint tests cover the positive statistics and every malformed
   pair/shape/finite/depth refusal.
3. Synthetic output is deterministic and hash-bound.
4. Real Claude-334 held checkpoints are audited if available; otherwise the
   REPORT marks them `NOT_RUN` and gives the exact next command.
5. REPORT, policy, full gates and a fresh exact-head review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, PR #3886, serving schema,
artifact export, GPU state or another lab's work. Do not claim the hold audit
proves carry or B2.
