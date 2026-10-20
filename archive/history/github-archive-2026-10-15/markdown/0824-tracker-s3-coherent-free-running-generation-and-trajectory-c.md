# #824 — TRACKER/S3: coherent free-running generation and trajectory correction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:20Z
- **Updated:** 2026-08-25T22:56:30Z
- **Closed:** 2026-08-25T22:56:30Z
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/824
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Measure and close the gap between teacher-forced next-token signals and the states the deployed system visits under its own prefixes.

## Current evidence

- Teacher-forced and same-corpus replay figures are not free-running coherence evidence.
- Greedy output failure was improved by sampling, but prompt relevance and trajectory stability remain weak.
- The current observation distribution underrepresents student-generated prefixes where drift compounds.

## Entry criteria

S1 has a promoted prompt-state decision. S2 is required before public capability promotion, though the first corrective observation experiment may begin once S0 evaluation is frozen.

## Native dependencies

Blocked by #822 and #823 for stage closure. The first offline gap instrument may be prepared once S0 evaluation is frozen.

## Promotion gate

Against frozen controls, bounded corrective observation reduces prompt irrelevance, contradiction, repetition, entity/topic drift, and horizon degradation without regressing teacher-forced quality or selective risk.

## Kill / redesign criterion

If the free-running gap does not improve within the predeclared corrective-round budget, stop defaulting to more same-distribution teacher-forced records and record GENERATION-NOT-ESTABLISHED.

## Native children

- [ ] #841 — Quantify the teacher-forced/free-running gap
- [ ] #840 — Compile student-prefix corrections with equal-budget controls
- [ ] #842 — Gate trajectory state and issue the generation verdict

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Use surface validity or distinctness alone as coherence.
- Introduce unbounded online learning or inference-time teacher access.

## Repository conformance

Primary IDs: RF-14, RF-21, RF-22, RF-29. Trace capture is offline; deployed correction/state remains bounded, deterministic, P-4 compliant, and replayable through the normative scorer.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (6)

### comment 5367450851 — 2026-08-21T08:29:53Z — Casey-allard

### S3 item A complete — #841 closed: the free-running gap is quantified and it is total (PR #894, merge `11d7ca8e`)

The first offline gap instrument this tracker sanctions is frozen and run-1 is recorded (`docs/free_running_eval_841.md`, `docs/free_running_841_result.json`). Headline (greedy, deployed engine, family v1 n=100, H=32): median first-divergence step **0**; **99/100 free-running rollouts are token-identical to suffix-only rollouts** (generation is memoryless beyond the 2-token suffix); **710‰ cycle collapse**; **zero typed abstentions** anywhere. The diagnostic categories this tracker's "current evidence" section anticipated are now measured: immediate off-text divergence, memoryless generation, unguarded repetition collapse, no selective declining.

Chain state:

- **A #841:** CLOSED — contract frozen + run-1 recorded; the #840 corrective improvement bar is frozen on real numbers (median +≥2 steps AND at0 −≥100‰ per round, no TF regression >10‰, ≤3 rounds else GENERATION-NOT-ESTABLISHED).
- **B #840:** native blocker (#841) now closed — but per this tracker's entry criteria the corrective observation work is stage-gated ("S1 has a promoted prompt-state decision"), and the S1 (#822) stage verdict is pending as a maintainer call. #840 is therefore **not executable** until that verdict (or an explicit maintainer parallel-start sanction like the one that covered #841).
- **C #842:** blocked on #840.

**Executable-frontier note for the maintainer:** with #841 closed, the roadmap's executable frontier is exhausted pending decisions that are yours: the S1 (#822) and S2 (#823) stage verdicts (evidence sets complete on both trackers), #839's routing (its activation condition did not occur — retire/re-scope/hold), and whether #840 may start ahead of the S1 verdict. F0 (#859, Lean prime-router kernel) remains the open parallel formal lane.


### comment 5377617875 — 2026-08-22T03:28:38Z — Casey-allard

## S3 stage verdict — `LIMIT` (2026-08-22)

Recording the S3 tracker verdict over the three completed native children. This comment is the source of record; the readable mirror lands in `docs/r4_intelligence_completion_plan.md` via PR #914 (this issue closes on that merge).

**Verdict: `LIMIT`.** Coherent free-running generation is **not established** at the current representation scope; the deployed artifact is a teacher-forced retrieval/continuation system, not a generative engine at this scope. `PROMOTE` is ruled out — the promotion gate (bounded corrective observation reduces the free-running failures without regressing teacher-forced quality or selective risk) is not met, and the stage kill/redesign criterion is met. This is a limiting (non-promotion) verdict: it narrows the claim and promotes nothing downstream.

