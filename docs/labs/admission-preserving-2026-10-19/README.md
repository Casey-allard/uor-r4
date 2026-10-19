# Admission-preserving A8 acceptance-rule DRAFT  2026-10-19

**Status:** registered pre-registration, **`TEST FITNESS: FIT`** (2026-10-19). FIT covers the registered
design only; the run additionally requires the Codex controller/evaluator source prerequisite merged and
exact-head reviewed, then PM dispatch with the costed plan, then Casey's typed D40 release. Host-only
assembly; no compute, pod or training dispatch.

## Authorship

At 08:32Z the PM (Kimi, [#2029 comment 6263071040](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6263071040))
transferred authorship to DeepSeek after Claude's 08:30Z deadline passed with no ETA or blocker.
Claude keeps the review seat; Codex reviews device/evaluator/state identity and cost; Antigravity
provides non-author exact-head review and, on request, a consumer verifier.

## Artifact

`docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`  
SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`, 17,745 B, 126 lines.

## FIT record (2026-10-19)

`TEST FITNESS: FIT` recorded via the harness against the merged DRAFT (`70bfa3394`), citing AGY exact-head
scientific PASS `6263653594`/`6263679907` (both P0s closed), PM Option A disposition `6263591101`, and Codex
source/state/cost contract PASS `6263391731` + source activation `6263783076`. FIT covers the registered
design only; no run is released by this record.

## Binding design (PM disposition, not re-litigated)

1. **Interval-level gate** audited every **64 completed batches**, both arms.
2. **TRAIN-derived fixed anchor**: 3,847 genuine TRAIN `P_succ` rows (`is_copy_successor`, C=192) in corpus order
   from the unvisited TRAIN tail (actual eligible indices 1,049,857…1,064,112; 2,425,925 total eligible; no in-sample
   overlap, no DEV/heldout/fresh) with canonical `(row_ordinal,target_position,target_token,is_psucc)` keys. Host-only
   anchor SHA-256 is `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1` (3,847 CSV lines); checkpoint-0
   baseline bound before updates; baseline >=500 or D31.
3. **Active interval rejection (PM ruling 09:04Z):** margin 24; on breach, preserve failed evidence, restore the last
   passing model/counters/epoch, reject that interval's updates (selective if isolatable, whole interval otherwise),
   record per-update rejection receipts, advance the schedule cursor past the rejected interval, and continue.
   Rejected updates never commit; this changes the trajectory rather than passively stopping.
4. **No-op counter-metric with teeth:** stop `RULE_EFFECTIVELY_NOOP` if rejection rate >0.50 at Q1/full or after
   3 consecutive full-interval rejections; 24/50%/3 are prospective FIT choices, not MDE consequences.
5. Inherited A8 pre-run gates, Q1 gate, DEV decision bars and sealed fresh; no KEEP/mechanism claim from the guard
   alone. Gate latency is `UNMEASURED` at FIT; calibration needs its own authorized stage or occurs inside the final
   authorized run. Readiness uses `R = added_gate_cost / ungated_two_arm_dose_cost`; `R <= 0.15` is a pre-training
   readiness check, never a live-run budget stop.

## Review findings applied

Codex's exact-head review (6263219926) required five text fixes, now applied: the post-update
TRAIN-anchor evaluator is a required source prerequisite (no DEV fallback; existing `eval-corpus` refuses
post-update checkpoints), audits synchronously pause updates with epoch/full-checkpoint identity,
258 evaluations/arm including baseline and the 20-batch tail, cost calibration occurs only under the
released run or separately authorized step, and trigger-precedence/unreachable classifications are explicit.

Codex's exact-head re-review passed the source/evaluator/state/cost contract; two precision items were then
applied: the explicit 15% readiness formula (`R = added_gate_cost / ungated_two_arm_dose_cost`, shared setup/custody
common to both, `R > 0.15` stops before training and replans) and the clarification that the TRAIN audit never
supplies optimization loss/gradients/targets and never removes anchor rows from the parent's training corpus.

## P0 review resolution

PM ruled **Option A (active interval rejection)** at 09:04Z after Antigravity proved the passive stop rule could not
satisfy its own bars on the deterministic trajectory. The sheet now implements selective/whole-interval rejection with
per-update receipts, continuation from the restored state, and the rejection-rate no-op counter. P0-B remains fixed as
above. The sheet is still `TEST FITNESS: PENDING` pending AGY re-review.

Antigravity's exact-head review ([PR #2777 comment 6263479435](https://github.com/UOR-Foundation/uor-r4/pull/2777#issuecomment-6263479435))
found two P0 issues: (A) the non-resumable stop-on-first-breach rule cannot satisfy the sheet's own Q1-pass bars on a
deterministic A8 trajectory (PM disposition required: active rollback-and-continue, offline archive audit first, or
diagnostic-only); (B) the unconditional first-3,847 TRAIN targets were only 25.89% P_succ and in-sample — the anchor is
now bound to genuine TRAIN P_succ rows from the unvisited tail with exact anchor SHA-256 `8c1b80e9…` (3,847 rows,
indices 1,049,857…1,064,112). The sheet remains `TEST FITNESS: PENDING`; P0-A is
awaiting PM ruling and no run is authorized.

## What this record is not

No model/device evaluation, no admission recomputation, no source or evidence change, no corpus or
donor work, no training release. The sheet remains PENDING until independent review/FIT; any source
or registration change is a separate exact-head-reviewed delivery.

## Next

Claude/Codex/Antigravity review the exact sheet head; apply findings; FIT; then registration step and
PM dispatch. Do not rent or run from this record.
