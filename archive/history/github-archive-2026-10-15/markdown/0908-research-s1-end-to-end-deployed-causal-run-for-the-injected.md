# #908 — research(S1): end-to-end deployed causal run for the injected-candidate skip-mix lane (#897/#904/#906) → S1 promotion verdict vs the frozen 20‰ floor

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T22:26:45Z
- **Updated:** 2026-08-25T06:11:47Z
- **Closed:** 2026-08-21T23:14:52Z
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/908
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The S1-redesign skip-mix lowering track has cleared its deployed-fidelity spot-check. #897 lowered the confirmed 1-token skip-mix scorer (SKMX/PSIB) onto the deployed `R4Engine` and shipped it **dormant** after a spot-check gap (41/87). #904 diagnosed that gap as BREADTH-BOUND, and #906 fixed it with candidate injection — the deployed lane now discovers candidates from its own SKMX/PSIB tables in addition to the base `ranked` list, lifting deployed follow to **58/87 (66.7%)**, clearing the predeclared 60% (53/87) fidelity bar (`docs/skipmix_candidate_injection_906.md`, result_cid in `docs/skipmix_candidate_injection_906_result.json`).

Fidelity is not promotion. Per the §6-4 re-entry decision on #822, **the frozen 20‰ end-to-end causal floor remains the promotion gate**. The spot-check measured whether the deployed lane reproduces the reference arm's *favorable minimal pairs*; it did **not** measure the lane's end-to-end causal effect on deployed top-1 over the full held-out population. This issue does exactly that, and returns the S1 promotion verdict to the maintainer.

Follow-up to #897 / #904 / #906; child of #822 (S1).

## Hypothesis

The injected-candidate skip-mix lane (`predict_decision_candidates_with_skipmix`), fit + emitted through the REAL release emitter and consumed by the deployed `R4Engine`, improves deployed held-out top-1 agreement with the recorded teacher labels by **≥ 20‰** over the deployed base (no SKMX/PSIB sections), with the paired 95% lower bound clearing the floor.

The off-serving phase-0 figure (+56.2‰ vs a *toy suffix-rate bigram base*, `docs/skipmix_confirm_897_result.json`) is expected to **overestimate** the deployed delta, because the deployed base is much stronger than the toy base — the same dynamic that sank #836's reachability ceiling and #886's fidelity check. This run measures the delta against the REAL deployed base, not the toy base.

## Pinned identities

