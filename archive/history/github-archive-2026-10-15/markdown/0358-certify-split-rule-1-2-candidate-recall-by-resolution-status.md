# #358 — certify: split Rule 1+2 candidate recall by resolution status

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T00:27:14Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-03T01:15:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/358
- **Merged:** 2026-08-03T01:15:21Z
- **Merge commit:** 5a34dfd51f6f02c7c06a5fd095431f02a4a757b9
- **Base / Head:** main / UOR-Foundation:graph-path-per-status-recall
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Makes the graph path's failure mode measurable. Instrumentation only — no
scoring, routing, or serving behavior changes.

## Why

#234's closure names the graph path as "the number to improve" (~0.6–1.2% top-1
depending on era). The schema-10 report also carries `candidate_recall` — whether
the candidate list *contains* the teacher's argmax token — at ~72%.

Read together those suggest a **ranking** failure rather than a **retrieval**
failure: the scorer finds the right token and puts it in the wrong place. That
would be large and actionable.

But they cannot be read together. `candidate_recall` divides by all scored
positions, and `exact_context` is ~86% of them carrying 36.65% top-1 — so the 72%
is plausibly explained by the exact path alone. The report has per-status
agreement but no per-status recall, so nothing currently distinguishes:

- **high graph-slice recall** → scorer retrieves the teacher token and ranks it
  badly. A weighting problem, tunable — and notably one the #290 far-field
  negative does not touch.
