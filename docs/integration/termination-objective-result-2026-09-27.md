# Termination-weighted objective result

**Scope.** Executes the [predeclared plan](termination-objective-plan-2026-09-27.md) —
a bounded objective-change experiment on the accepted step-15,672 parents. Development
only: parents preserved, frozen evaluator and prose criteria unchanged, no candidate
promoted, and no further dose follows automatically from this result.

## Instrument and identity

A new additive, absent-by-default `end_weight` in `uor-r4-training` multiplies the
**training-objective** loss at targets that decode to `.` `!` `?` or EOS (boundary ids
`[1, 3, 16, 33]`); the weighted mean uses the weight sum as denominator and only the
training gradient is weighted, so every reported metric (development NLL, retained-fit
NLL, checkpoints) stays standard and comparable. Executed source `c54d7801`; binary
sha256 `b0ff04aa…`; the default path is proven unchanged (uniform-weight gradients match
the existing loss within 1e-5; no report field or evaluation path changed).

The training artifacts were produced by source `c54d7801` (base `f8bee0ff`). The
delivered branch rebases the same additive diff onto `bbce4cf6` over intervening
absent-by-default merges; on the rebased source the full library suite passes (132/132)
and re-evaluating both treatment checkpoints (`…-witness` roots, binary `9e351540…`)
reproduces the sealed `generations.json` and `story-probes.json` identically, so the
delivered evaluation path reproduces the artifacts exactly.

Four fits ran sequentially from the parents: 15,672 → 16,696 (**1,024 updates /
4,194,304 targets** each), B16/T256/full256/shard2/seed 240924, no resumes, single final
checkpoint each. Treatment arms use `end_weight = 2.5`; controls leave it absent.
Campaign copies are committed under
`docs/evidence/termination-objective-2026-09-27/campaigns/`.

Cross-lab: this diff is additive and absent-by-default relative to the fourth lab's
active `joint_campaign.rs` initializer work (either branch rebases over the other), and
Google's open PRs touch only `crates/uor-r4-integer`.

## Fits

| Fit | Status | Steps | Elapsed | Initial dev NLL | Final dev NLL | Final retained NLL |
|---|---|---:|---:|---:|---:|---:|
| quaternion-endw | TARGET_COMPLETE | 15,672→16,696 | 2,686 s | 2.095483 | 2.098166 | 2.101165 |
| quaternion-plain | TARGET_COMPLETE | 15,672→16,696 | 2,861 s | 2.095483 | 2.087941 | 2.094570 |
| householder_pair-endw | TARGET_COMPLETE | 15,672→16,696 | 4,768 s | 2.076352 | 2.080168 | 2.090829 |
| householder_pair-plain | TARGET_COMPLETE | 15,672→16,696 | 4,752 s | 2.076352 | 2.064079 | 2.081592 |

The standard unweighted development NLL is slightly **worse** for the treatment than the
dose-matched control in both arms (quaternion +0.0102, householder +0.0161): the
reweighted objective is not free.

## Evaluations (frozen evaluator; sample and probe policies unchanged)

| Final | Comparison-tail NLL | NoRead NLL | NoRead penalty | Complete /32 | First-noun /32 | Pairs /16 |
|---|---:|---:|---:|---:|---:|---:|
| Parent quaternion (15,672) | 1.996490 | 2.492065 | 0.495575 | 21 | 26 | 8 |
| quaternion-endw | 1.994137 | 2.500926 | 0.506789 | **29** | 30 | 14 |
| quaternion-plain | 1.984753 | 2.491002 | 0.506250 | **27** | 29 | 12 |
| Parent householder pair (15,672) | 1.974724 | 2.472759 | 0.498035 | 23 | 31 | 10 |
| householder_pair-endw | 1.977860 | 2.484759 | 0.506900 | **29** | 31 | 13 |
| householder_pair-plain | 1.965662 | 2.475963 | 0.510301 | **28** | 31 | 13 |

Every story-probe completion stops at `first_sentence_boundary`; no short-cycle stops.

## Source-row transitions (the concrete greedy panel)

