# ORDER: m1-a8-full-dose-rerun, antigravity-57

Lab: antigravity. Item: [m1-a8-full-dose-rerun](../../../../roadmap/items/m1-a8-full-dose-rerun.md). Base: `31ebdb8b1`.

## Goal

The item's done-when reads:

> Paired DEV verdict recorded for the registered arms and seeds; reach kept (M1 bar (b), G-Adm) or a registered diagnostic stop classified as instrument versus mechanism; fresh slice stays sealed.

Execute the registered M1 A8 admission-preserving full-dose rerun (`Relation` `LearnV1` seed 1 vs `FrozenZero` `FrozenZeroV1` seed 1, `AuditPolicy::registered()` with `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, `consecutive_full_rejections = 3`, `audits_per_arm = 258`, `max_batches = 16,404`) on GPU after replanning only the pre-`rows_v1` `15%` relative gate-cost ratio (`R = 78.41%` post-`rows_v1`) under a one-page prep sheet, non-author FIT review, D68 decision review, and D67 pre-run review.

## Cancel probe (run first)

Ran `a8-qualify-stage verify` on `/Users/casey.allard/uor-r4/local/antigravity/a8-finite-cuda-qual-20261025/a8-finite-cuda-qual-20261025/qual-out`:

- **Result**: `status = VERIFIED_FINITE_CUDA_QUALIFICATION`, `finite_qualification_controls_status = PASS` (`synthetic-control.json` `PASS` and all 5 Stage 1 CUDA stages `PASS` on both arms; `792` timing pins and `142` bookkeeping pins verified), `readiness_gate_status = REPLAN_BEFORE_TRAINING` (`projected_added_gate_ns = 4,947,754,777,700 ns`, `ungated_two_arm_dose_ns = 6,309,716,689,996 ns`, `readiness_ratio_bps_ceil = 7842`).
- **Classification**: `instrument` (CHARTER §4.2). The 19 Oct `15%` relative ratio was calibrated against pre-`rows_v1` ungated training (`~43,000 s`, where `15%` was `~6,450 s` / `$1.71`); after the `rows_v1` `6.8x` training speedup (`6,309.72 s`), the entire worst-case audited two-arm workload (`11,257.47 s = 3.13 GPU-hours`, `~$2.84` on RTX 5090) is below both the 19 Oct `$1.71` (`6,450 s`) gate allowance and `$11.40` total budget while `validate_ratio` (`crates/uor-r4-discrete/src/attention_audited.rs:112-119`) still enforces the pre-`rows_v1` `15%` ratio. Replanning the relative ratio bound while keeping `AuditPolicy::registered()` unchanged unblocks the full-dose rerun.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-a8-full-dose-rerun*`
- `docs/labs/m1-a8-full-dose-rerun/**`
- `docs/loop/tasks/m1-a8-full-dose-rerun/**`

## Done when

1. One-page prep sheet `docs/harness/prep/m1-a8-full-dose-rerun-2026-10-25.md` is committed with fresh-context non-author `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review posted on `#2831`.
2. `crates/uor-r4-discrete` enforces the replanned post-`rows_v1` gate-cost readiness bound (`R <= 85%`, `projected_added_gate_ns <= 5,500,000,000,000 ns`) with `AuditPolicy::registered()` unchanged, and `a8-qualify-stage` stages and verifies the two-arm dose.
3. Both registered arms (`Relation` seed 1 and `FrozenZero` seed 1) execute via `discrete-stack attention-audit` on a leased RTX 5090 GPU pod; the paired DEV verdict (or registered diagnostic stop classified as `instrument` vs `mechanism`) is recorded with `fresh_evaluations == 0`, uploaded to `caseyallard/uor-r4-store`, and recorded in `docs/labs/m1-a8-full-dose-rerun/`, `docs/research/notebook/m1-discrete-attention.md`, and `REPORT.md`.

## Stop and ask if

- Any Stage 1 finite qualification control or anchor invariant fails verification.
- Any file outside the declared scope is needed.
- GPU pod spend approaches the `$8/h` or 4-pod fleet cap.
