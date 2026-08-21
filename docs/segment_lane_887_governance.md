# #887 — governance verdict: the 20‰ causal floor for the segment re-ranking arm class

**Scope.** Governance / pre-registration decision. S1 follow-up B of #836
(verdict REVISE, lane dormant); follows the #886 bounded spot-check; parent
programme #822. This record answers #887's question — whether the frozen 20‰
`CAUSAL_FLOOR_PERMILLE` is the correct promotion bar for the bounded segment
re-ranking lane — as an explicit, documented governance decision. It changes no
floor, no serving path, no `model/ids.toml`, and no `CONFORMANCE.md`. Records in
`docs/` are appended to, not rewritten.

## 1. The pre-registered bar and the arm it was set for

- **Pre-registered floor (frozen before any arm was built):**
  `CAUSAL_FLOOR_PERMILLE = 20.0‰` (`crates/uor-r4-api/tests/causal_prompt_run_834.rs`,
  `psi_arm_run_834.rs`).
- **#822 promotion gate (verbatim):** "A selected bounded state/residual arm
  improves predeclared document-disjoint, EXCT-disabled causal metrics with a
  positive confidence-bound margin on two domains; gains survive prompt swaps and
  paraphrases, improve relevance rather than only diversity, and meet runtime
  budgets."
- The 20‰ floor is the pre-registered instantiation of that gate's "positive
  confidence-bound margin."

## 2. Question 1 — is 20‰ calibrated for a re-ranking arm, or was it set for a stronger class?

**Finding: the segment lane is the pre-registered arm class; the bar was set for it.**

- The arm that produced the whole-prompt signal is the #834 §6.2 **Ψ reference
  arm** — a learned content-token→teacher-argmax **residual** table accumulated
  over a persistent prompt-content **segment session**. Its own #834 record
  classifies it "SELECT: **persistent-state** (segment lane)"
  (`docs/psi_arm_834_result.json`). That is squarely the "bounded
  **state/residual** arm" of #822's gate — not a weaker, out-of-class mechanism
  to which a stronger arm's bar was mistakenly applied.
- The "re-ranking" descriptor names the lane's **deployed realization** (the
  residual only reorders the served top-8 candidates), not a different arm class.
  The mechanism under test — a persistent-state residual — is exactly the class
  the 20‰ bar was pre-registered for.
- Therefore there is no arm-class mismatch that would, on its own, justify a
  different bar. The bar is calibrated for this arm.

## 3. Question 2 — does the segment lane fail the gate?

**Finding: yes, decisively, on two independent lines of evidence.**