| Arm | Parent → endw (resolved/regressed) | Parent → plain (resolved/regressed) | endw-only (vs control) | plain-only |
|---|---|---|---:|---:|
| Quaternion | 21→29 (8 / 0) | 21→27 (6 / 0) | 2 (`03/original`, `04/edited`) | 0 |
| Householder pair | 23→29 (7 / 1) | 23→28 (6 / 1) | 1 (`15/original`) | 0 |

The single regression (`11/original`, `key` → `keys.`) occurs identically in **both**
conditions, so it is dose-side, not objective-side. Combined, 15 of the 20 failing rows
resolve under the treatment and 12 under the control; treatment-specific rows total **3**
(2 non-termination + 1 wrong-first-noun) against **0** control-specific.

**EXTEND source threshold (`≥4 vs control`): not met.** Reading 1 (endw-only flips ≥ 4)
gives 3; the absolute non-termination reading gives `10` (endw) vs `8` (control) — a +2
increment over control, again below 4.

## Prose (the primary criterion)

Principal read against the frozen four criteria: **0/5 in all four finals**, and the
independent evidence audit returned identical verdicts. The closest calls (role/entity
slips that keep them below the all-four bar) are quaternion-plain seed 2015 and
householder-pair-endw seed 2016; the audit confirmed that only an unprincipled
asymmetric judgment of those two stories would flip the branch. Sampled outputs stop at
the 128-token cap in 16 of 20 candidate stories (4/5, 3/5, 4/5, 5/5 per final), so
cap-termination is not resolved by this dose.

**EXTEND story threshold (`≥1 acceptable story vs parent and control`): not met** (all
arms 0, parent 0).

## Guardrails

Hold in every arm: comparison-tail NLL within parent + 0.05; NoRead penalty ≥ 0.02; no
short-cycle increases; source-panel complete ≥ parent − 2 (under either the continuous or
the retained-integer parent reading).

## Decision (predeclared rule applied)

**INERT.** Neither EXTEND threshold is robustly met, and HARM is not indicated
(guardrails hold; prose does not worsen beyond the baseline 0/5). The measured facts are:
the treatment adds a marginal +3 unique rows (2 non-termination, 1 wrong-noun) with zero
unique losses and a small standard-NLL cost, while the **plain dose alone already
recovers most of the panel** (12 of 20 failing rows) with no prose gain either. At this
dose, termination weighting is a weak lever, not the working lever.

Two disclosures, kept explicit rather than silently resolved:

1. **Rule gap.** The INERT band was written as "probe rows within ±2"; the parent-relative
   movement is +6 to +8 rows. Read as **treatment-minus-control** (+2 and +1), the band
   holds, and the parent-relative movement is a separate dose effect. The plan is not
   rewritten; this interpretation is recorded.
2. **Judgment sensitivity.** The branch hinges on at most two borderline stories; the
   strict reading (both principal and audit) gives 0/5 everywhere.

**Consequence: stop the termination-weighting branch.** The residual failures under
greedy remain entity/role collapse and cap-truncation, which place the next rung in the
**state/read path** — the cross-lab native radial reader comparison already in flight, or
a declared conditional-depth hypothesis. No promotion; the parents remain the accepted
artifacts and no new dose is authorized by this result.

## Limits

- One dose (1,024 updates) and one weight (2.5); this INERT result does not sweep either.
- The authored 32-row panel is dose-sensitive; panel movement is a development
  observation, not a capability claim.
- Sixteen of twenty sampled stories stop at the cap, limiting termination measurement;
  the frozen criteria and stop policy are unchanged.
- No integer/hard export, no fresh holdout, no energy or general-language claim.

## Cost and delivery

- Fits 15,069 s total; evaluations 273 s; checks/tests/build ≈200 s; one ≈2 min lane wait;
  total model wall ≈4.6 h; retained storage ≈180 MiB inside the existing model store; SSD
  build cache reused; complete elapsed charged once to the shared ledger.
- [Evidence](../evidence/termination-objective-2026-09-27.json) binds the campaigns,
  checkpoints, evaluations, row sets, guardrails and the independent audit. Delivered
  through a protected PR referencing #973 and #820; issue and handoff updated.
