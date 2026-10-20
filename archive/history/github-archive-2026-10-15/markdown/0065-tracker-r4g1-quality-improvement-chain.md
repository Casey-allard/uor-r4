# #65 — TRACKER: R4G1 quality-improvement chain

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:12Z
- **Updated:** 2026-07-23T15:46:21Z
- **Closed:** 2026-07-23T15:46:21Z
- **Labels:** graph-compiler
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/65
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Single map of every known way to fix and improve graph output quality, in dependency order. Root context: Gate C measured formula v1 at 0.3% vs 31.7% TLA3 baseline (2026-07-22); the deployed quality gate in src/r4g1.rs rejects graphs scoring below baseline (no-EXCT). Confirmed M.V.G. targets: docs/transformerless/BASELINE.md §4.

Chain 1 — unblock the scorer (Phase 4, now):
1. #64 chain-telescoped + EXCT precedence redesign (IN FLIGHT — everything else here depends on it)
2. F-emissions ablation decision (ΔT value test)
3. Emission smoothing calibration (add-one vs Witten-Bell vs absolute discounting)
4. Residualized EXCT for the deployed path (removes probe-time float; unblocks exact-context evidence within the integer contract)
5. Interaction residuals for explicit overlap nodes (PDF's original non-duplication mechanism; needs overlap nodes in the cover)

Chain 2 — better substrate (Phase 2/3):
6. Cover fineness sweep (region budget vs recall/bytes under the fixed formula)
7. Hidden-state trace lane for cover induction (richer representations than bundle geometry; observe.rs seam)
8. Anti-degeneracy objectives in cover scoring (J(C): invariance, reuse, MDL — Phase 3)

Chain 3 — better evidence (data):
9. D3 natural-corpus observation run (Simple English Wikipedia, CC BY-SA 4.0, CID-pinned manifest)
10. Larger/better teacher evaluation (pinned-revision swap beyond 135M; quality ceiling of the teacher bounds the graph)
11. Richer teacher evidence (top-k > 3 logits in observation records)

Chain 4 — measurement integrity:
12. HF-path (SmolLM2) full Gate C on the D3 distribution (the M.V.G. judge is currently fixture-corpus only)
13. Canonical bits/token definition (11.88 scorer accounting vs 6.54 P2 WB — one metric, documented)
14. CI Gate C trend tracking (4-way table per commit, regression alarms)

Chain 5 — deployed behavior (Phase 5):
15. ResolutionStatus-driven behavior in the deployed path (Novel/Contradictory → manifest policy: widen, EXCT, abstain)
16. Repetition control in the graph scoring path (legacy recency penalty has no graph equivalent yet)

Each issue body lists its dependencies and acceptance checks. Close this tracker when every linked issue is closed or formally descoped.


---

## Definition of Done
- Every linked issue (#66–#80) is closed — either completed with its acceptance criteria met, or formally descoped with the measured evidence recorded.
- A final quality summary is posted here: the 4-way Gate C table at chain end vs the 0.3% starting point, per the M.V.G. targets in `docs/transformerless/BASELINE.md` §4.

## Acceptance criteria
- [ ] All 15 linked issues are closed with evidence in their threads.
- [ ] The summary table shows graph no-EXCT top-1 agreement and bits/token vs the TLA3 baseline at the end of the chain.
- [ ] Every M.V.G. target row in BASELINE.md §4 has a measured value and a plain pass/fail verdict.

## Comments (1)

### comment 5060444954 — 2026-07-23T15:46:20Z — auser

# Final summary — R4G1 quality-improvement chain (#65)

Closing per the tracker's Definition of Done: all 15 linked issues resolved with evidence, and this is the end-of-chain quality summary against the starting point.

## Starting point vs end point (fixture corpus, 30,036 held-out positions, measured 2026-07-23)

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-over-cloud (chain start, formula v1) | 0.3% | 76.9643 |
| graph chain+EXCT precedence (chain end, Rule 1+2) | 31.7% | 9.8612 |
| TLA3 store baseline | 31.7% | 11.8781 |

Chain-end Rule 1+2 is argmax-identical to the baseline on all 30,036 positions (+0/−0 win/loss) with better bits/token (9.86 < 11.88). Witness replay 64/64. Artifact byte-identical across runs (κ de04eec8…). Deployed quality gate (src/r4g1.rs) passes at threshold legitimately.

## M.V.G. contract state (BASELINE.md §4, confirmed 2026-07-22)

- Target 1 (agreement ≥ baseline+5pp stretch; ≥ baseline floor): floor PASS (identical everywhere, D3 natural 15.04% / continuity 14.76% included), stretch FAIL — and now understood structurally: under EXCT precedence the graph is argmax-identical to the baseline on every measured corpus, so agreement-over-baseline can only tie. Reopening needs a reframed stretch (bits-margin or residual-value), recorded on #73.
- Target 2 (bits ≤ baseline − 0.3): PASS on fixture (−2.02) and both D3 partitions (−5.46 natural, −6.01 continuity).
- Target 3 (artifact ≤ 2× current): PASS (10.87 MB incl. RX1 EXCT).
- Target 4 (zero allocations/step): PASS (allocation census, incl. the #78 status path).
- Target 5 (latency ≤ 2×): not yet measured — carried to the M.V.G. checkpoint as open evidence debt.
- Target 6 (fallback < 20%): PASS (0% Novel on all measured corpora).

## Outcomes by chain

**Chain 1 — unblock the scorer (root cause found and fixed):**
- #64 chain-telescoped + EXCT precedence redesign: the double-counting mechanism (correlated sibling-subtree stacking) eliminated by construction; old formula retained as score_candidates_legacy for the comparison table. CLOSED with the 4-way table.
- #66 F-emissions ablation: ΔT dropped — the folded offset is argmax-neutral by construction and cost 4× distribution quality (71.52 → 17.73 bits without it). CLOSED.
- #67 smoothing calibration: add-one kept decisively (9.8612 vs 14.18 next-best; agreement tied). CLOSED.
- #68 residualized EXCT (RX1): compile-time-baked residuals, integer-only deployed read path, parity with probe-time quantization machine-tested; deployed path re-enabled. CLOSED.

**Chain 2 — substrate:**
- #70 cover fineness sweep: 42-region default confirmed adequate for the scoring metric (Gate C bit-identical at all 9 grid points); metric-conditional recall note recorded (+14.1pp d3 recall for +11 kB at 54 regions if recall ever becomes the objective). CLOSED.
- #71 hidden-state trace lane: landed via PR #95. CLOSED.

**Chain 3 — evidence:**
- #72 D3 natural-corpus run: Simple English Wikipedia partition made real (3,000 articles sealed, CC BY-SA 4.0, CID-pinned manifest, from-text observation driver). CLOSED.
- #73 larger-teacher ceiling: DESCOPED with the structural argument — a bigger teacher raises the floor for every scorer equally but cannot produce agreement margin while the graph is argmax-identical to the baseline. Reopens when the stretch clause is reframed. CLOSED.
- #74 richer teacher evidence (top-k>3): CLOSED by evaluation.

**Chain 4 — measurement integrity:**
- #75 HF-path full Gate C on D3: the M.V.G. judge ran on both partitions (natural 400-article slice: 17,457 held-out; continuity: 40,342 held-out) with full 4-way tables and §4 verdicts. CLOSED.
- #76 canonical bits/token definition: one formula (mean cross-entropy of the true next token), comparability rule, historical families annotated. CLOSED.
- #77 CI Gate C trend tracking: pinned record + alarm in CI (top-1 drop > 2pp or bits regression > 0.1 fails the run). CLOSED.

**Chain 5 — deployed behavior:**
- #78 ResolutionStatus-driven behavior: policy-as-data (Novel → widen-once-then-abstain), status/widened/abstained in API, zero-allocation path. CLOSED.
- #79 bounded integer-only repetition control: landed via PR #92. CLOSED.
- #80 overlap alternatives: closed as evaluated, trigger not fired — 100% of positions on every measured corpus are ExactContext status; no Graph/Novel volume exists to fail the trigger. Reopen condition recorded.

**Chain 1 remainder:**
- #69 interaction residuals: EVALUATED, NOT JUSTIFIED — all 16 strongest co-activation pairs degrade held-out scoring (Rule 1: +0.64…+2.25 bits/token; Rule 1+2: +0.02…+0.29 bits and up to −0.53pp agreement, breaking the EXCT argmax-identity). No candidate clears the ≥0.05 bits/token adoption threshold. Pinned harness: `crates/uor-r4-core/tests/interaction_residuals.rs` (PR #101). Note: #99's zero-delta result was produced by a metric the review flagged as fake; cleanup tracked in #102. CLOSED.
- #81 (duplicate of #64's finished work): CLOSED.

## Honest remainder (carried to the M.V.G. checkpoint)

- Rule 1 (graph residuals alone, no EXCT) is still weak: 1.2% fixture, 0.19% natural. The scorer today is exact-context precedence plus structure; residual value-add over exact lookup is the open quality question the next cycle (cover v2, hidden-state lane productization, richer evidence) addresses.
- Latency target (§4.5) unmeasured.
- Full 3,000-article D3 run (vs the 400-article declared slice) is the follow-up that gives the checkpoint its full-n numbers.

The chain's method held: every quality claim carries its measurement, every rejection carries its table.



---
_Comment/review ids in this file: 5060444954_

