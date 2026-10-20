# #234 — D3: the current Gate C evaluation distribution is answered entirely by exact-context lookup, so it cannot measure generalization

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @maurathat
- **Created:** 2026-07-27T03:32:18Z
- **Updated:** 2026-07-30T06:07:03Z
- **Closed:** 2026-07-30T06:07:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/234
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body


### Summary

D3 (evaluation distribution) is listed in the graph compiler implementation plan as still open — "natural-corpus pick still open" — and Gates C, G, and K all inherit that declaration. The recorded Gate C run shows why this is not a housekeeping item: on that corpus, exact-context lookup answered **every** held-out probe, so the routing, residual, and chain-telescoping machinery was never exercised.

Until an evaluation distribution is declared in which EXCT cannot answer, Gate C restates the baseline rather than measuring what the graph compiler adds.

### Evidence

From the double-counting risk-register entry (confirmed 2026-07-22), after the Rule 1 chain-telescoping and Rule 2 EXCT-precedence redesign:

| Configuration | top-1 agreement | bits/token |
| :---- | :---- | :---- |
| Σ-over-cloud (pre-fix) | 0.3% | 76.96 |
| Rule 1 only (no EXCT) | 1.2% | 71.52 |
| Rule 1 \+ Rule 2 | 31.7% | 9.86 |
| TLA3 baseline | 31.7% | 11.88 |

Two things in that table matter more than the headline:

1. **Win/loss against baseline is ±0, with identical argmax everywhere.** The graph path ties the flat baseline exactly on prediction; only the distribution shape differs.  
2. **Status was `ExactContext` on 30,036 of 30,036 held-out probes.** Rule 2 answered everything. Rule 1 alone — the layered residual path — scores 1.2%.

So the 31.7% is attributable to exact-context memorization, not to the multiresolution structure. That is a property of the corpus, not a result about the compiler.

### Why this is a corpus problem rather than a compiler problem

A held-out set constructed such that every context already appears in the exact-context store cannot test generalization by construction. The evaluation is not wrong; it is measuring the wrong thing, and it will keep reporting a tie no matter what the graph compiler does.

This also means the Gate C trend alarm now running in CI is monitoring a quantity that is currently insensitive to the changes it is meant to guard.

### What is being asked for

A declared D3 evaluation distribution with:

1. **A stated EXCT-miss rate.** The fraction of held-out contexts absent from the exact-context store, reported as a property of the distribution and fixed at declaration time. A distribution with a miss rate near zero cannot serve as a Gate C corpus.  
2. **A construction method**, not just a corpus name — how held-out contexts are selected so the miss rate is achieved and reproducible. Story-id splitting alone does not achieve it, as the current result shows.  
3. **Reported status breakdown per evaluation run.** The count of held-out probes resolved at each `ResolutionStatus` level, alongside the headline metrics. This makes the failure mode visible in every future report rather than discoverable only by reading the risk register.  
4. **Slices**, per Gate K, so that in-store and out-of-store contexts can be read separately. A single blended number hides exactly the distinction this issue is about.

### Suggested acceptance criterion

Gate C reports are considered informative when the declared distribution has a non-trivial EXCT-miss rate and the per-status breakdown is included in `score_report.json`. The specific miss-rate threshold is a maintainer decision and should be set as part of the D3 declaration rather than proposed here.

### Relationship to other work

- **Blocks meaningful Gate C**, and therefore the value of the Gate C trend alarm in CI.  
- **Gates G and K inherit D3** per the implementation plan, so this decision propagates.  
- Independent of the evaluation-harness questions filed separately (mul-free store fragmentation, teacher floor `inf`). Those concern whether the instrument works; this concerns what it is pointed at. Both need resolving, in either order.

### Offer

Happy to do the work of constructing and documenting a candidate distribution plus the miss-rate measurement, if the maintainers would rather review a proposal than specify one. The parts that need to be a maintainer decision are the corpus choice itself and the miss-rate threshold, since Gates C, G, and K all inherit them.

### References

- `docs/r4_graph_compiler_implementation_plan.md` — §2 (D1–D8, D3 open), §8 (Gates A–K), §10 (risk register, double-counting entry confirmed 2026-07-22)  
- `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md` — confidence-bounded empirical-claim protocol, which any declared distribution should conform to  
- Gate K: distribution, n, confidence intervals, slices, protocol



## Comments (9)

### comment 5086999301 — 2026-07-27T03:34:56Z — maurathat

@afflom flagging this one for you specifically: the two pieces that need a maintainer decision are the corpus choice and the EXCT-miss-rate threshold, since Gates C, G, and K all inherit them. Prior D3 corpus work for reference: #72, #75, #197 (all closed; the declaration itself is still open per the plan §2). Happy to construct and document a candidate distribution plus the miss-rate measurement if you'd rather review a proposal than specify one.

### comment 5087014374 — 2026-07-27T03:37:55Z — afflom

> @afflom flagging this one for you specifically: the two pieces that need a maintainer decision are the corpus choice and the EXCT-miss-rate threshold, since Gates C, G, and K all inherit them. Prior D3 corpus work for reference: #72, #75, #197 (all closed; the declaration itself is still open per the plan §2). Happy to construct and document a candidate distribution plus the miss-rate measurement if you'd rather review a proposal than specify one.

