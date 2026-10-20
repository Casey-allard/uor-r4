# #64 — Scoring redesign: correlated residual stacking across overlapping memberships

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T22:41:12Z
- **Updated:** 2026-07-23T00:48:56Z
- **Closed:** 2026-07-23T00:48:56Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/64
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

M.V.G. checkpoint blocker, measured 2026-07-22 by the Phase-4 Gate C harness (reproduced twice, fixture corpus, 30,036 held-out positions):

| scorer | top-1 teacher-argmax | bits/token |
|---|---|---|
| graph (no EXCT) | 0.3% | 70.47 |
| graph (with EXCT) | 0.3% | 76.96 |
| TLA3 store baseline | 31.7% | 11.88 |

Mechanism (instrumented): the literal Σ-over-cloud scoring formula S(v) = B(v) + ΣΔE + ΣΔT + ΔX double-counts correlated evidence — one token receives +91.7 nats from 13 sibling-subtree emission lists and wins 98% of 2,000 probed positions while the teacher's argmax sits ≈ −72 nats. An A-only ablation (F emptied) is also ~0.2%, so the stacking is not F-specific. Root-plus-residual decomposition prevents duplication along ancestry chains but NOT correlation across sibling regions sharing a refinement parent when top-M (M=3) overlapping memberships are active. The PDF's 'double counting' risk is confirmed, and the plan's risk register now records it as such.

Redesign candidates (pick one or a combination at the M.V.G. checkpoint):
1. Chain-telescoped residual application: apply ΔE only along the refinement chain from root to each active region's deepest covered ancestor (no sibling fan-out), so refinement corrections compose instead of stack.
2. Cloud-size normalization: normalize the emission sum by the number of contributing regions (or weight by membership margin/support).
3. Bound or drop F emissions: predicted-cloud ΔT contributes little if A-only ablation is equally bad — measure whether F helps at all before keeping it.
4. D4 EXCT precedence: exact-context evidence outranks compressed residuals (consistent with the D4 fallback policy) — EXCT's confident-prefix signal (~−0.1 nats, which drives the baseline's 31.7%) currently gets swamped by the stacked sum.

Context: this must be resolved BEFORE the Phase-5 runtime makes the scorer the default; targets 1–2 of the confirmed M.V.G. contract (BASELINE.md §4) fail by a wide margin under the current formula. Implementation: crates/uor-r4-core/src/transformerless/{score.rs, score_runtime.rs}; harness: r4 transformerless score. Plan: docs/r4_graph_compiler_implementation_plan.md §10 risk register.

## Comments (1)

### comment 5053124696 — 2026-07-23T00:48:56Z — auser

RESOLVED via chain-telescoped residual application (Rule 1) + EXCT precedence (Rule 2), pushed as 94501bd.

Full 4-way Gate C (fixture corpus, 30,036 held-out; reproduced by maintainer agent, byte-identical artifact):

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old formula) | 0.3% | 76.96 |
| graph chain-telescoped (Rule 1) | 1.2% | 71.52 |
| graph chain+EXCT precedence (Rule 1+2) | 31.7% | 9.86 |
| TLA3 store baseline | 31.7% | 11.88 |

- Rule 1+2 is argmax-IDENTICAL to the baseline on all 30,036 positions (+0/-0 win/loss) with better bits/token (9.86 < 11.88). The deployed quality gate (rule12_precedence >= baseline) passes exactly at threshold.
- Witness replay 64/64; Theorems 6/10 preserved (replay rejects duplicate contribution IDs; sibling stacking is impossible by construction).
- Honest remainder: Rule 1 alone (graph residuals, no EXCT) is still 1.2% — the exact-context signal carries the recovery; residual value-add over exact-context lookup is the open quality question, now tracked in the chain (#66 F-ablation, #68 residualized EXCT (largely delivered as RX1 in this work), #70 cover fineness, #71 hidden-state lane).
- Statuses: ExactContext 30036/30036 on this corpus (chain/Novel paths unit-tested instead).
- M.V.G. contract (BASELINE.md §4): floor MET, bits MET, artifact barely MET (10.8 MB, RX1 EXCT dominates - flag for Phase 5), fallback 0%. The +5pp stretch target remains open.

Plan risk-register row updated with the outcome; GLOSSARY scoring-model entry updated; index.html and src/r4g1.rs consumers migrated to the schema-2 report fields.


---
_Comment/review ids in this file: 5053124696_

