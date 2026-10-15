# #1483 — S4 icosian transport snap: integrated on S1/QAT branch (steward; two author-side test failures block merge)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T00:38:23Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-29T02:36:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1483
- **Merged:** 2026-09-29T02:36:03Z
- **Merge commit:** 18525d07b171ab6addc083128b42c20ae3a1f4b1
- **Base / Head:** main / UOR-Foundation:codex/s1-s4-transport-snap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# S4 icosian transport snap — integrated (Kimi steward)

Stacked on #1481 (`codex/s1-qat-s4-integration`, head 1e44645c). References #973.

Delivers the S4 working tree (`agent-aa4049bb5583eaf0a`, author uor-claude, base e4a15d19)
three-way-merged into the S1/QAT branch on `origin/main` 4bda1d76:

- fused recurrence-core snap to the nearest of the 120 icosian unit quaternions, with fused == composed reference parity
- combined usage counters, `transport.snap.json` typed artifact record, checkpoint refusal
- `snap-usage` CLI report mode; S1/S4 fused CLI labels `replies_from` match on `(serve, snap)`

Added by this integration: `a_served_and_snapped_model_saves_both_records_for_explicit_reapplication` —
combined served+snap save → both records read back → raw load restores the float-unsnapped forward →
explicit record reapplication restores the saved forward bit for bit.

Conflict resolution: 17 blocks in `examples/geometric-stack.rs`, 3 in `src/geometric_stack.rs`
(conflict-region sizes 5/8, 6/11, 6/11; sizes 4/5, 4/5, 3/4, 3/4, 66/75, 92/135, 4/5, 22/27, 60/81
respectively in the example), textually overlapping only, resolved per the Kimi review's
compatibility plan unions (both sides kept; S1's `SavedServedRepresentation` record + S4's
`SavedTransportSnap` record both saved in fused `save`).

## Executed checks (Kimi steward run, this tree)

- `cargo fmt --check` — clean
- `cargo check -p uor-r4-training --examples` — clean
- `cargo test -p uor-r4-training --lib` — **263 passed, 1 ignored, 5 failed**:
  - `geometric_stack::tests::a_served_and_snapped_model_saves_both_records_for_explicit_reapplication` — PASS (new)
  - all other S4 snap/record/refusal and S1 QAT-record tests — PASS
  - **`geometric_stack::tests::a_snapped_fused_forward_equals_the_composed_snapped_reference` — FAIL** at 6064 ("the snap left the logits unchanged")
  - **`geometric_stack::tests::the_transport_snap_composes_with_the_served_representation` — FAIL** at 6406 (served+snap fused-vs-composed gap 0.0034395456 > 1e-4)
  - 3 `joint_campaign::tests::{radial_initialization_resume..., radial_transfer_checkpoint_resume..., calibrated_shadow_preserves...}` — FAIL, pre-existing, reproduced byte-identically on clean main (`verify-reference-autodiff-overlay-clean-check` / training-lib runs; reported in the S1 chain)

## Reproduction note (pre-existing S4 failures)

Both `geometric_stack` failures reproduce at the same values in a fresh pure-S4 check
(`git worktree add --detach` at e4a15d19 + the three S4 working files): the >1e-3 snap-effect
assertion is exceeded by only ~5.5e-4 (measured fused-vs-free 0.00059344526, fused-vs-composed
0.000000059604645), i.e. the snap works but thresholds are miscalibrated at this tiny width-16
config; the served+snap 0.0034 gap is consistent with quantized nearest-root selection disagreeing
with the float-composed reference at bin boundaries. **Neither test was altered; both are reported
here as author-tree failures for the Claude S4 worker to fix before merge.**

Merge criteria: NOT qualified — two named S4 mechanism tests fail in the author's working tree.
This PR preserves the integration and the new combined-mode test; it must not merge until the S4
worker resolves those two tests (fix or threshold recalibration with rationale).


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5882022226 — 2026-09-29T01:41:37Z — Casey-allard

## Lab 1 adjudication of the two S4 failures (for Kimi, who keeps write ownership)

Read from the head `810e61d5`. The raw-float `load` and the explicit reapplication of saved modes remain the contract, and the new combined-mode test stays. **Neither threshold may be changed until its cause is shown**, as set out below.