- **low graph-slice recall** → candidate generation never surfaces the answer,
  and no scorer tuning helps. A routing/beam problem (#277/#246 territory).

Those have disjoint fixes. The project is currently choosing between them blind.

## Change

Splits Rule 1+2 candidate recall by resolution status, mirroring the existing
`status_hits` / `status_bits` accumulation exactly. New additive field
`rule12_candidate_recall_per_status`; schema 10 → 11, following #293's convention
for added instrumentation.

The existing blended `candidate_recall` is unchanged, so no consumer breaks.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
and the certify suite (53 tests across 11 files) are green.

**Not yet verified end-to-end.** Producing the actual split requires regenerating
a score report, and per #232 the certify path spends ~2.5 of 3.2 hours on that
eval. The instrumentation is cheap; the number is not. Flagging rather than
implying the split has been observed — the field is added, the value has not been
read.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### comment 5161121365 — 2026-08-03T00:31:05Z — auser

## Measured. The graph path retrieves well and ranks badly — the split resolves it.

Ran the Gate C measurement with this instrumentation on the fixture corpus
(`scripts/gate_c_trend.sh`'s path — the CI trend job, not the ~2.5h full certify
of #232; I over-estimated the cost initially).

| slice | positions | candidate recall top-1 | candidate recall top-3 | top-1 agreement | conversion |
| --- | ---: | ---: | ---: | ---: | ---: |
| exact_context | 85,429 | 79.60% | 90.53% | 42.47% | **53%** |
| **graph** | 14,860 | **62.97%** | **87.55%** | **1.96%** | **3.1%** |
| novel | 17 | 82.35% | 94.12% | 23.53% | 29% |

"Conversion" = agreement / recall: of the positions where the teacher's argmax is
*in the candidate list*, how often it is actually selected.

### The answer

**Ranking, not retrieval.** The graph path finds the teacher's argmax 63% of the
time and ranks it first 2% of the time — a 61-point gap. Its top-3 recall is
87.6%, so nearly nine times in ten the right answer is already in the candidate
list.

Exact-context converts at 53%; the graph path converts at 3.1%. That is a **17×
difference in ranking effectiveness on candidates it already retrieved**.

### Consequences

1. **Routing is not the bottleneck for graph-path accuracy.** Candidate
   generation is working. #277/#246-style routing and VP-tree work will not move
   this number much — worth knowing before more effort goes there.
2. **The headroom is in the scorer and it is large.** At exact-context's
   conversion rate the graph slice would sit near 33% top-1 instead of 1.96%.
3. **Disjoint from #290.** The far-field operator was killed as a long-range
   mechanism (§5.2 gate, negative). This is a ranking/weighting problem in a path
   that already retrieves well — untouched by that result.

### Validity check

The per-status values reweight to the existing blended figure:
(85,429×0.7960 + 14,860×0.6297 + 17×0.8235) / 100,306 = 0.7713 against the
reported `rule12_top1` of 0.77139. The new accumulation agrees with the one
already in the report, which is the cheap check that it is wired correctly.

Note this fixture corpus scores 100,306 positions; the schema-10 report in
`trend_output/` scored 30,036, so absolute agreement figures differ by era
(graph 1.96% here vs 0.574% there vs ~1.23% in #234's closure). Same order, and
the retrieval-vs-ranking conclusion does not depend on which era is used.


### comment 5161145152 — 2026-08-03T00:36:21Z — auser

## Follow-up: it is not a near miss. The graph scorer buries tokens it retrieved.

The recall split left two live diagnoses — a near-miss the scorer almost gets, or
a signal carrying little ranking information. Added the teacher argmax's rank
within the candidate list to separate them.

| slice | median teacher rank | rank 1 | top-3 | rank ≥ 65 |
| --- | ---: | ---: | ---: | ---: |
| exact_context | **1.0** | 53.4% | 74.5% | **0** |
| **graph** | **63.0** | 3.1% | 4.7% | **47.2%** |

Graph-slice distribution over the 9,357 positions where the teacher token *was*
retrieved:

```
rank    1:  291     2:   63     3:   84   4-8:  271
     9-16:  866  17-32: 1781  33-64: 1582  65-128: 2344  129+: 2075
```

### Reading

**Calibration is ruled out.** A tie-breaking or small systematic-bias fix moves
rank 2–3 mass up to rank 1. There is almost none: 147 positions at ranks 2–3
against 4,419 beyond rank 65. Nearly half of successfully retrieved teacher
tokens sit past rank 65.

Exact-context never pushes the teacher past rank 64 at all — its buckets 65-128
and 129+ are both empty — while the graph path puts 47.2% there. The two paths
are not differently-tuned versions of the same thing; one ranks and the other
effectively does not.

### Wiring check

A teacher token at rank 1 *is* a correct selection, so `buckets[0]` must equal
the hit count. It does, exactly, for all three statuses: 36283/36283, 291/291,
4/4. That is the check that the tie-break condition (`score == teacher_score &&
token < teacher_argmax`) is oriented correctly — getting it backwards would shift
every rank by the number of tied tokens without looking obviously wrong.

### Where this points

Scores are assembled as

```rust
score(token) = root_score(token) + residual + transition_offset
             - 2_000_000  if token in recent_tokens
```

The observed signature — good retrieval, teacher scattered deep, no mass near the
top — is what you would expect if `root_score`, a global prior, dominates the
residual in magnitude. Ranking would then be mostly by global token frequency
regardless of which region the graph traversal selected, which is consistent with
the graph path finding the right neighbourhood and then ignoring it.

That is a hypothesis, not a result. Testing it needs the score split into its
root and residual components on graph-status rows — the natural next measurement,
and now clearly worth taking since calibration is excluded.


### comment 5161183631 — 2026-08-03T00:44:55Z — auser

## The graph residual is not inert — it is actively harmful. Removing it would be a 4.5× improvement on that slice.

Rank data ruled out calibration. My stated hypothesis was that the root prior
dominates and the residual is inert. **That hypothesis is wrong**, and the actual
finding is more consequential.

Decomposition: scores are `root_score + residual + transition_offset`, minus a
repetition penalty. `transition_offset` is common to all candidates and provably
cannot reorder them, so the question is whether the per-token residual moves the
argmax away from what the prior alone would choose. Comparing
`argmax(root + penalty)` against the real argmax under the identical
ascending-token / strict-`>` tie-break:

| slice | root prior alone | full scorer | bits/token | residual effect |
| --- | ---: | ---: | ---: | --- |
| exact_context | 24.63% | **42.47%** | 6.83 | helps, +1.7× |
| **graph** | **8.76%** | **1.96%** | **17.10** | **hurts, −4.5×** |
| novel | 23.53% | 23.53% | 9.29 | none (no residual exists) |

**On graph-status positions the root prior alone selects the teacher token 8.76%
of the time. Adding the graph residual drops it to 1.96%.**

The residual is not weak or ignored. It changes the decision 89% of the time
(`root_only_agrees` = 11.29%) and carries 8.28 nats of median spread against the
root's 14.09. It is large, it is decisive, and on this slice it points the wrong
way.

### Why this is a measurement and not an artifact

- **novel** reports `residual_spread = 0` and `root_only_agrees = 100%`. Novel
  means an empty selected chain, hence no residual, hence the two argmaxes must
  be identical. They are.
- **exact_context** shows the *same code path* helping — 24.63% → 42.47%. The
  machinery works where EXCT precedence applies; it is the graph-path residual
  specifically that inverts.
- Status is classified by chain emptiness, not by the residual, so removing the
  residual would not reshuffle which positions count as graph.

### Relation to coherence against the teacher / matmul path

Top-1 is a proxy; the coherence measure is bits/token, and the graph slice sits
at 17.10 against exact-context's 6.83. A residual that actively misranks does not
merely miss the argmax — it distorts the whole distribution, which is what a
~10-bit gap looks like. Any candidate fix should be judged on bits and finally
against the teacher-parity suite (top-1 / top-8 recall / Δbits vs the live
SmolLM2 teacher), not on top-1 alone.

### What this does not establish

That deleting the residual is the fix. This measures the argmax under
`root + penalty` on the existing candidate sets; it shows the residual is net
destructive on this slice, not that 8.76% is the ceiling or that a sign/scale
correction recovers more. The likely causes worth separating next are sign or
scale error in how chain-telescoped residuals are accumulated, versus the graph
evidence being genuinely wrong on positions where EXCT does not resolve. Those
are distinguishable, and the instrumentation to do it is now in place.

Also worth noting: three hypotheses have been advanced on this thread today and
this is the second to be refuted by its own measurement. Treating the mechanism
above as a lead rather than a conclusion.


### comment 5161200956 — 2026-08-03T00:48:45Z — auser

## Sweep result: neither sign nor scale. The graph residual carries no ranking signal.

Two explanations survived the previous measurement — inverted accumulation
direction (optimum at α < 0) or wrong scale (optimum at 0 < α < 1). Sweeping
`root + α·residual` in exact i64 rational arithmetic refutes both.

| α | exact_context | **graph** |
| ---: | ---: | ---: |
| −1.00 | 11.49% | **8.7618%** |
| −0.50 | 13.75% | **8.7618%** |
| −0.25 | 17.19% | **8.7618%** |
| **0.00** | 24.63% | **8.7618%** ← root prior alone |
| +0.25 | 33.17% | 8.6474% |
| +0.50 | 38.94% | 6.8035% |
| **+1.00** | **42.47%** ← best | **1.9583%** ← shipped |

**Not a sign error.** Negating the residual yields *exactly* the same 8.7618% as
deleting it — identical to four decimals, the same hit count. An inverted signal
would make α = −1 beat α = 0. It is indistinguishable from absence.

**Not a scale error.** No positive α beats α = 0; even α = 0.25 is marginally
worse, and the damage grows monotonically with weight.

That leaves the third pre-registered branch: **the graph residual carries no
usable ranking signal**. It is not pointing the wrong way — it is carrying
nothing, and weighting it injects noise proportional to the weight.

### The control is what makes this diagnostic

On exact_context the *same term* is monotonically informative across the entire
sweep — 11.49% at α = −1 rising to 42.47% at α = +1, best at full weight. The
residual machinery is sound and the shipped weighting is correct there.

So this is not "residuals are broken." It is specifically the residual computed
for positions where EXCT does not resolve. That points at what the residual
tables encode on the graph path — a compiler-side question, not a scorer one.

### Consistency

Six checks pass exactly: α = 1 reproduces the shipped per-status agreement
(42.4715% / 1.9583%) and α = 0 reproduces the independently-measured root-only
agreement (24.6275% / 8.7618%), on both populated slices, via a different code
path from the one that produced those numbers earlier.

`novel` is flat at 23.53% across all α, as required — an empty chain means no
residual, so α cannot matter.

### What this does NOT license

Setting α = 0 and calling it a 4.5× win. The sweep measures **argmax only**.
Coherence against the teacher / matmul path is a distributional property, and the
graph slice's problem is as much its 17.10 bits/token against exact-context's
6.83 as it is top-1. A change can correct the argmax and leave the distribution
just as wrong; bits/token has to be measured before α = 0 is called an
improvement rather than merely a higher top-1.

The substantive finding is diagnostic, not a patch: **the graph path retrieves
well (63% recall), and the evidence it then applies is uninformative.** Whatever
the residual tables encode for non-EXCT positions is not ranking information.


### comment 5161231109 — 2026-08-03T00:55:30Z — auser

## Coherence check: suppressing the residual improves the distribution too — but closes only ~3 of the ~10-bit gap.

The sweep left an open question I flagged rather than assumed: top-1 rising
1.96% → 8.76% does not establish that bits/token improves, and coherence against
the teacher / matmul path is distributional.

Measured through the same softmax and the same uncovered-vocab floor as the
shipped path, only the scores differing:

| slice | top-1 α=1 | top-1 α=0 | bits α=1 | bits α=0 | Δ bits |
| --- | ---: | ---: | ---: | ---: | ---: |
| exact_context | 42.47% | 24.63% | 6.83 | 7.85 | **+1.01** (worse) |
| **graph** | 1.96% | **8.76%** | 17.10 | **14.01** | **−3.09** (better) |
| novel | 23.53% | 23.53% | 9.29 | 9.29 | 0.00 |

Both metrics move the right way on the graph slice. The exact_context control
moves the opposite way, confirming the residual is genuinely informative where
EXCT resolves. `novel` is exactly unchanged, as an empty chain requires.

### The important qualification

**This is not a fix.** At 14.01 bits the graph path remains 7.2 bits from
exact_context's 6.83. Suppressing the residual closes roughly 3 of the ~10-bit
gap; the remaining ~7 bits have a different cause that none of today's
measurements touch.

So the residual is one confirmed contributor to graph-path incoherence, not the
explanation for it. A stopgap that gates the term off on graph rows would be a
genuine improvement on both axes and would still leave the path far from
coherent.

### Where the thread stands

Localized over five measurements, each checked against an invariant it had to
satisfy:

1. retrieval works, ranking does not — 63% recall, 2% selection
2. not calibration — median teacher rank 63, no near-top mass
3. the residual is active and destructive — 8.76% → 1.96%
4. neither sign nor scale — negating equals deleting exactly; no positive weight
   beats zero
5. suppressing it helps coherence as well as top-1 — but only by ~3 of ~10 bits

The open question is upstream and unchanged by this result: **what do the
residual tables encode for positions where EXCT does not resolve?** The same
machinery is monotonically informative where it does resolve, so this is a
compiler-side question about emission on the graph path, not scorer tuning.

The second open question is now separate and quantified: what accounts for the
remaining ~7 bits.


### comment 5161311829 — 2026-08-03T01:11:50Z — auser

## The defect is chain selection, not residual arithmetic.

Residual accrues only on `selected_chain` nodes; tokens emitted by active or
predicted nodes keep residual exactly `ZERO`. I predicted that would let
off-chain tokens float above chain tokens carrying a summed penalty. **The
measurement inverts that**, and the real structure is sharper.

| slice | zero-residual candidates | teacher **on** chain | winner **off** chain |
| --- | ---: | ---: | ---: |
| exact_context | 0.00% | **79.60%** | 0.00% |
| **graph** | **89.86%** | **1.35%** | 11.29% |
| novel | 100.00% | 0.00% | 100.00% |

89% of graph-slice winners are **on** chain, while the teacher token is on chain
only **1.35%** of the time. Putting that together with the earlier recall figure:

- candidate generation finds the teacher **63%** of the time, via active/predicted
  nodes, as an off-chain candidate with residual exactly zero;
- the selected chain's emissions contain the teacher **1.35%** of the time;
- the scorer boosts chain emissions, reliably promoting tokens that are not the
  answer and burying one that is.

That is why α = 0 helps on both axes: removing the residual removes the boost and
lets the retrieved teacher compete on the root prior alone.

**So the residual is not broken — it faithfully amplifies a chain that is picking
the wrong regions.** Tuning, negating, or rescaling the term cannot fix that,
which is exactly what the α sweep found.

`exact_context` is a different regime and acts as the control: 0% zero-residual
candidates and the teacher on chain 79.6% of the time. Same machinery, and there
it is monotonically helpful. The difference is not the scorer — it is which chain
gets selected.

### Where this leaves the thread

The question is no longer "why does the scorer rank badly" but **"why does chain
selection pick regions whose emissions exclude a token the cover already
retrieved?"** Candidate generation and chain selection are consulting the same
cover and disagreeing 98.65% of the time on the graph slice.

### Corrections to earlier claims in this thread

1. I described α = −1 as "indistinguishable from absence." What was measured is
   the same *agreement rate* (8.7618%), not the same decisions — both can
   misrank while agreeing with the teacher equally often. The sweep conclusion
   stands (no α beats 0), but that phrasing overstated it.
2. I proposed a mechanism in `compute_residual_deltas` where a missing parent
   entry defaults to `ScoreQ::ZERO` and breaks telescoping. That function has no
   callers outside its own tests — dead code, not on the score path. Withdrawn.

That is four hypotheses on this thread refuted by their own measurements. The
finding that survived is the one I did not predict, in the direction opposite to
the one I guessed.

### Note on this PR

The commit carrying this measurement is not pushed here: I enqueued #358 and then
continued committing to its branch, which the merge queue correctly rejects. Per
AGENTS.md that work belongs on a fresh branch off `main` after this lands, and it
will go there. My error, and precisely the #323 lesson the repo already documents.



---
_Comment/review ids in this file: 5161121365, 5161145152, 5161183631, 5161200956, 5161231109, 5161311829_

