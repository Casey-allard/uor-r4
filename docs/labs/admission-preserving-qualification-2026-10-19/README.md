# Admission-preserving dose — finite qualification binding (2026-10-19)

**Status:** binding record. No compute, no pod. The FIT sheet
`docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` stays byte-exact at
SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`; this record carries
the qualification binding that must not mutate the sheet (Codex 6267441702).

**Source prerequisite (sheet step 2) — satisfied and bound:** the admission-preserving
controller and finite qualification producer merged as
`fd409fcdf3bc43d607e768bae6f8cf403aa75901` (#2782; AGY exact-head APPROVE 6266518570; 198 host
tests PASS; real CUDA qualification/restoration/timing NOT_RUN at merge).

**Protocol:** `train61-synthetic1-interval64-two-arm-throughput-v1` — **384 attempts per real
arm** with accepted-update/production restore evidence, selected baseline samples, complete
timing and custody scope. The dose's pre-flight includes this finite CUDA qualification
**before any full-dose training update**:

1. **Rollback equivalence (fixture level):** restore-from-checkpoint after a synthetic failed
   interval reproduces the pre-interval model state byte-exact (model, cursors, epoch,
   counters), with accepted-update and production-restore evidence.
2. **Interval-audit timing:** measured per-audit cost on the run hardware, feeding the sheet's
   readiness formula `R = added_gate_cost / ungated_two_arm_dose_cost ≤ 15%`; the measured value
   is recorded on the run receipt.
3. **Rejection receipts:** the producer emits the selective/whole-interval rejection receipt
   schema; format verified against the sheet's record requirements.

A qualification failure is an instrument stop (`INSTRUMENT_UNREACHABLE`, D31) before any
training update; no DEV fallback. Qualification draft and quote: #2029 comment 6266885149.
The independent chain stands: finite FIT'd sheet → owner costed-plan approval → PM dispatch →
Casey's typed D40 release.
