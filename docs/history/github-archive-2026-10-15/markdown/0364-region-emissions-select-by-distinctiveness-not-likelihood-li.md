# #364 — Region emissions select by distinctiveness, not likelihood — lists cover 1.57% of next-token mass

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-03T02:42:12Z
- **Updated:** 2026-08-03T20:17:43Z
- **Closed:** 2026-08-03T19:49:48Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/364
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Graph-path root cause: emissions are selected by distinctiveness, not likelihood

Region emission lists cover **1.57%** of the probability mass they are supposed to
model. This is the root cause of the graph path's ~1–2% top-1 and its ~10 bits/token
coherence gap, traced through #358 / #360 / #363.

## The defect

`crates/uor-r4-graph-certify/src/score.rs`, emission construction:

```rust
let ln = lp_n - lp_p;            // log P_region(token) − log P_parent(token)
…
residuals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
residuals.truncate(config.emission_entries);   // DEFAULT_EMISSION_ENTRIES = 64
```

Selection ranks by **log-ratio against the parent**, keeping the tokens most
over-represented in the region. A rare token 5× over-represented outranks the
region's actual most-probable next token, whose ratio is near zero precisely
because it is common everywhere.

## Measurement

Fixture artifact, comparing the shipped top-E-by-ratio list against a
top-E-by-probability one over the same region evidence:

```
regions=38  mean_overlap_with_top_count=0.0078  mean_probability_mass_kept=0.0157  (E=64)
```

- **Overlap 0.78%** — the two selections are nearly disjoint, ~0.5 tokens in
  common out of 64.
- **Mass kept 1.57%** — the emitted list covers 1.6% of what actually happens
  next in that region.

**98.4% of each region's real next-token mass carries residual 0** and is scored
at the bare global prior, while the 64 kept tokens receive large positive boosts.

## Why the scoring identity still looks right

`root_score + log-ratio = log P_region` is the correct decomposition. The bug is
not the formula, it is the **truncation**: dropped tokens silently take ratio 0
rather than their true correction, and ranking by ratio drops exactly the tokens
that carry the mass. The identity holds for 64 rare tokens and is violated for
everything else.

## What this explains