- Bundle: `.uor-models/compiled/smollm2-360m-broad-clean` (the attested #755-native #833 baseline).
- `corpus.meta` CID prefix `blake3:aa9d1767…`; train / held-out split from `induction::split_positions` (≈ 288,794 / 72,130).
- Fit: `skipmix_fit::fit_skipmix_tables(&corpus, DEFAULT_TOP_K)`; emit: `score::emit_scored_r4g1` with `ScoredGraphSections { skipmix_rows, psi_bag_rows, … }`; consume: `R4Engine::load_accepting_quality` → `predict_decision_candidates_with_skipmix`.
- Stats: `ci95_permille` / `paired_delta_permille` (normal-approx, 1.96·σ), verbatim from the phase-0 harness convention.

## Execution scope

Offline compiler/certifier fit + emit, plus the **deployed** `R4Engine` admission and prediction path. Teacher-free: the causal delta compares deployed base vs deployed skip-mix served tokens against the corpus's **recorded** `t_argmax` labels — no live teacher pass. Evidence outside this deployed scope is not credited as a deployed-serving result.

## Reachability arithmetic (five-minute gate)

The lane can only move the metric on positions where it **changes** the deployed base top-1. Define `changed = { i : base_top1(i) ≠ skip_top1(i) }` over held-out. The reachable ceiling on the causal delta is `|changed| / n` (best case: every change flips base-wrong→skip-right). If `|changed|/n < 20‰` on the deployed engine, the floor is unreachable **by arithmetic** regardless of scorer quality → do not launch the full run. This is measured on a subsample first (below).

## Binding cheap instrument + required verdict

**Reachability probe** (subsample of the held-out split, `R4_SKIPMIX_PHASE=probe`): emit base (empty SKMX/PSIB) and skip (fitted) graphs from shared recompiled sections; load both engines; replay the subsample through `predict_decision_candidates_with_skipmix` on each; report `changed`, `toward` (base-wrong→skip-right), `away` (base-right→skip-wrong), the reachability ceiling `changed/n`, and the subsample net delta `paired_delta_permille`.

**Required to proceed to the full run:** reachability ceiling ≥ 20‰ **and** subsample net-delta point estimate > 0 and consistent with clearing 20‰ at full scale. Otherwise the full run does **not** launch (AGENTS.md long-run discipline) and the verdict is recorded from the probe.

## Nulls / negative controls

- **Absent-section identity** (built-in): base engine with `predict_decision_candidates_with_skipmix` is byte-identical to plain base ⇒ delta = 0 when the tables carry no signal. The instrument *can* read 0.
- **Label-shuffle key null**: fit the SKMX/PSIB tables on a corpus whose train `t_argmax` targets are deterministically rotated (window↔target association broken, target multiset preserved), emit `graph_null`, load `engine_null`, measure its net delta vs base. A real conditioning signal ⇒ this null **collapses** to ≤ 0 (candidate injection on decorrelated tables should not help, and plausibly hurts). Analog of the phase-0 key-shuffle (−116.3‰) / conditioning-specificity nulls.

## Predeclared thresholds and distinct outcome branches

```
metric to move:       deployed held-out top-1 (paired skip − base), permille
reachability ceiling: measured on the deployed engine (probe); must be >= 20 permille to launch full
instrument + verdict: reachability probe (subsample) — ceiling >= 20 permille AND net > 0 to proceed
exit rule (PROMOTE):  full-run paired skip-vs-base 95% LOWER bound >= 20.0 permille
                      AND the label-shuffle null net delta <= 0 (does not itself clear the floor)
if positive:          recommend S1 PROMOTE to Casey; on his verdict, register RF-31
                      (ids.toml -> Gherkin -> failing test -> impl -> check-model --write) and
                      wire the lane onto the serving path
if negative:          recommend S1 REVISE/retain-dormant; lane stays dormant (no ids.toml row,
                      no CONFORMANCE.md change); record the sub-floor deployed delta honestly
cost estimate:        teacher-free; one-time build_observations on ~288k train + 2-3 graph emissions
                      + replay of ~72k held-out x 2-3 engines. Minutes-scale in release, NOT the
                      ~24h teacher-forced evaluate-report path.
```

The stage promotion verdict itself is a **maintainer decision** (#822); this run produces the evidence and a recommendation, it does not unilaterally promote.

## Evidence-record path

`docs/skipmix_endtoend_causal_<this-issue>.md` (append-only record) + `docs/skipmix_endtoend_causal_<this-issue>_result.json` (CID-bound) + a new `--ignored` harness `crates/uor-r4-api/tests/skipmix_endtoend_causal_<this-issue>.rs`.

## Conformance mapping / claim status

Binds existing capabilities on the deployed path (no new capability unless PROMOTE triggers RF-31). No `model/ids.toml` row and no `CONFORMANCE.md` change under this issue; RF-31 registration is a separate, promotion-conditional step. Claim language follows `docs/formal_vocabulary.md`; the result is an **Empirical Criterion**. A negative (sub-floor) outcome is first-class completion evidence and does not silently promote S1 or any downstream stage.


## Comments (2)

### comment 5376179273 — 2026-08-21T22:47:36Z — Casey-allard

### End-to-end deployed causal run — FLOOR CLEARED (outcome against the run contract)

Ran the reachability probe first (AGENTS.md binding cheap instrument), then the full end-to-end deployed causal run on the attested `smollm2-360m-broad-clean` bundle (`corpus.meta` CID `blake3:aa9d1767…`; train/held-out 288,794 / 72,130). **Teacher-free** — deployed base (no SKMX/PSIB) vs deployed skip-mix (`predict_decision_candidates_with_skipmix`), both scored against recorded `t_argmax`. Result reproduced byte-for-byte across two runs (`result_cid blake3:e32e4e33…`).

**Reachability probe (6,000-position subsample) — PASSED:** ceiling 549.2‰ (≫ 20‰); subsample net delta **+35.67‰ [25.76, 45.58]** (lower bound already clears the floor). Proceeded to the full run per the pre-declared gate.

**Full run (72,130 held-out positions):**

| arm | top-1 | rate (‰) |
|---|---|---|
| base (no SKMX/PSIB) | 19,372 | 268.57 [265.34, 271.81] |
| **skip (injected lane)** | **21,424** | **297.02 [293.68, 300.35]** |
| null (label-shuffle) | 2,281 | 31.62 [30.35, 32.90] |

| paired vs base | Δ (‰) | 95% CI |
|---|---|---|
| **skip − base** | **+28.45** | **[25.57, 31.32]** |
| null − base | −236.95 | [−240.33, −233.56] |

- **Floor CLEARED:** paired skip−base 95% lower bound **25.57‰ ≥ 20‰** frozen floor.
- **Null COLLAPSES:** the conditioning-specificity (label-shuffle) null lands at −236.95‰ — the signal is lane-specific, not a generic "injection helps" artifact.
- **Reachability:** the lane changes base top-1 on 39,360/72,130 (545.68‰); toward 6,651, away 4,599, neutral 28,110; net +2,052 = +28.45‰.
- As predicted, the deployed +28.45‰ is ~half the off-serving +56.2‰ (toy base) — the strong deployed base absorbs much of the toy-base delta — but the lowered lane still clears the floor.

**Recommendation: PROMOTE** — the S1 promotion verdict is a maintainer decision on #822. On a PROMOTE verdict the promotion-conditional follow-on is RF-31 registration (`model/ids.toml` → Gherkin → failing test → impl → regenerated `CONFORMANCE.md`) + wiring the lane onto the serving path.

Evidence lands via PR (harness `crates/uor-r4-api/tests/skipmix_endtoend_causal_908.rs`, record `docs/skipmix_endtoend_causal_908.md`, CID JSON `docs/skipmix_endtoend_causal_908_result.json`). The lane remains **dormant** under this issue (no `model/ids.toml` row, no `CONFORMANCE.md` change).


### comment 5406302701 — 2026-08-25T06:11:46Z — Casey-allard

## Append-only normative-scope resolution — #933 (2026-08-25)

The #908 result remains valid **reference/off-serving `R4Engine` evidence** under its original artifact and decision rule: 21,424 / 72,130 = 29.702%, with +28.45 permille [25.57, 31.32] against its matched base. The earlier use of “deployed” must not be read as proof that the ADR-designated `R4G1Runtime` selected those tokens on the production path.

#933 supplied the distinct normative evidence that #908 did not. PR #936 merged through the protected queue as `e346816cb40b089583cbd3fff2cdd84924362c65`. On its exact canonical bundle, `R4G1Runtime` records 21,293 / 72,130 = 29.5203% versus same-position TLA 20,284 / 72,130 = 28.1214%, paired +13.988 permille [11.057, 16.919]. Its same-generation sections-absent control records 18,806 / 72,130 = 26.0723%, so the normative lane effect is +34.479 permille [31.681, 37.277] and clears the unchanged +20 permille lower-bound gate.

Graph CID: `blake3:ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`
Report CID: `blake3:88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`

This RATIFY resolves RF-31 only for the exact #933 graph/report/population/`R4G1Runtime`/greedy-decode/schema-2 envelope. It does not relabel or replace the #908 row, create a universal 30% floor, or establish live-teacher parity or free-running quality.

Record: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/normative_r4g1_quality_933.md
PR: https://github.com/UOR-Foundation/uor-r4/pull/936


---
_Comment/review ids in this file: 5376179273, 5406302701_