We can understand the correct answer with reconstructability analysis (ra) and understanding Pareto optimality. We don't decide this, the implementation must be parametric. 

### comment 5087065489 — 2026-07-27T03:49:13Z — maurathat

Refining the "maintainer decision" framing in the issue body: the miss-rate threshold shouldn't be decided at all — it should be parametric, with the analysis making the choice legible rather than someone decreeing it.

1. **Per-status breakdown, independent of everything below.** Counts of held-out probes resolved at each `ResolutionStatus`, emitted in `score_report.json` alongside the headline metrics. This isn't a parameter choice, it's instrumentation — and without it the 30,036/30,036 EXCT result is only discoverable by reading the risk register. Worth landing on its own.

2. **Report the three-structure margin.** What Gate C should be measuring is how much of held-out behavior each sub-structure reconstructs: EXCT alone, the graph path alone, and their composition. Composition-minus-EXCT is what the compiler actually adds, and the blended number hides it. This is already measurable at one operating point from the double-counting entry — Rule 1 alone 1.2%, Rule 1+2 31.7% — so the ask is to make that decomposition standard rather than incidental.

3. **Parametric evaluation.** The harness takes the split-construction parameter driving miss rate as an input and emits the metric surface at each operating point, rather than one blessed threshold. `score_report.json` gains a sweep, not a constant.

4. **Pareto, not point selection.** Corpus and miss rate trade representativeness of the deployment distribution against sensitivity to the graph path. That's a frontier. Declaring D3 then means documenting it and pinning CI's trend alarm to one or two operating points — a stability choice, not a correctness one.

This shrinks what needs a maintainer to exactly one thing: which operating points CI pins. The rest falls out of the parametric run.

Framing note: this is reconstructability analysis in the Klir/Zwick sense — reconstructing a joint from overlapping projections, scored by information loss against degrees of freedom. Full structure-lattice search grows super-exponentially and would need directed search from independence or saturation, so I'm not proposing that here; items 2–3 are the tractable subset.


### comment 5123513787 — 2026-07-29T21:24:06Z — Casey-allard

## The serving surface reproduces this issue's finding — measured via the new honest C row (PR #288)

First held-out measurement of the deployed configuration (R4Engine + score.r4g1 + D4 policy, bundle SmolLM2-135M-Instruct-7e27bd9f9532, stride subsample n=1021):

- **served 1021/1021 — every position resolved `ExactContext`**; 0 graph, 0 novel, 0 widened, 0 abstentions
- on served: top1 5.6% | teacher agreement 12.6%

Two consequences for this issue:

1. The blended-number failure mode is now *visible by default*: the C row reports per-status breakdowns (item 3 of what this issue asks for) — `uor_r4_api::serving_eval::StatusBreakdown`, printed with every row. A declared D3 distribution with a stated EXCT-miss rate (items 1-2) remains the open work; this row gives it a measurement surface that will show the miss rate the moment such a corpus exists.
2. New data point the original table couldn't see: on the serving path the exact-context-answers-everything distribution scores only 12.6% agreement (vs 31.7% in the score-time Rule 1+2 eval) — the serving surface derives signatures from token windows rather than observation vectors, so even the memorization tier is measured differently there. Worth folding into the declaration's construction requirements: the held-out set should be stated against the *serving* signature derivation, not only the observation one.

### comment 5124270515 — 2026-07-29T23:03:46Z — Casey-allard