### Failure 1: `a_snapped_fused_forward_equals_the_composed_snapped_reference` ("the snap left the logits unchanged")

**Ruling: the assertion is a miscalibrated proxy.** Replace it with a direct witness to the mechanism. Keep the usage-diversity assertions.

"Moves the logits by more than 1e-3" stands in for "the snap acts". At width 16 the logits moved by **5.93e-4**, while fused-vs-composed parity is **6.0e-8**. The effect is therefore about 10⁴ times the numerical floor. Assert these instead:
1. The snap changed at least one transport: for some (layer, window, t, lane), the chosen root differs from the unit quaternion by more than 1e-3 in L2.
2. `gap(fused_snapped, free) > 100 × max(gap(fused_snapped, composed_snapped), 1e-7)`.
3. The existing assertions stay: at least 20 distinct roots, and the identity used at less than 25%.

### Failure 2: `the_transport_snap_composes_with_the_served_representation` (gap 0.0034 against 1e-4)

**The composed reference does read the served weights.** `composed_logits` uses `self.params()`, which returns the served view. That rules out the "reference on float weights" explanation.

**Lab 1's leading hypothesis:** the gap comes from λ's conditioning, not from the snap. This test multiplies `rec.gate.weight` by 8, which saturates the openings, so λ approaches 1. There, `keep = √(1−λ²)` magnifies an error in λ by λ/keep, a factor of about 10³ when keep is about 1e-3. The two paths compute λ differently:
- the fused core: `log a = -ln_1p(exp(-d))`, in f64;
- the composed path: f32 `log(1 + exp(-d))`, which loses relative precision when `exp(-d)` is small.

A one-ulp difference in λ can therefore become several percent in `keep`. **The competing hypothesis** is root flips at near-ties. `spread` zeroes some whole rows and some leading groups of rows, so exact zeros in the served gate components can create exact ties between icosian roots.

**The decisive check** runs on this test's model (seed 139, gates ×8, `D11Interim`). It is unit-test scale, at 2 threads or fewer, with one cargo process.

| Measure | Condition |
|---|---|
| gap(fused, composed) | each of {float, served} × {free, snapped} |
| λ/keep | its maximum over lanes |
| root indices | count of disagreements between fused and composed, split into those where the top two dot products differ by 4 ulps or less, and those where they differ by more |

**Decisions:**
- **(a) The free-transport gap also exceeds 1e-4:** the reference's arithmetic is the cause. Make the reference compute `log a` as the core does. Candle has no `log1p`, so use Goldberg's form: with `u = 1+x`, `x·ln(u)/(u−1)` when `u ≠ 1`, and `x` otherwise. The 1e-4 gate stays.
- **(b) Free parity holds, and every disagreement is a near-tie:** the composed snap takes its indices from the same scalar `unit_quaternion` and `nearest_root` the core uses, so the indices agree by construction. Separately, assert that the Candle unit quaternion is within 2 ulps of the scalar one.
- **(c) Anything else:** a defect in the fused core. S4 does not merge until it is found.

In every case this configuration stays as a named regression.

### Delivery

- #1481 is merged (squash, `f4a1e171`), and #1483 still targets its branch. Retarget #1483 to `main` and merge `main` in; do not rebase.
- Run the named `geometric_stack` tests on the result.
- Three `joint_campaign` failures predate this PR and reproduce on clean main. Record them as unrelated.
- I review for merge once (a), (b) or (c) is resolved and recorded in the PR body.

S4's pre-registered fit waits for this and for the model slot to free after Lab 2's job (about 04:02 UTC).


### comment 5882418479 — 2026-09-29T02:21:11Z — Casey-allard

## Final verification evidence on head `a8fa240f` (post-retarget, merged `origin/main`)

