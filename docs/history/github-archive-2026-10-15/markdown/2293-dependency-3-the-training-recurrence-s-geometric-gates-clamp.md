# #2293 — Dependency 3: the training recurrence's geometric gates (clamped opening, exact rate table, Hamilton output gate)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:55:40Z
- **Updated:** 2026-10-13T04:58:38Z
- **Closed:** 2026-10-13T04:57:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2293
- **Merged:** 2026-10-13T04:57:25Z
- **Merge commit:** d7934bf03c50d0275617f5fa1ad133c6224d8449
- **Base / Head:** main / UOR-Foundation:kimi/recurrence-geometric-gates-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged; v5 memory 20/40 unchanged)

## Dependency 3 delivered: the training recurrence's geometric gates

Record: `docs/labs/recurrence-geometric-gates-2026-10-13/README.md`. Base `9a20ca953`, rebased onto
`origin/main` @ `f387345d4` (the only conflict was the shared `current-state.md` ledger; both entries kept).
Decision: **NOT YET PROMOTED** — source only, no run, no device receipt. Device qualification is dependency 4.

### The claim, stated only because it is verified

**The recurrence is now squash-free in the executed training path** for `rec_gates=geometric`. Verified by grep
after the change (`exp|expf|exp2f|sigmoid|tanh|tanhf|softplus|log1p|gelu`, word-boundaried), per arm:

| Region | squash-family tokens |
| --- | --- |
| geometric decay arm (`composed_recurrence`, host) | **0** |
| geometric output-gate arm (host) | **0** |
| `rate_select` host op — code | **0** (its docs name `exp`/`softplus` twice, to say they are absent) |
| `rate_select` CUDA kernels (`cuda_stack_kernels.rs` § 2d) — code | **0** |

Qualification, stated rather than implied: `RecGates::Historical` remains the default and retains the old chain
in its `else` branches (8 such tokens in the same function) so existing configs, checkpoints and tests
reproduce exactly — it is documented as prohibited for new runs and is not what rung 0 declares. The fused
legacy `RecurrenceCore` op is likewise retained for that mode and is **not** on the geometric path (the model
executes the op composition). The read path's `exp` is untouched: rung 0 has no read layer (rung 0b's item).

### What landed

- **`RecGates`** on `StackConfig` (`historical` default, `geometric`), the same config-selected pattern as
  `Gate`/`Objective`/`QuaternionWeights`; `rec_gates=geometric` composes the model forward as: clamped-linear
  opening `clamp(gate + raw per-lane decay score, 0, 1)` → **exact rate selection** → the unchanged
  `quaternion_scan` carry → **`hamilton_gate`** (OP1, #2260) on the recurrence output. The per-lane decay
  parameter is read **raw** as the rate-selector score — its softplus transform is gone.
- **The fixed dyadic table**: `lambda = 2^-(s+1)`, `s = 0..16`, exact f32 powers of two, with
  `keep = sqrt(1 - lambda^2)` precomputed beside each entry (`|keep^2 + lambda^2 - 1| <= 6e-8`; the last four
  keeps are the documented f32 plateau at `1`). The D11-compatible form: serving is table reads.
- **The declared geometric secant surrogate** (the `#2279` pattern, labelled the same way in code and docs):
  the hard selection's derivative is zero almost everywhere, so the backward pairs the incoming adjoints with
  the selected entry's lawful neighbours' complete counterfactual table displacement, resolved along the fixed
  edge `1/16` — half per side, the whole weight to a lone neighbour at either end. No probability, no
  exponential, no learned coefficient.
- **CUDA kernels + f32/bf16 launchers** for the table read and its adjoint (`act_t`-typed; `lambda` built from
  its exact f32 bit pattern, `keep` by `sqrtf`).
- **Refusals**: a geometric-recurrence model refuses any non-CUDA device loudly at the model level (D24); the
  mode is refused on a transformer control or a width that is not whole 4-blocks; and **export/D11 and
  `qat=true` refuse it** because the integer engine binds the historical decay parameter — a stale or
  undeclared binding is refused, never silently converted.

### Checks at this exact head

- [x] Checks run at this exact head, offline, on the owner's laptop
- [x] `cargo test -p uor-r4-training --lib --offline` — **951 passed / 3 failed / 10 ignored**. The 3 failures
      are **pre-existing on main**, not mine: `a_trained_in_activation_snap_is_straight_through_and_round_trips`,
      `the_icosian_snap_is_straight_through_and_round_trips`, `the_snap_changes_the_logits_to_those_of_the_snapped_weights`
      all fail with `#2279`'s refusal ("dense quaternion artifacts are retained inactive; their model path is
      forbidden"), which is present in this branch's base commit `9a20ca953` (checked with `git show`). A
      flaky `joint_campaign` checkpoint-binding family (3 tests) also fails intermittently on main; it passed
      in the final run.
- [x] `cargo check -p uor-r4-training --all-targets --offline` — 0 errors (before and after the rebase)
- [x] `cargo fmt -p uor-r4-training --check` — clean
- [x] `python3 scripts/check_no_softmax.py` — **before 1649, after 1649** (baseline unchanged: nothing was
      removed and no occurrence was added — the historical chain is retained, not deleted; **no baseline
      update**)
- [x] `python3 scripts/check_no_matmul.py` — **before 93, after 93** (baseline unchanged)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/harness/cli.py check` — 0 errors (the `prep.fit_before_run` warn is the rung-0 sheet's
      `PENDING` state, which is correct; the rest are other milestones' unfrozen panels and other labs' branches)
- [x] The new kernels were **executed on the host from their own source text** and checked against an
      independent reference: rate table 5e-13 max abs, keep table 4.6e-10 max rel, secant credit 5.5e-8 max rel
      (arithmetic and indexing only — no device claim)
- [x] Tests added (7): table exactness/monotonicity and `keep^2 + lambda^2 = 1`; hand-computed selection
      including both table ends; the secant credit against hand values for an interior and a boundary entry;
      the clamp's exact subgradient; the model CUDA refusal and the two validation refusals; a composed
      one-step case; the mode's serde round-trip with byte-identical legacy configs
- [x] The full suite was run on the pre-rebase tree; the rebase touched **only** `docs/integration/current-state.md`
      (verified), so no compiled or tested source changed after that run

### Deviations, stated plainly

1. **The historical recurrence is retained in-tree, so no branch-archive patch was created.** The brief asked
   for the old chain to be preserved as an indexed patch; nothing was deleted, so nothing needed archiving —
   the historical form *is* the reproduction path for every existing artifact (the `Gate::Swiglu` /
   `Objective::Ce` precedent). If you prefer deletion-with-archive, it is a mechanical follow-up.
2. **The amended sheet is acknowledged**: #2290's NOT_FIT review rewrote the rung-0 bars (vacuity-proof); the
   sheet still lists the recurrence as dependency 3 and the run as blocked on dependencies 2–4 with dependency 1
   (#2279) satisfied — this PR delivers 3 and touches nothing in the sheet.
3. **Main's pre-existing test failures are reported, not absorbed** (above): this PR adds none and fixes none.

Cost: one source session on the owner's laptop, no compute, no pod leased.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