| observation (source) | explained by |
| --- | --- |
| teacher present in region emissions 3.8% (#363) | list covers 1.6% of mass |
| 89% of winners are on-chain (#360) | boosted rare tokens win |
| median teacher rank 63 (#358) | ~64 boosted tokens sit above it per chain node |
| α = 0 improves top-1 4.5× and bits by 3.09 (#358) | removes the spurious boosts |
| graph 17.10 bits/token vs exact_context 6.83 | distribution dominated by rare tokens |
| 82% of graph retrieval is the bare prior (#363) | regions contribute almost no usable evidence |

Components previously suspected and cleared: ranking/calibration, residual sign
and scale, chain selection. Predicted cloud F measured as genuinely useful
(11.1%). The defect is upstream of all of them.

## Fix shape

Compiler-side only — no runtime, artifact ABI, or kernel change:

1. rank emissions by **posterior probability** (or by contribution to covered
   mass) and store the log-ratio as the value, preserving the scoring identity
   while covering the mass that occurs; or
2. keep a mass target rather than a fixed count — retain entries until some
   fraction of the region's next-token mass is covered; or
3. raise `emission_entries` — the weakest option, since with 0.78% overlap the
   ranking is wrong regardless of depth, and artifact size grows for little gain.

Option 1 or 2 should be measured on the **graph slice** with bits/token as the
criterion, using the per-status instrumentation now on `main`.

## Relation to #362

#362 adds trigram/bigram context rows against the same gap. Both are worth
having, but they are different fixes: #362 supplies context-conditioned evidence
alongside the graph, while this makes the graph's own evidence usable. With
regions currently covering 1.6% of mass, #362's ablation should include the
marginal-contribution condition — otherwise a trigram win cannot be distinguished
from routing around a graph that is not contributing.

Measurement branch: `graph-emissions-followup` (`a501d01`).


## Comments (12)

### comment 5161939613 — 2026-08-03T03:17:58Z — auser

## Telescoping is intact. The region estimates themselves are the problem.

Follow-up to #364/#366. The hypothesis was that per-level truncation breaks the
telescoping sum — a token kept at a descendant need not be kept at its ancestors,
and a missing level contributes 0 instead of its real correction.

**Largely refuted.**

| graph slice | ratio (shipped) | probability |
| --- | ---: | ---: |
| mean chain levels | 2.73 | 2.73 |
| mean levels emitting teacher | 0.01 | 1.60 |
| **telescoping COMPLETE** | 0.09% | **54.18%** |
| telescoping PARTIAL (broken sum) | 1.26% | 9.61% |

Partial sums are real but 9.61% — far too few to explain graph top-1 of 2.90%
against 8.76% with the residual disabled. Under corrected selection the sum is
**intact for 54% of positions**.

### The harder conclusion

Even where the chain is complete and the correction is mathematically right,
applying it makes both metrics worse — top-1 2.90% vs 8.76%, bits +1.32. So on
graph-status positions **`log P_region` is a worse next-token predictor than the
global unigram prior.**

That is not a defect in selection, telescoping, or scoring arithmetic. Those are
now all measured and either fixed or exonerated. It says the region-conditional
distributions do not carry predictive value over the prior.

### Most likely cause: sparsity, not a bug

Roughly 2,600 positions per region against a 32,000-token vocabulary. Per-region
next-token estimates are extremely sparse, and add-one smoothing does not rescue
them — a region's empirical distribution is dominated by sampling noise, so
conditioning on it is worse than not conditioning at all.

The α sweep independently supports this: the optimum sits near **α ≈ 0.25**,
which is the shape of a **shrinkage coefficient**, not of a sign error (would
peak at α < 0) or a scale error in the usual sense.

### Direction

This points at estimation, not plumbing:

1. **Shrinkage toward the parent weighted by region evidence count** — the
   standard fix for sparse conditional estimates (Witten-Bell / Jelinek-Mercer
   style). The stored value is already a parent-relative delta, so this is a
   per-region weight, not a format change.
2. **Larger regions or fewer of them** — trades resolution for estimate quality;
   testable against the same Gate C slices.
3. **Better smoothing than add-one** — `Smoothing` is already a configurable
   enum (#67), so alternatives are cheap to ablate.

Option 1 is the most direct and is measurable with the instrumentation now on
`main`.

### Note on #362

This strengthens the case for trigram/bigram context rows. They condition on
*lexical* context, where counts are far denser than per-region geometric
conditioning at this cover size, so they may supply the evidence the regions
cannot at current sparsity. It also sharpens the marginal-contribution condition:
if regions carry no predictive value over the prior, a trigram win is a
replacement, not an enhancement, and that should be stated explicitly either way.

Measurement branch: `graph-residual-calibration` (`bd928fe`).


### comment 5162260604 — 2026-08-03T04:21:34Z — auser

Follow-up complete through merged PRs #366 and #367:

- #366 merged at `42f6926fd47eab390e7f1c328d60dab7ef25e60e`; probability-ranked selection raises emission mass coverage from 1.57% to 69.17% and improves graph bits/token, while remaining opt-in.
- #367 merged at `6b22fa6bb146102fec3846cefc0ba7125defc4ff`; telescoping integrity is largely intact (54.18% complete under probability selection), so the remaining top-1 loss is not explained by broken sums.

Leaving this issue open for the next measurement-backed residual calibration/shrinkage step, as documented in the merged PRs.

### comment 5162295506 — 2026-08-03T04:26:48Z — auser

## Correction: the sparsity explanation is wrong. Regions are densely estimated.

I wrote above that the residual's remaining harm was "most likely sparsity, not a
bug — roughly 2,600 positions per region against a 32,000-token vocabulary." That
figure was wrong. I took it from the Gate C held-out position count rather than
the training corpus, and did not check it.

Measured directly:

```
regions=38  mean_n=3,105,750  mean_types=2,198  mean_lambda_wb=0.9992
```

Regions average **3.1 million next-token observations over ~2,200 distinct
types** — about 1,400 observations per type. These are densely estimated
distributions.

### The test that revealed it

Witten-Bell shrinkage (`--emission-shrinkage witten-bell`, scaling each region's
residual by `n/(n+T)` at compile time) changed nothing:

| variant | graph top-1 | graph bits |
| --- | ---: | ---: |
| probability + none | 2.90% | 13.32 |
| probability + witten-bell | 2.90% | 13.31 |
| **root-only baseline** | **8.76%** | **12.00** |

Because λ = 0.9992, it barely shrank anything — I effectively ran the same
configuration twice. The experiment failed to test the hypothesis, and in failing
revealed the hypothesis was unnecessary.

### What this leaves

The finding is sharper and harder: **with 3.1M observations per region,
`log P_region` is still a worse next-token predictor than the global unigram
prior.** That cannot be estimation noise.

It indicates the regions are internally well estimated but the contexts routed
into them do not share those regions' continuations — a **routing / semantic-fit**
problem, not an estimation one. The cover is grouping contexts that are similar
under masked-Hamming signature distance but that differ in what follows them.

The α sweep already bounds the whole reweighting family: no positive α beats
α = 0, so no global rescaling recovers value the conditioning does not contain.
Direction 1 in this issue (shrinkage) is therefore answered and closed;
directions 2 and 3 (coarser regions, different smoothing) address estimate
quality, which is not the binding constraint.

### Revised direction

The open question is whether the geometric cover groups contexts by
next-token-relevant structure at all. That is testable: compare the next-token
distribution of contexts *within* a region against the region aggregate — if
within-region contexts disagree with their own region's distribution, the
grouping is not predictive regardless of how well each region is estimated.

That is a different and more fundamental question than anything measured so far,
and it bears directly on #362: if geometric grouping is not next-token
predictive, lexical trigram context is not an enhancement to it but a
replacement.

Measurement branch: `graph-emission-shrinkage` (`8ca83a2`).


### comment 5162349095 — 2026-08-03T04:36:15Z — auser

## The cover IS predictive — 1.41× over arbitrary. The problem is signal-to-noise, not validity.

Control: does a context's **own** region emit its actual continuation more often
than an **unrelated** region does? The unrelated node is chosen by a fixed
integer hash of the position index — deterministic, no RNG, independent of the
context's geometry — so the test has a built-in null.

| selection | own region | unrelated region | **lift** |
| --- | ---: | ---: | ---: |
| ratio (shipped) | 0.74% | 0.48% | 1.55× |
| **probability (#366)** | **60.36%** | **42.69%** | **1.41×** |

**The geometric premise holds.** Routing carries real next-token information.
This is the first hypothesis in this thread to survive its own test, after seven
that did not.

### But specialization is thin

The 42.69% baseline is the important number: an unrelated region's top-64
contains the teacher nearly half the time. Emission lists are largely the same
globally-common tokens across regions, so a region distinguishes itself from an
arbitrary one by only 1.41×.

### This closes the explanation

The regions are:

- **not mis-estimated** — 3.1M observations each over ~2,200 types;
- **not meaningless** — 1.41× lift over arbitrary;
- **mostly generic with a thin specialized margin.**

And the scorer applies that thin margin as though it were a full correction. That
single fact accounts for every result in the thread: why no positive α helps
(signal-to-noise too low for full-strength application), why `log P_region` loses
to the unigram prior despite abundant data, why shrinkage by evidence count was
beside the point (the estimates are fine; the *contrast* is weak), and why
correcting emission selection (#366) improved the candidate set without making
the residual helpful.

### What this implies for direction

The lever is **contrast**, not estimation:

1. **Increase specialization** — the cover's regions overlap heavily in what they
   emit. Finer or better-separated regions, or an admissibility criterion on
   region distinctiveness, would raise the 1.41×.
2. **Apply conditioning proportional to measured contrast** — weight a region's
   residual by how far its distribution actually departs from the global prior,
   rather than uniformly. This differs from the shrinkage already tried, which
   keyed on evidence *count*; count is not the binding constraint, contrast is.
3. **Accept the geometric path as a router, not a scorer** — it demonstrably
   routes (1.41× lift, and predicted cloud F contributes 11%) while its emission
   conditioning does not pay for itself at current contrast.

Direction 2 is the cheapest test and is a direct consequence of this measurement.

### Bearing on #362

Lexical trigram context should be compared against a 1.41× geometric baseline,
not against zero. The geometric path contributes something real; the open
question is whether it contributes enough to justify its cost alongside a
context-conditioned n-gram table.

Measurement branch: `graph-cover-predictive` (`387a20c`).


### comment 5162439405 — 2026-08-03T04:53:52Z — auser

The shrinkage follow-up is now merged via PR #368 at `0aa005a9ddfbe36c1f1bdcc36ddac003d811f124`.

The compiler exposes opt-in `--emission-shrinkage none|witten-bell`; the default remains unchanged. On the fixture, probability selection measured 36.6% top-1 and 7.7933 bits/token without shrinkage versus 36.6% and 7.7927 with Witten–Bell (mean λ=0.9992), so the control is safe but not promoted as default. The issue remains open for genuinely sparse-corpus calibration evidence.

### comment 5162466215 — 2026-08-03T04:58:37Z — auser

## Contrast weighting works — and establishes the ceiling. Parity, not improvement.

Direction 2: weight each region's residual by `1 − overlap(region top-E, global
prior top-E)`. A region whose most-likely tokens *are* the prior's conditions on
nothing; a distinctive one contributes fully. Distinct from the evidence-count
shrinkage already tried, which measured λ = 0.9992 and was never the binding
constraint.

| variant | graph top-1 | graph bits |
| --- | ---: | ---: |
| probability + none | 2.90% | 13.32 |
| probability + witten-bell | 2.90% | 13.31 |
| **probability + contrast** | **5.42%** | **12.01** |
| **root-only baseline** | **8.76%** | **12.00** |

Contrast weighting nearly doubles top-1 and **closes the bits gap entirely** —
12.01 against a 12.00 baseline. It also beats the equivalent global constant
(α = 0.5 gives 5.26%), so per-region contrast carries real variance; my stated
concern that it would degenerate into a fixed α was wrong.

### The ceiling

It does not clear the bar. Conditioning is now **exactly break-even on coherence
and still negative on top-1**. At this cover's specialization level — own region
emits the teacher 1.41× more often than an unrelated one, against a 42.7%
unrelated baseline — the best achievable from geometric emission conditioning is
**parity with not conditioning at all**.

Contrast is the right quantity: weighting by it recovered essentially all the
recoverable value. There is no more to recover at 1.41× specialization.

### What that isolates

The remaining lever is **region construction**, not scoring. Every scoring-side
question is now measured and closed:

| component | verdict |
| --- | --- |
| emission selection | defect, fixed (#366): mass coverage 1.57% → 69.17% |
| ranking / calibration | exonerated |
| residual sign, residual scale | exonerated (α sweep) |
| chain selection | exonerated (0.59% discarded) |
| telescoping integrity | intact (54% complete) |
| estimate sparsity | refuted (3.1M obs/region) |
| conditioning weight | **optimized — reaches parity, not gain** |

For the graph path to beat the prior, regions must become more distinctive than
1.41×. That is a cover-induction question: separation criteria, granularity, or
an admissibility rule on region distinctiveness — all upstream of anything in the
scorer.

### Bearing on #362 and the FMM line

- **#362**: lexical trigram context should be compared against a geometric path
  whose realistic contribution is *parity with a unigram prior*, not against
  zero and not against the shipped 1.96%. On this evidence trigram rows would be
  a replacement for emission conditioning, with the geometric path retained for
  routing (where it does work — predicted cloud F contributes 11%).
- **#290 / #359 / #361**: these optimize the emission-residual machinery. That
  machinery is now measured at break-even against a unigram prior on the graph
  slice. Worth knowing before further optimization effort — the speedups are
  real, but they accelerate a component not currently paying for itself in
  accuracy.

Measurement branch: `graph-contrast-weighting` (`7fc472b`).


### comment 5169027280 — 2026-08-03T16:27:33Z — Casey-allard


# The ceiling is scorer-conditional: granularity converts under the fixed scorer, and lexical rows beat the baseline on top-1

Full campaign on the checked-in TLA7 fixture (tracker #374; instrumentation #371/#372/#373/#375-#378, all merged today). Three results bear directly on this issue's conclusion.

**1. The 1.41×/parity ceiling reproduces under the shipped scorer — and dissolves under the #366+#370 opt-ins.** The revived cover sweep (#373), run for the first time under both configs on the identical 9-point grid: with ratio/none, granularity buys nothing (36.5% → 36.2% top-1 at 38 → 102 regions; the knee rule confirms the default cover adequate — this issue's verdict, reproduced). With probability/contrast, the same knob converts monotonically: 36.98%/7.599 at 38 regions → **37.70%/7.409 at 102 regions**, the knee rule flips to "default too coarse," and every contrast row beats the store baseline's bits (7.41–7.60 vs 8.4461) with top-1 closing to −1.4pp. Per-region stats show the mechanism: mass-kept 1.57% → 69–77%, contrast mean 0.50 → 0.59 (max 0.91). So the ceiling measured here was a joint artifact of the shipped emission config and the coarse cover — cover-induction has headroom once the scorer keeps probability mass. This strengthens the case for an induction-time distinctiveness term (the split objective optimizes within-region entropy while the payoff is between-region contrast) and for revisiting Design P (#276) as a distinctiveness lever.

**2. Trigram rows (#362) are top-1-stronger than everything measured — and distributionally degenerate.** With the new Gate C window mode (#375/#378; the historical surface never exercised rows at all — empty window, proven by exact_context_ngram=0), rows-on scores **41.00% top-1, beating the 39.09% store baseline** — the first configuration on record to do so; rows resolve 99.1% of positions. But bits collapse to 10.79 (single-candidate distributions): rows as shipped are an argmax mechanism, not a distribution model. The replacement architecture this issue sketched (lexical rows for emission, geometric path for routing) looks right on top-1 and needs row *distributions* (top-k exists in the NGR1 format; the scorer consumes only the argmax) before it's adoptable on coherence.

**3. The repetition penalty is a measured ~10pp top-1 cost on this distribution.** Window-on rows-off drops to 26.69% vs 36.47% window-off — the −2M recent-token penalty, which serving always pays on real windows, destroys accuracy on teacher-forced story text. Filing separately as its own measurement question.

Raw tables, reports, and per-status decompositions on #374. Proposed follow-ups: (a) induction-time distinctiveness term (ObjectiveWeights slot, schema bump), (b) row-distribution emission (NGR1 top-k consumed as a distribution), (c) repetition-penalty measurement issue, (d) #276 re-evaluation. Sequencing per the #374 queued-next comment.


### comment 5169569394 — 2026-08-03T17:22:19Z — auser

## Correction: the induction already targets continuation structure. My architectural claim was wrong.

I wrote — in this issue and in the body of merged PR #370, calling it "the
architectural statement this thread converges on" — that the cover is *induced by
context-bundle geometry* while predictive value requires grouping by
continuation, and that fixing it "would need an induction objective optimizing
continuation discriminativeness."

That is wrong. `induction.rs` already does exactly that. From its own module docs
and split acceptance:

```rust
// a region splits only when the within-region next-token entropy reduction
//   H(parent) − Σ_c (|c|/|parent|)·H(c)
// exceeds entropy_gain_bits
let entropy_allows_split = gain > config.entropy_gain_bits;   // DEFAULT 0.25
```

plus an information-bottleneck objective recording `H(A|R)`, `H(S_future|R)` and
an `I(Z;X) − β·I(Z;Y_future)` term per eligible split.

So continuation structure is already in the induction loop. I should have read
the induction before characterizing it, and I asserted the opposite in a merged
PR.

### The corrected statement

The distinction that survives is narrower and more precise:

- **Candidate partitions come from geometry** — spherical k-means on context
  bundles proposes the splits;
- **continuation structure only vetoes them** — next-token entropy gain is an
  acceptance filter, not the thing being maximized.

So the cover can only capture continuation structure that *happens to align with
signature-space geometry*. Entropy gain can reject a bad split; it cannot propose
a good one that geometry did not offer. That is consistent with everything
measured — 1.41× own-region lift, contrast bounded in [0.39, 0.75], and
conditioning topping out at parity — without requiring the false claim that
continuation is absent from the objective.

### It also hands us a concrete lever

`entropy_gain_bits` defaults to **0.25**; the pinned config used for the #290
cover used **0.1**. That threshold directly controls how discriminative an
accepted split must be. Raising it forces splits to earn more next-token entropy
reduction, at the cost of fewer regions — a real trade-off, and cheaply testable
against the contrast distribution and the graph-slice metrics now instrumented.

That is a much smaller and better-posed experiment than "change the induction
objective," which is what I was about to propose on the strength of a claim I had
not checked.

### Standing

Everything measured on the scoring side still holds — the emission-selection
defect and its fix (#366), the exonerations, the 1.41× lift, the parity ceiling.
Only my explanation of *why* the cover is weakly aligned was wrong, and this is
the fifth interpretation of mine to need correction in this investigation.


### comment 5169732245 — 2026-08-03T17:36:50Z — auser

## The entropy threshold is not a usable lever: at 0.75 bits the cover collapses to zero splits.

Following the correction that induction *does* gate splits on next-token entropy
(`entropy_gain_bits`, default 0.25), the obvious question was whether raising
that threshold produces more discriminative regions. Swept on the fixture corpus
(depths 3, k0 8, budget 256):

| entropy-gain | regions | splits accepted | depths reached |
| ---: | ---: | ---: | ---: |
| 0.10 | 54 | 23 | 3 |
| **0.25** (default) | 38 | 15 | 3 |
| **0.75** | **8** | **0** | **1** |
| 1.50 | 8 | 0 | 1 |

**At 0.75 bits, no split anywhere in the tree is accepted.** The cover collapses
to the initial k0 = 8 clustering with no refinement. Since 15 splits clear 0.25,
every achievable gain lies in a narrow band of roughly **(0.25, 0.75) bits**.

### What this establishes

Raising the threshold does not buy distinctiveness — it buys *nothing*, because
the geometry never proposes a split that discriminative. The lever is exhausted
at its current setting.

This is the corrected framing confirmed from the other direction:

- **geometry proposes** the candidate splits (spherical k-means on context
  bundles);
- **entropy only vetoes** them;
- and the proposals themselves cap out below 0.75 bits of next-token entropy
  reduction.

So the binding constraint is the **proposal mechanism**, not the acceptance
filter. That is why regions top out at 1.41× own-region lift and contrast ≤ 0.75,
and why conditioning cannot beat parity: the material the cover is built from is
only marginally continuation-discriminative before any filtering happens.

### Consequence for direction

Improving region distinctiveness requires changing **what splits are proposed** —
i.e. clustering on something that correlates with continuation, rather than
filtering geometric proposals after the fact. That is a substantially larger
change than a threshold, and this measurement is the evidence that the cheap
version of the fix does not exist.

### Note on coverage

The shipped `cover-sweep` (#373) spans `entropy_gain_bits ∈ {0.25, 0.10}` — both
below the collapse point — so this band was unmeasured. Worth adding a point
above 0.5 to that sweep's axis if it is going to be used to reason about the
threshold, otherwise it can only ever show the "more regions" direction.


### comment 5170034888 — 2026-08-03T18:07:33Z — auser

## Real-bundle result: n-gram rows do not touch generalization, and the "126-bit pathology" is likely a bits-accounting artifact, not a model failure.

Replicated the context-order A/B on the real `SmolLM2-135M-Instruct-7e27bd9f9532`
bundle (362-region pinned cover, vocab 49,152). Two findings, one of which
corrects something I said earlier today.

### 1. N-gram rows improve only the memorizable population

| order | slice | n | top-1 | bits | recall | median teacher rank |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 0 | graph | 10,784 | 5.99% | 126.96 | 73.1% | 110 |
| 0 | novel | 4,221 | 5.24% | 110.33 | 72.1% | 83 |
| 1 | graph | 653 | 5.05% | 115.05 | 70.4% | 244 |
| 1 | novel | 165 | 1.82% | 91.03 | 58.8% | 178 |
| 2 | graph | 653 | 5.05% | 115.05 | 70.4% | 244 |
| 2 | novel | 165 | 1.82% | 91.03 | 58.8% | 178 |

**Orders 1 and 2 are bit-identical on every non-lookup slice.** Trigram rows
change nothing outside the n-gram-resolved population; the blended difference
(22.31% → 21.58%) comes entirely from within it, trading a little top-1 for a
little bits.

At no context order does the system exceed ~6% top-1 or fall below ~91
bits/token on positions requiring generalization. N-gram rows resolve 92% of
positions and improve those; the remaining 8% are untouched, by construction —
a position with no matching n-gram gets no n-gram help.

Worth stating plainly because it bears on the project's thesis: the blended gain
is **compensation for a failed generalization mechanism, not construction of
one**. That is a legitimate engineering choice for a served system, but it is not
progress on generalization, and #234's concern — that this evaluation is largely
answerable by exact-context lookup — applies with full force to any metric moved
this way.

### 2. Correction: the 126-bit figure is probably not a model pathology

I described the graph slice's 126.96 bits/token as "confidently wrong" output
resembling the high-confidence gibberish the single-chain restriction was built
to prevent. The score magnitudes do not support that:

| | root spread | residual spread |
| --- | ---: | ---: |
| real bundle (order 0) | 8.75 nats | **3.10 nats** |
| real bundle (order 2) | 8.65 nats | **2.18 nats** |
| fixture | 6.87 nats | 6.57 nats |

Residuals are *smaller* on the real bundle than on the fixture. Scores spanning
~9 nats cannot yield 127 bits/token.

**Probable mechanism, offered as a hypothesis to verify rather than a
conclusion.** `outcome_bits` computes `log2(sum / w_teacher)` and charges
uncovered vocabulary at `w_floor = weight(root_floor)`, with `max_s` taken over
candidate scores. Candidate scores include `transition_offset`, which is added to
every candidate — rank-neutral, which is why it never appeared in any argmax
measurement — but **the floor is not shifted with them**. A positive offset
therefore pushes candidates up relative to the floor, collapses the
uncovered-mass weight, and charges an enormous penalty on any position where the
teacher is absent from the candidate set (≈30% here, given 70% recall).

That predicts what is observed: 22.53 bits root-only (no offset, no residual)
versus 126.96 with them, on identical positions, with all score spreads under 9
nats.

If this holds, then bits/token on the graph slice has been **overstating the
model's error** wherever the offset is large, and comparisons that mix
offset-bearing and offset-free scores — including several I have quoted in this
issue — are not like-for-like. It does not affect top-1 or any argmax result,
since the offset is rank-neutral.

### The genuinely encouraging number

Candidate recall on non-lookup positions is **58–73%** with median teacher rank
83–244. The system retrieves the right token on novel contexts most of the time
and then ranks it near-last. That is a broken ranking over present information,
not absent capability — a much better position than "the geometry carries no
signal," which is what the fixture suggested.

### Suggested next step

Verify the offset hypothesis directly: compare bits on positions where the
teacher **is** in the candidate set against those where it is not, and check
whether `transition_offset` magnitude predicts the gap. If confirmed, either
shift the floor with the candidates or exclude the offset from the bits
computation — and re-derive any bits-based conclusion in this issue that spans
the two regimes.


### comment 5171006613 — 2026-08-03T19:49:44Z — auser

## Campaign resolved

The remaining measurement question from this issue is now answered on `main`:

- Probability-ranked emissions landed in #366, and the telescoping control landed in #367.
- Witten–Bell shrinkage and contrast weighting were measured in #368/#370; the subsequent #374 campaign showed the scorer/cover interaction and the trigram A/B result.
- The Gate C window and lexical-row attribution landed in #378; top-k NGRAM distributions landed in #383.
- The transition-offset bits hypothesis from the latest comment was verified and corrected in [PR #389](https://github.com/UOR-Foundation/uor-r4/pull/389), merge commit `2c1bfa2`: the uncovered-vocabulary floor is priced on the same candidate scale. This removes the real-bundle present/absent inversion while leaving the zero-offset fixture metrics unchanged.

The resulting conclusion is explicit: the original ratio-selection defect is addressed through the measured opt-in/compiler paths, while the shipped defaults remain byte-compatible. Further generalization/reporting work is tracked separately in #390; this parent campaign has no remaining implementation item.

### comment 5171259859 — 2026-08-03T20:17:43Z — auser

## The parity ceiling does not hold on the real bundle: contrast-weighted conditioning clears the baseline.

The ceiling ("geometric conditioning tops out at parity with the prior") was
established entirely on the Gate C **fixture** — 38 regions, contrast bounded in
[0.39, 0.75]. The real bundle has **362 regions and contrast reaching 0.9531**,
and with #389 fixing the bits accounting, the comparison is finally valid there.

Real bundle, 362-region pinned cover, `--context-order 0` (so the graph slice
holds 10,784 positions rather than the 653 that survive once n-gram rows absorb
92%), corrected bits:

| shrinkage | graph top-1 | root-only top-1 | graph bits | root-only bits |
| --- | ---: | ---: | ---: | ---: |
| none | 3.75% | 5.19% | 23.36 | 22.53 |
| witten-bell | 3.76% | 5.19% | 23.34 | 22.53 |
| **contrast** | **5.99%** | 5.19% | **22.51** | 22.53 |

**Contrast weighting clears the root-only baseline** — the first time in this
investigation the geometric path has beaten "don't condition at all."

### Size of the effect, stated precisely

- **Top-1: a real gain.** 5.19% → 5.99%, +0.80 pp, **+15% relative**. Against
  unweighted conditioning it is +60% relative (3.75% → 5.99%), so the weighting
  is doing the work, not the residual by itself.
- **Bits: a wash.** 22.51 vs 22.53. A −0.02 margin is noise, not a coherence
  improvement.

So: **the geometric path adds measurable ranking value on a real cover when
weighted by contrast, and no measurable distributional value.** Unweighted and
count-shrunk conditioning still hurt on both, exactly as measured everywhere else.

### What this revises

The #364 ceiling verdict is **fixture-specific and should not be generalized**.
At 38 regions with contrast compressed into [0.39, 0.75], per-region weighting
degenerates toward a uniform α and parity is the best available. At 362 regions
with contrast spanning [0.156, 0.953] there is real variance to exploit, and
exploiting it beats the prior.

That materially weakens the "n-gram rows replace the geometric path" reading. The
replacement finding itself still stands — n-gram rows resolve 92% of positions —
but the component being displaced is not worthless: it is a positive contributor
on the positions it handles, once weighted correctly.

### Bearing on #359 / #361

Those optimize emission-residual machinery. On this evidence that machinery does
contribute, provided contrast weighting is in force. The decision about whether
it stays in the serving path should be taken against these numbers rather than
against the fixture's parity verdict.

### Caveat

`--context-order 0` was chosen to give the graph slice enough positions to
measure. With n-gram rows enabled the slice is 653 positions and the same
comparison would be underpowered. Whether the geometric path still clears its
baseline *alongside* n-gram rows is a separate question and is not answered here.

Measurement branch: `graph-real-ceiling`.



---
_Comment/review ids in this file: 5161939613, 5162260604, 5162295506, 5162349095, 5162439405, 5162466215, 5169027280, 5169569394, 5169732245, 5170034888, 5171006613, 5171259859_

