# #920 — docs(#843): freeze the bounded-semantic-transitions design contract; append benchmark Amendment A1

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T13:56:44Z
- **Updated:** 2026-08-22T17:39:25Z
- **Closed:** 2026-08-22T14:19:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/920
- **Merged:** 2026-08-22T14:19:03Z
- **Merge commit:** 5f4d740f25f78954094194b9b8923e98709c4c6a
- **Base / Head:** main / UOR-Foundation:issue-843-bounded-semantic-transitions
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 1 of 6 for issue #843 (item B of S4 tracker #826). **Docs only.**

Before writing a line of planner code, the binding cheap instrument that #843's own run contract requires — the coverage/reachability probe, which must be non-degenerate *before* any packed or full fitting — was run against the S4 benchmark frozen by sibling item A. **It returned DEGENERATE.** Four independent structural results, each reproducible from `uor_r4_graph_compiler::compositional_planning` at main `3a4801c4` in about 15 seconds with no fixture and no network:

1. **13 of the 20 frozen horizon cells are 0/512 solvable.** All ten H ∈ {1, 2} cells are identically zero; at H = 4 only symbolic-transformation (171/512) and multi-hop-evidence (342/512) are non-zero. The generator's own documented invariant — that instances are constructed to be solvable within `horizon` — does not hold for H ≤ 2, because the shortest gold plans are 3–8 steps.
2. **The strongest non-oracle null saturates.** A *structure-keyed* memorized-trajectory null — fit on seeds 0..256, evaluated on the disjoint seeds 256..512, generator seed excluded from the key — scores valid-plan rate **1.0000 in every non-vacuous cell**. The S4 promotion statistic (candidate − strongest non-oracle baseline) is therefore **at or below zero, by construction, for every candidate mechanism, geometric or not**, against a frozen effect floor of δ_min = 0.05.
3. **Five of six split axes have exactly one cell.** Over 4096 seeds: `by_entity` = 1, `by_vocabulary` = 1, `by_template` = 1 for all five families; `by_topology` ∈ {1,1,2,1,4}; `by_operator_composition` ∈ {3,3,2,3,7}. The no-split-leakage guarantee currently holds *vacuously* — it partitions a one-element set.
4. **`TaskInstance::id()` mixes the raw generation seed into the content hash**, so distinct ids number 4096/4096 while only 2–7 structurally distinct problems exist per family. An id-keyed memorized-trajectory null therefore scores 0.000 and reads as a healthy control, masking result 2. This is the route-attention failure mode (119 of 120 heads vacuous) recurring.

Outcome: the fit did not launch. This PR freezes the design contract for the remaining increments and records the instrument evidence plus a bounded repair, appended to the item-A specification rather than rewriting it.

## Scope and non-goals

**In scope (this PR):** three documents, no code.

- `docs/bounded_semantic_transitions_spec_843.md` (**new**, 517 lines) — the frozen design contract: the typed transition observation record including negative and conflicting examples; deterministic induction to an operator **rule set** (schematic, not a grounded state×action table) with content-addressed partitioning, ordered reductions, deduplication, declared-conflict handling, ordinal support bands, and held-out split enforcement; the fixed capacities and the typed overflow/conflict/no-plan/low-confidence decline rules; four new **optional** R4G1 sections; the P-4-only, allocation-free deployed planner on the normative `R4Engine` path with caller-owned scratch; the replayable plan-witness encoding; the equal-budget arm and null set; the three-way differential and the planted-mutation table; and the measurement protocol and run contract.
- `docs/compositional_planning_spec_844.md` — **appended section 11, Amendment A1**. Records the instrument evidence above and freezes the bounded repair: low-horizon non-vacuity (instance difficulty scales with the requested horizon), real split-axis cardinality (≥ 8 cells per axis per family), a content-derived identity with the seed excluded, and strongest-null headroom (≥ δ_min in every cell). Sections 0–10 of that document are untouched.
- `docs/r4_intelligence_completion_plan.md` — readable-mirror entry. The GitHub trackers stay the source of record.

**Non-goals:** no geometry mapping or W(3,3) qualification (item C, which stays `on-hold`); no untouched-partition verdict (item D); no free-running generation re-entry; no corpus-schema extension.

## Acceptance-criteria status