Declaration machinery merged in batch-2 (PR #289, merge c187ef0): `score_report.json` schema 9 carries a `DistributionDeclaration` — held-out positions, ExactContext-resolved count, the **EXCT-miss rate fixed at declaration time**, and a Gate C validity verdict against a declared 5% minimum, printed loudly at score time. Items **1 and 3** of this issue are delivered (3 on both surfaces: score-time via gate_c status counts + this declaration; serving-time via the C row's per-status breakdowns, PR #288). **Item 2 remains the open scope**: a construction method that actually achieves a non-trivial miss rate. Note for that work: the #268 finding (record streams interleave stories in ~9-token runs) means any construction should be declared against *story-contiguous* replay semantics, and per the #288 measurement the held-out set should also be stated against the serving signature derivation (token windows), not only the observation one.

### comment 5125827283 — 2026-07-30T02:53:47Z — Casey-allard

## Item 2 resolved by a structural finding, not a corpus construction (2026-07-29, branch `issue-234-exct-construction`, commit 0e5f1c3)

**The status-based EXCT-miss rate is structurally zero on every possible corpus.** Reading the Rule 2 probe (`score_runtime.rs`): it walks `(0..=STAGES).rev()` and stops at the first POPULATED prefix — and the level-0 (root, empty-key) prefix is populated with total ≥ EXCT_SUPPORT_MIN on any non-empty train split. So `ScoreStatus::ExactContext` cannot miss, no construction method can achieve the declared 5% threshold on the status basis, and the schema-9 declaration measures a quantity that is identically ~0 by code, not by corpus. Sub-full-depth resolutions are prefix backoff wearing the EXCT badge.

**Instrumented (schema 10, semantics-neutral):** per-position probe-resolution-level histogram + a STRICT miss rate (probe resolved at the full graded code with support); Gate C validity verdict now judges the strict basis. Serving/scoring semantics untouched.

**Measured on the pinned fixture harness** (same 30,036-position split as the recorded Gate C row):

| quantity | value |
|---|---|
| status-based miss rate | 0.0% (30,036/30,036 ExactContext — reproduces the recorded complaint exactly) |
| **strict full-code miss rate** | **50.3%** (15,117/30,036) |
| probe resolution levels L0…L4 | 0 / 2,099 / 7,117 / 5,901 / 14,919 |
| Rule 1+2 vs TLA baseline | 30.02/10.24 vs 30.02/12.36 bits — identical argmax (0/0 win-loss), the recorded tie |
| Rule 1 (graph alone) | 1.22/16.17 |

**Consequence for item 2:** the existing story-split D3 distribution ALREADY exceeds the 5% threshold under strict semantics — no new corpus or construction method is required. Half of Gate C sits on probes whose "exact context" evidence is depth-1–3 backoff, on which the graph machinery is preempted and never exercised.

**Open maintainer decision (⚑ Casey):** Rule 2 firing depth. Today any populated prefix with support ≥ 5 preempts the graph. If Rule 2 is restricted to full-code (or a declared depth floor), the 50.3% flows to Rule 1 — which currently scores 1.2%/16.2, so an honest Gate C will initially read FAR worse than the tie it reports today. That is the measurement working, not a regression: it makes the graph path's actual contribution visible for the first time. This decision changes serving semantics and every downstream Gate C row, so it is yours; the declaration-layer instrumentation above does not wait on it.

### comment 5125851002 — 2026-07-30T02:57:42Z — Casey-allard

**Maintainer decision (Casey, 2026-07-29): Rule 2 fires at FULL CODE ONLY** — EXCT precedence means exact context in the strict sense (probe resolved at level == STAGES with support ≥ EXCT_SUPPORT_MIN). Prefix backoff no longer preempts the graph; sub-full-depth probes fall through to Rule 1 (graph / Novel → D4 abstention policy at serving). Known consequence, accepted on the record: honest Gate C will initially read far below the current 30/30 tie on the ~50% strict-miss slice (Rule 1 measures 1.2%/16.2 today) — that is the graph path's true contribution becoming visible, and it becomes the number to improve. Implementation follows on branch issue-234-rule2-depth; witness replay and both EXCT probe surfaces change together.

### comment 5125907953 — 2026-07-30T03:06:18Z — Casey-allard

Decision implemented: PR #294 (full-code-only Rule 2, both probe surfaces + docs; witness replay 64/64 consistent). Honest Gate C row on the pinned fixture harness, with the accepted regression on the record: blended 21.05%/12.53 bits (was the 30/30 tie), status split ExactContext 14,919 / Graph 15,080 / Novel 37 — the ExactContext count equals #293's L4 histogram bin exactly, so the instrumentation and the semantics change cross-verify. Per-status: exact 41.1%/8.55, graph 1.23%/16.47. With #293 (probe-level histogram + strict declaration basis, schema 10) and #294 both merged, all four asks of this issue are delivered and the D3 story split is a valid Gate C distribution on the strict basis (50.3% ≥ 5%); the graph path's 1.2% is now the number to improve. Both PRs are in the merge queue; this issue can close when they land.

### comment 5127234818 — 2026-07-30T06:07:02Z — Casey-allard

Closing with the DoD evidence — all four asks delivered and merged:

1. **Stated EXCT-miss rate**: schema-9 declaration (batch-2, c187ef0) + the schema-10 STRICT basis (PR #293, d3293ba) after the structural finding that the status-based rate is identically ~0 on any corpus (the probe backs off to populated prefixes, root included).
2. **Construction method**: none needed — resolved by measurement. Under strict full-code semantics the existing D3 story split already exceeds the 5% threshold (50.3% strict miss on the sign-era fixture; 13.9% under metric-consistent dot probes, PR #298 evidence), and under metric-consistent probing the distribution exercises the graph path on real probes. The construction question dissolved once the instrument was fixed.
3. **Per-status breakdown in every report**: status counts + per-status metrics (batch-2) + probe-level histogram (PR #293), printed loudly at score time and carried in score_report.json.
4. **Slices**: per-status, per-probe-level, and the strict/status dual basis; Gate C validity verdict judges the strict rate.

Semantics decision implemented: Rule 2 fires at FULL CODE ONLY (maintainer decision, PR #294, 68daf4d) — the honest Gate C row replaced the 30/30 tie, the trend pin re-priced twice with era notes as the metric-consistency work landed (#296, #298), and the graph path (~1.2% on its slice) is now the measured number to improve (#277/#246). Merge commits: d3293ba (#293), 68daf4d (#294), plus the evidence rows in #296/#298.


---
_Comment/review ids in this file: 5086999301, 5087014374, 5087065489, 5123513787, 5124270515, 5125827283, 5125851002, 5125907953, 5127234818_