**(a) Reachability — the ceiling is below the floor, unconditionally (#836).**
The reference arm's own **optimistic** whole-prompt effect is **+17.5‰, 95% CI
[+15.9, +19.0]** over the suffix baseline (suffix 246.6‰ → Ψ 264.1‰;
`docs/psi_arm_834_result.json`), measured **off** the serving path,
**unquantized**, at **full-vocab** top-64/key. Its CI upper bound **19.0‰ <
20.0‰**. The deployed packed lane is strictly more constrained (integer-`ScoreQ`
quantization; the top-8 candidate bound), so its reachable ceiling is **≤ 19.0‰ <
20.0‰**. No in-mechanism change — candidate-support widening included — can lift
the ceiling above the reference arm's own unquantized full-vocab ceiling. The
conclusion is arithmetic, not a run outcome (`docs/segment_lane_836_verdict.md`,
`segment_lane_836_result.json`).

**(b) Deployed fidelity — the lane does not even track its own ceiling (#886).**
The bounded spot-check on the real #833 bundle found the deployed lane follows
only **1/10** of the reference arm's favorable minimal pairs (robust **1/6**
among the six pairs where both sides served), below the pre-registered **≥6/10**
"faithful" bar — a **LOWERING-FIDELITY GAP** (`docs/segment_lane_886_spotcheck.md`,
`segment_lane_886_result.json`). Consistent with it, the deployed paired causal
delta is **−1.6‰, CI [−2.4, −0.8]** (suffix-local; full context does not beat the
suffix on the serving path), minimal-pairs follow **0/1460**
(`docs/segment_lane_836_result.json` / `causal_run_834_result.json`).

The #822 gate additionally requires a two-domain, prompt-swap-surviving positive
margin that improves relevance rather than diversity. The deployed lane fails the
single-metric margin outright (negative delta), so the multi-domain and
prompt-swap conditions are moot.

## 4. Question 3 — should the bar differ by arm class, and who signs off?

**Not exercised — and no live path to a PROMOTE exists under any
evidence-consistent bar.**

- §2 establishes the segment lane **is** the pre-registered arm class, so there
  is no "different class" to which a different bar would attach.
- Even a hypothetically lower bar cannot rescue the **deployed** lane: it does not
  track its own 19‰ ceiling (1/10) and its deployed causal effect is **negative**
  (−1.6‰). A bar low enough to admit a negative-effect lane would not be a
  causal-influence bar at all.
- Moving or re-classing the floor now — after the arm has been measured — to admit
  this lane would be exactly the post-hoc floor-move the pre-registration exists
  to prevent ("Do not lower the floor to manufacture a PROMOTE," #887).
- Any future arm-class-differentiated bar remains a **maintainer**
  pre-registration decision, frozen **before** re-measurement with a named signer.
  This record neither creates nor forecloses that option; it declines to exercise
  it, there being no evidence-consistent bar under which the segment lane
  promotes.

## 5. Decision

**The 20‰ `CAUSAL_FLOOR_PERMILLE` STANDS for the bounded state/residual (segment
re-ranking) arm class.** The segment lane **fails** the promotion gate and is
**retired from the promotion track**; it remains **dormant / ledgered as an
evidence-backed negative**. Unchanged, as under #836: **no `model/ids.toml`
serving row, `CONFORMANCE.md` unchanged, no public quality claim.** The activation
gate is unchanged: *re-clears the 20‰ causal floor on the packed serving path* —
which the reachability arithmetic (§3a) shows the in-mechanism segment lane cannot
do.

This is the pre-declared default outcome of #887 ("the bar stands; the lane stays
dormant"), reached with #886's number in hand.

## 6. Consequence for #888 (candidate-support widening) — closed off

#888 ("widen the segment lane candidate support beyond top-8") is **closed off**
by this verdict. #888's own gate says widening is worth building "only if #887
concludes the promotion bar … is below ~19‰ (or reframed by arm class) … and #886
shows the deployed lane faithfully tracks the reference." Both gates **fail**: the
bar stands at 20‰ for this arm class (§5) and #886 recorded a lowering-fidelity
gap (§3b). Per #888's own text, "prefer retiring the lane (dormant/ledgered) over
building this." #888 should be closed as **not planned**, superseded by this
verdict. The emission-path non-equivalence #886 recorded — the lane's only PSTATE
emitter, `convert_with_segment_table`, is not serving-equivalent to the released
`graph/score.r4g1` cover/score emitter on held-out data — is a further reason
widening the deployed lane would not, by itself, produce a released-graph
promotion.

## 7. Scope for the S1 stage verdict (not decided here)

This record resolves the **segment-lane** promotion question (the #836 follow-up
lane of #822). It is **not** the S1 stage verdict: #822 closes only after every
native child (#835 ✓, #834, #836 ✓) has a final verdict, and **#834 remains open**
(maintainer-parked, holding an interim negative pending further Ψ-family arm
evaluation). The #822 kill/redesign criterion — "if **two** independently
motivated arms fail causal controls, return to the representation/compiler model"
— is **not** triggered by this single arm's retirement alone. The stage-level
`PROMOTE / REVISE / LIMIT / RETIRE` decision on #822 remains the maintainer's,
against the full child set.

## Execution scope & conformance

Documentation / governance. No serving-path change, no `model/ids.toml`, no
`CONFORMANCE.md` edit, no code. Any future accepted bar change would be a new
pre-registered artifact under `docs/`, referenced from the promotion gate. Primary
IDs: RF-01, RF-27, RF-28.

## Definition of done

This committed decision record (the 20‰ bar stands for the segment re-ranking arm
class; the lane is retired to dormant) plus the #822 tracker update recording the
governance verdict, and the closing-off of #888. Closes #887.