**Evidence (all three children resolved):**
- **#841 (A) — gap quantified and total** (PR #894, `11d7ca8e`): median first-divergence step 0; 99/100 free-running rollouts token-identical to suffix-only (memoryless beyond the 2-token suffix); 710‰ cycle collapse; 0‰ typed abstentions. The corrective §6 bar is frozen on these numbers.
- **#840 (B) — `GENERATION-NOT-ESTABLISHED` (do-not-launch)** (PR #912, `35f8cc87`): correctable footprint ~1‰ vs the required 100‰ diverged-at-0 drop; the promoted skip-mix lane (RF-31) regresses free-running (suffix-locality 99→19 of 100). Limiting factor = representation.
- **#842 (C) — `NOT TRIGGERED` / `GENERATION-NOT-ESTABLISHED`** (PR #913, `abcad4e5`): a predeclared non-vacuous diagnostic classifies every failure (survived 0 · single-step-at-0 59 · candidate-gap 16 · rank-limit 25 · state-starvation 0); 0/100 failures addressable by bounded trajectory state (state-addressable 0‰; `ceiling_clears_bar = false`).

**Global falsifier:** bounded student-prefix correction (#840) and bounded trajectory state (#842) both fail to reduce the frozen free-running gap → teacher-forced retrieval/continuation system, not a generative engine at this scope.

**Conformance:** no runtime / format / compiler change across the chain (RF-14/21/22/29; `CONFORMANCE.md` unchanged).

**Downstream:** closing this tracker clears its native-blocker edge to S4 (#826), but S3 is `LIMIT`, not `PROMOTE`, and #826 requires promoted S2 and S3 verdicts; with S2 (#823) at REVISE (its #839 phase-2 trigger-gated) and S3 limited, **S4 remains gated**. Generation re-entry requires a representation carrying cross-step free-running memory, re-measured under the unchanged #841 §6 bar and the frozen #838 selective-prediction gates.


### comment 5416375065 — 2026-08-25T20:31:34Z — Casey-allard

_edited 2026-08-25T20:32:11Z_

S3 re-entry opened as #942. This does not revise the historical LIMIT verdict. The bounded first slice wires full admitted prompt + committed generated-prefix memory into the existing normative session-signature lane; the frozen #841 bar remains unchanged and no coherence claim is made by implementation alone.

### comment 5417131613 — 2026-08-25T21:35:53Z — Casey-allard

## S3 re-entry closure — LIMIT unchanged (2026-08-25)\n\nAll native S3 children are resolved, including the bounded representation re-entry:\n\n- #942 merged as b8f78786eb8f7697b4419742008375a58fe8767b and established structural reachability: the complete admitted prompt plus committed generated prefix now reaches every normative R4G1 production decision through a fixed-size incremental session signature.\n- #944 merged as cf22879a35c896ece3df886ccdaca8716d4b3ee6 and issued the required non-vacuity verdict against the exact admitted #933 schema-2 artifact: INERT.\n- Exact #944 result: 512 positions inspected; 492 explicit context-row resolutions; 20/20 primary context probes admitted; 0 primary misses; 0 secondary session-probe attempts/admissions; 0 candidate-list changes; 0 served-token changes.\n- Tested-position CID: blake3:f108e9caab5edc0b7b4d312cf5ae187608d9653ea3d4e24bfa00ef68a07507b7\n- Observation CID: blake3:c6876af09d4c416f10f7098b18eda94cf8e6e4217e9cf28bb66a73f8f86748fe\n\nVerdict: LIMIT remains unchanged. The implementation proves cross-step memory reachability, but the admitted compiled representation makes that lane behaviorally inert because primary routing never yields to it. Coherent free-running generation remains NOT ESTABLISHED, PROMOTE remains ruled out, and the broader #841 protocol remains NOT_RUN under the predeclared #944 contract.\n\nThe readable evidence mirror is already on main in docs/r4_intelligence_completion_plan.md and the #944 record. No additional test or certification run is warranted for this tracker-only closure.\n\nAny future S3 re-entry must first compile signature-addressable state or explicitly revise the routing composition under equal-budget controls, then repeat the bounded #944 canary before spending the frozen #841 protocol.\n\nClosing #824 as completed with an evidence-backed LIMIT verdict.

### comment 5417193463 — 2026-08-25T21:41:26Z — Casey-allard

S3 engineering re-entry is now tracked as #946. The historical LIMIT verdict remains unchanged; #824 is reopened only to own this bounded compiler/runtime leaf and its eventual evidence-backed result.

### comment 5418005637 — 2026-08-25T22:56:29Z — Casey-allard

## S3 engineering re-entry closure — LIMIT unchanged

The bounded compiler/runtime re-entry #946 is complete and merged through PR #947 at `43edd2de1639651a66d94e75c717a0e6d827136b`.

The representation defect was implemented away: full-prefix trajectory signatures are compiled into explicit admitted R4G1 regions, and production reached the trajectory lane in all 20 of the 512-case canary fallback positions. The behavioral exit rule remained negative: earlier-prefix changes produced 0 candidate-list changes and 0 served-token changes. Product verdict: `INERT`.

The historical S3 `LIMIT` verdict therefore remains unchanged. Coherent free-running generation is still NOT ESTABLISHED, #841 remains NOT_RUN by contract, and nothing is promoted downstream. The readable record is `docs/trajectory_routing_946.md` with its bound product JSON on main.

All native and re-entry work owned by this tracker now has a completed evidence-backed verdict. Closing #824 again as completed with `LIMIT` unchanged.


---
_Comment/review ids in this file: 5367450851, 5377617875, 5416375065, 5417131613, 5417193463, 5418005637_

