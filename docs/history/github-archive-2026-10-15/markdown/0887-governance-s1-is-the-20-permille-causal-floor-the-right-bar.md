# #887 — governance(S1): is the 20-permille causal floor the right bar for a re-ranking lane? (follow-up B of #836)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T01:33:46Z
- **Updated:** 2026-08-21T04:00:22Z
- **Closed:** 2026-08-21T03:58:22Z
- **Labels:** question, conformance
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/887
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #822 · Follows: #886 · Context: #836 (4c verdict: REVISE)

## Objective

Decide, as a **governance / pre-registration** question, whether the frozen
**20‰ causal floor** (`CAUSAL_FLOOR_PERMILLE`) is the correct promotion bar for a
bounded **re-ranking** lane whose honest measured ceiling is ~**+17.5‰** (CI
upper 19.0‰). This is the pivotal gate: #836's arithmetic shows that **no**
in-mechanism change to the segment lane can clear 20‰, because 19.0‰ is the
reference arm's own optimistic ceiling.

## Why this, and why after #886

- The 20‰ floor was pre-registered before any arm was built (correctly — it must
  not be moved post hoc to force a pass). Revisiting it is legitimate **only** as
  an explicit, documented governance decision, not as a quiet threshold edit.
- #886 supplies the missing evidence: whether the deployed lane actually
  realizes its ~19‰ ceiling on the real bundle. A governance decision about the
  bar should be made with that number in hand, not before.

## Questions to resolve (and record)

1. Is 20‰ calibrated for a *candidate-support / re-ranking* arm, or was it set
   for a stronger class of prompt-conditioning arm (persistent-state /
   conditional-residual)? #822's promotion gate speaks of "a selected bounded
   state/residual arm"; is the segment re-ranking lane in that class at all?
2. If the bar is right, does the segment lane simply **fail** the gate (→ retire
   it from any promotion track, keep it dormant/ledgered as a negative result)?
3. If the bar should differ **by arm class**, what is the pre-registered,
   two-domain, prompt-swap-surviving criterion for a re-ranking lane, and who
   signs off? Any new bar is frozen **before** re-measurement, exactly as the
   original was.

## Discipline / non-goals

- **Do not** lower the floor to manufacture a PROMOTE. The default outcome of
  this issue is "the bar stands; the lane stays dormant."
- No code change lands from this issue by itself; its product is a recorded
  governance decision that gates #836's candidate-support follow-up.

## Execution scope & conformance

Documentation / governance. No serving-path change, no `model/ids.toml`, no
`CONFORMANCE.md` edit. Any accepted bar change is a new pre-registered artifact
under `docs/`, referenced from the promotion gate. Primary IDs: RF-01, RF-27,
RF-28.

## Definition of done

A committed decision record stating whether 20‰ stands for this arm class (and
if not, the new pre-registered criterion), and a #822 update. This unblocks —
or closes off — the candidate-support widening follow-up.


## Comments (1)

### comment 5365023056 — 2026-08-21T04:00:22Z — Casey-allard

✅ **Completed — governance verdict recorded and merged.**

**Verdict:** the pre-registered 20‰ `CAUSAL_FLOOR_PERMILLE` **stands** for the bounded state/residual (segment re-ranking) arm class. The segment lane **fails** the #822 promotion gate and is **retired from the promotion track**, remaining dormant / ledgered as an evidence-backed negative. This is the pre-declared default outcome ("the bar stands; the lane stays dormant"), reached with #886's number in hand.

**Two independent lines of evidence:**
- **Reachability (#836):** reference-arm ceiling +17.5‰, CI upper **19.0‰ < 20‰**, unconditional — no in-mechanism change (candidate-support widening included) can lift it above the reference arm's own unquantized full-vocab ceiling.
- **Deployed fidelity (#886):** deployed lane follows **1/10 < 6/10** faithful bar (robust 1/6); deployed causal delta **−1.6‰**, CI [−2.4, −0.8] (suffix-local; full context does not beat the suffix on the serving path).

**Arm-class finding:** the segment lane *is* the pre-registered "bounded state/residual arm" (its #834 §6.2 reference is classed "SELECT: persistent-state"), not a weaker out-of-class re-ranker to which the wrong bar was applied — so the bar was calibrated for it. A differentiated bar (option 3) is declined: even a lower bar cannot admit a lane whose deployed effect is negative, and moving the floor post-hoc is exactly what the pre-registration prevents.

**Decision record:** `docs/segment_lane_887_governance.md`
**Merge commit:** `7bb8b4ecbd441ee7018185a0d2e73a1216a6927f` (PR #890)
**Scope/conformance:** documentation/governance only — no serving path, no `model/ids.toml`, no `CONFORMANCE.md`. RF-01, RF-27, RF-28.
**Consequence:** #888 (candidate-support widening) is closed off (record §6). The S1 stage verdict on #822 is unchanged and remains the maintainer's, with #834 still open.



---
_Comment/review ids in this file: 5365023056_