This increment satisfies none of #843's five acceptance criteria and claims none of them — it freezes the contract against which the later increments are judged, and records why the fit could not launch yet. Increment mapping: 2 benchmark repair → 3 observation and induction → 4 packed sections → 5 deployed planner → 6 measurement and verdict.

## Tests and verification

Docs-only diff (`*.md`, none of them `include_str!`'d into Rust), so the documented fast path applies and `gates-pr` runs the claim-wording gate; the merge queue re-verifies the merged content in full. Locally: `python3 scripts/check_claim_wording.py` → **passed**. The instrument itself was two `--ignored` probe harnesses run in a worktree against the crate as it stands on main; they are deliberately **not** in this diff — the shipped instruments land in increment 2 as the four A1 gate tests, where they are the gate rather than a scratch measurement.

## Evidence artifacts

Exact numbers are in `docs/compositional_planning_spec_844.md` §11.1 (per-cell solvable fractions, per-cell null saturation, per-axis cardinality) and in `docs/bounded_semantic_transitions_spec_843.md` §4.1 (reachable vocabulary: 409–545 states and 3–4 operators per family, 1083–1924 grounded transitions, union 2577 states / 8715 transitions, and a maximum observed BFS queue depth of 64 — exactly `W_MAX`, so the frozen frontier bound *binds* and the retained-frontier ordering is decisive).

## Negative and unavailable results

**The corpus-observation arm does not launch, and this is recorded as a structural result rather than an empirical `UNAVAILABLE`.** #843 asks whether corpus or instruction observations can induce useful transitions. The v4 observation record (`observation.rs`, `RECORD_SIZE = 88`) is (story token, next token, top-8 tokens, top-8 weights, span, byte anchors) plus a 16-byte probability sidecar. Of the ten typed object kinds the planner requires, it provides fields for **zero**, so coverage of the benchmark's typed state/action vocabulary is 0.000 against δ_min = 0.05. Under the long-run discipline's first gate this arm does not launch; no observation shard is read and no wall-clock is spent. Reopening condition: an observation record carrying typed state/action slots — a schema change, not a scale change.

## Compatibility and migration

No code, no format, no API change. Every guarantee and every frozen number of the item-A specification stands: δ_min = 0.05, n = 512 per held-out cell per horizon, H ∈ {1, 2, 4, 8}, H_max = 16, W_max = 64, Holm–Bonferroni, the five families, the six split axes, the six baselines, the primary metric name, and RF-32. Amendment A1 changes the generator's *diversity and identity*, not the constitution's values. Sibling item A stays in its shipped state — its deliverable landed, and this is new evidence appended to it under the append-only records rule.

## Conformance effects

None in this PR. `CONFORMANCE.md` is unchanged and no RF id is added or altered. The contract reserves **RF-33 `bounded_semantic_transitions`** (normative deployed-serving scope) for increment 5/6, to be registered **only if** an arm is actually lowered; a negative verdict registers no new deployed capability. Existing RF-01/08/12/13/27/28/32 remain reference or certifier-instrument evidence and are explicitly not deployed-planning evidence.

## Claim-boundary changes

No claim is strengthened. One claim is **weakened, deliberately**: the item-A specification's no-split-leakage guarantee is now recorded as currently vacuous, and its baseline-non-degeneracy guarantee is recorded as not implying headroom above the strongest null. Section 11.3 records the general lesson — disjointness, totality, and determinism guarantees can all pass while the instrument separates nothing — and requires a per-cell non-vacuity count and a strongest-null saturation check before any future benchmark in this programme is called frozen.

## Intentionally excluded

The two scratch probe harnesses (`probe843.rs`, `probe843b.rs`); they were measurement scaffolding and their shipped successors are increment 2's gate instruments. No generated file, model, cache, credential, or user-owned file is staged — the diff is exactly the three documents named above. The maintainer's day-branch checkout is untouched; all work is in a dedicated worktree.

## Maintainer sign-off carried in this PR

Recorded on #843 before authoring: increment shape = design freeze plus focused build PRs; observation source = probe the corpus first, then induce from the fitting split; arms = three planning arms plus four nulls; capacities = measure the fitting vocabulary and freeze at 4× headroom. On the blocker: repair the generator inside #843 as an appended amendment, with **generator only — all frozen constitution numbers stay frozen**.

References #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