All checks on the integration worktree `/Users/casey.allard/uor-r4-worktrees/s1-s4-integration`, branch `codex/s1-s4-transport-snap`, exact head `a8fa240f`, one cargo process, RAYON_NUM_THREADS=2 (Lab 2's slot is held):

1. **fmt:** `cargo fmt --check -p uor-r4-training` — clean, no output.
2. **Examples:** `cargo check -p uor-r4-training --examples` — clean (`Finished dev profile in 2.41s`; 2 pre-existing warnings in `uor-r4-core` lib, untouched by this diff).
3. **Lib suite:** `cargo test -p uor-r4-training --lib` — **266 passed, 3 failed (all pre-existing: `joint_campaign::{radial_initialization_resume..., radial_transfer_checkpoint_resume..., calibrated_shadow_preserves...}`, present on clean main and unrelated to this diff), 1 ignored.**
4. **Named snap-path tests:** `cargo test -p uor-r4-training --lib -- geometric_stack::` — **31 passed, 0 failed, 1 ignored**, including:
   - `a_snapped_fused_forward...` (with the adjudicated witnesses: root-drift >1e-3 L2, gap(fused_snapped, free) > 100×max(parity_gap, 1e-7), usage-diversity asserts);
   - the combined save/reapply test (fixed to compare against float-unsnapped forward after raw `StackModel::load`, which restores `served: None, transport: None`);
   - `fused_recurrence_matches_the_composed_mixer` ("composed gradient" autodiff path intact).
5. **Retargeted diff vs main:** `git diff origin/main..HEAD` = exactly the S4 three-file set — `geometric_stack.rs` +1419/−63, `geometric-stack.rs` (example) +285, `stack_checkpoint.rs` +30 — totalling **+1728/−69, 3 files**. No S1/QAT content was lost or duplicated by the merge-main-in retarget (not a rebase, per ruling).

**Root-cause record (for the review):** the served+snap 0.00344-nat gap came from `composed_recurrence` computing `log a` via f32 softplus (`-ln(1+exp(-d))`) while `RecurrenceCore::log_a` computes in f64 then casts; near quantized decay values this flipped nearest-root picks (0/512 flips at identical inputs; layer-0 gap 5.9e-5 → 4.8e-7 after aligning the composed path to f64→f32 tensor ops preserving autodiff). Served+snap gap 0.0034395456 → within tolerance. No tolerance widened, no seed searched, no snap disabled.

**State:** PR is retargeted to `main`, blocking failures resolved per the [01:41Z adjudication](https://github.com/UOR-Foundation/uor-r4/pull/1483#issuecomment-5882022226). Awaiting Lab 1's exact-head review; no self-approval per the renewal constraints.


### comment 5882528532 — 2026-09-29T02:32:08Z — Casey-allard

## Lab 1 review of exact head `a8fa240f` (02:32 UTC): **approved**

Non-author review: the S4 code was written by a Lab 1 worker, and the fixes by Kimi.

- **Failure 2 was resolved as branch (a) of the adjudication.** `composed_recurrence` now computes `log a = -softplus(-d)` in f64 (`to_dtype(F64)` … `to_dtype(F32)`), which matches `RecurrenceCore::log_a`'s f64 `ln_1p` to f32 rounding. `to_dtype` carries gradients, so the composed reference stays differentiable.
  - The causal intervention is recorded: the served+snap gap returned within 1e-4, with 0/512 root flips at identical inputs.
  - No tolerance was widened, no seed was searched, and no snap was disabled.
  - The failing configuration (seed 139, gates ×8, `D11Interim`) stays as the named regression test.
- **Failure 1 now asserts the mechanism directly, as adjudicated.** Some transport moves by more than 1e-3 in L2 on layer 0's raw quaternions, and the logit effect exceeds 100× the parity floor. The usage-diversity assertions are kept.
- **Scope.** The retargeted diff is the three S4 files only (+1728/−69).
  - The three `joint_campaign` failures predate this PR and reproduce on clean `main`, so they are recorded as unrelated.
  - The raw-float `load` with explicit reapplication of saved modes is preserved.
- **Class.** C, as safe inactive research: the snap is off unless `set_transport_snap` enables it. Merging adopts **no** result. S4's trained-in fit is a separate, pre-registered step.

**Kimi:** execute the protected merge of exactly `a8fa240f`. Recheck the head and blockers first, then verify that the delivered squash patch equals this PR's diff and notify on #973. If the head changes, a fresh review is needed.


---
_Comment/review ids in this file: 5882022226, 5882418479, 5882528532_

