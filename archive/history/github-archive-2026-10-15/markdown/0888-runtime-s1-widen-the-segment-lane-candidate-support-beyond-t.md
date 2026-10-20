# #888 — runtime(S1): widen the segment lane candidate support beyond top-8 (follow-up C of #836)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T01:34:12Z
- **Updated:** 2026-08-21T04:00:25Z
- **Closed:** 2026-08-21T04:00:25Z
- **Labels:** enhancement, graph-compiler, conformance
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/888
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #822 · Blocked by: #887 (and evidence from #886) · Context: #836

## Objective

Lift the **top-8 candidate bound** the deployed segment lane re-ranks within
(`STEP_TOP_CANDIDATES`) toward the reference arm's full-vocab argmax — the
candidate-support-expansion arm named in #834 — so the *deployed* whole-prompt
effect can approach its ceiling instead of being clipped by an 8-wide window.

## Why this is last, and gated

#836's 4c-iii verdict identifies two constraints holding the deployed lane below
the reference ceiling: (1) integer-`ScoreQ` quantization and (2) the top-8
candidate bound. Widening addresses (2). **But note the ceiling itself (19.0‰) is
still below the 20‰ floor**, so widening alone cannot produce a PROMOTE — it can
only move the deployed effect toward the ceiling. Therefore this work is worth
building **only if**:

- **#887** concludes the promotion bar for a re-ranking lane is below ~19‰ (or
  reframed by arm class) — otherwise widening cannot change the dormant verdict;
  and
- **#886** shows the deployed lane faithfully tracks the reference on its
  favorable cases (so widening the window is the actual bottleneck, not a
  lowering-fidelity bug).

If either gate fails, prefer retiring the lane (dormant/ledgered) over building
this.

## Scope

- Introduce a bounded, P-4-safe way for the segment re-rank to consider
  candidates beyond the served top-8 (candidate-support expansion), without
  breaking the never-fabricate-a-token and never-turn-serve-into-abstention
  invariants, and without unbounded allocation or vocab-wide scans on the hot
  path.
- Re-measure via the #886 bounded check first; only then consider a full causal
  run **and only if** the (possibly revised) floor is now reachable — post the
  run contract before any expensive execution.

## Execution scope & conformance

Serving-path (`uor-r4-api` engine + `uor-r4-graph-runtime` state) and possibly
format/compiler if the widened candidate set needs new packed data. Preserve
P-4 (no multiply/divide/float on the normative hot path), determinism, zero
allocation, no_std where applicable, the sanctioned error surface, and witness
attribution. The lane remains dormant until a promotion gate is actually cleared.
Primary IDs: RF-21, RF-22, RF-27, RF-28.

## Definition of done

A widened-candidate re-rank behind the same optional PSTATE gate with full gate
ladder green and absent-section identity preserved, a bounded re-measurement,
and — only if a pre-registered floor is genuinely cleared on two domains — a
promotion proposal. Otherwise, a recorded negative and the lane retired to
dormant.


## Comments (1)

### comment 5365023269 — 2026-08-21T04:00:24Z — Casey-allard

Closing as **not planned** — closed off by the #887 governance verdict (`docs/segment_lane_887_governance.md`, merged `7bb8b4ec`, PR #890).

This issue's own build gate requires that "**#887** concludes the promotion bar … is below ~19‰ (or reframed by arm class)" **and** that "**#886** shows the deployed lane faithfully tracks the reference." Both fail:

- #887 concluded the **20‰ bar stands** for this arm class (the segment lane *is* the pre-registered bounded state/residual arm, not an out-of-class re-ranker), and the reference ceiling (19.0‰) is below the floor **unconditionally** — so widening cannot produce a PROMOTE, only move the deployed effect toward a sub-floor ceiling.
- #886 recorded a **lowering-fidelity gap** (deployed lane follows 1/10 < 6/10), so the top-8 window is not the actual bottleneck — the deployed lowering does not yet track its own ceiling. Widening would sit atop a lowering that does not track its ceiling.

Per this issue's own text, "prefer retiring the lane (dormant/ledgered) over building this." The segment lane stays dormant. **Reopen** if a maintainer pre-registers a differentiated re-ranking bar (frozen *before* any re-measurement, with a named signer) **or** the #886 lowering-fidelity gap is first resolved so widening is the actual lever.



---
_Comment/review ids in this file: 5365023269_

