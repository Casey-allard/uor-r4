# #362 — feat: add trigram-to-unigram context fallback rows

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T01:31:06Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-03T03:32:15Z
- **Labels:** enhancement
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/362
- **Merged:** 2026-08-03T03:32:15Z
- **Merge commit:** e77b1d4585b58bc2e696816ba4fc446b18d3890b
- **Base / Head:** main / UOR-Foundation:issue-362-ngram
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Objective

Integrate explicit hierarchical context evidence into the R4G1 prediction path:

- unigram/root rows: no context, token frequency prior;
- bigram rows: `previous_token -> next_token`;
- trigram rows: `(previous_previous_token, previous_token) -> next_token`;
- prediction precedence: use a supported trigram row first, then its bigram row, then the unigram/root prior.

The fallback is deterministic and allocation-free in the deployed runtime.

## Implementation

- derive unigram, bigram, and trigram counts from corpus positions without crossing story boundaries;
- define sparse-context support/backoff semantics and deterministic tie-breaking;
- add the packed optional R4G1 NGRAM section with canonical sorted rows;
- add a borrowed GraphView accessor and integer-only runtime lookup;
- wire the compiler/exporter into the normal scored artifact path;
- retain the 8-token rolling window and masked-Hamming router as separate semantic-region machinery.

## Validation

- full local workspace test suite and clippy pass;
- no-std and alloc format ladders pass;
- malformed header, row-range, overlap, reserved-byte, ordering, and trailing-byte rejection;
- deterministic artifact-byte rebuild;
- deployed R4G1 runtime parity for trigram precedence and bigram fallback;
- deterministic tie-breaking, zero steady-state allocations, and integer-only source scan;
- GitHub CI run 30780462653 on `acff2d0`: audit, D2 differential (Ubuntu/macOS), wasm, fuzz smoke, fmt/clippy/nextest, BDD, docs, no-std, deterministic rebuild, canonical κ, and Gate C all successful.

## Non-goals

This does not redefine graph-region similarity or replace the masked-Hamming router. If trigram context is later used as a routing feature, that should be a separately measured change.

The held-out teacher-forced ablation comparing unigram-only, unigram+bigram, and trigram backoff remains the empirical decision gate for enabling this optional section by default.

Related: #359, #290

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### comment 5161442876 — 2026-08-03T01:40:29Z — auser

Implemented the first runtime/artifact slice in `83890ef`:

- optional canonical `NGR1` section for bigram/trigram rows;
- stage-2 validation plus borrowed `GraphView::ngram_table()`;
- compiler/exporter wiring from story-boundary-safe corpus counts;
- integer-only graph-runtime lookup with trigram → bigram → existing EMIT unigram fallback;
- deterministic ordering and malformed-section tests.

Focused validation is green: graph-format tests, graph-runtime tests, graph-certify tests, format no_std checks, fmt, and clippy.

Still required before this PR is merge-ready: make the certifier/API `GraphScorer` consume the same NGRAM rows and add the held-out unigram/bigram/trigram ablation plus allocation/source-scan evidence.

### comment 5161463820 — 2026-08-03T01:45:06Z — auser

API/certifier wiring is now in `4b2210f`:

- `GraphScorer` loads NGRAM rows from the validated artifact;
- allocation-free `score_step` and reference `score_candidates` use the same trigram → bigram → root fallback;
- witnesses retain the bounded recent-token context so independent replay reproduces lexical decisions;
- API status mapping continues through the existing `ExactContext` policy path.

Additional validation: graph-format, graph-certify, graph-runtime, and graph-api tests pass; focused clippy/fmt checks are green.

### comment 5161503003 — 2026-08-03T01:52:03Z — auser

Added measurement visibility in `7d7d9c4`:

- score reports now record NGRAM row count, retained candidate count, and section bytes;
- CLI prints the NGRAM footprint alongside EMIT/EXCT;
- added a deterministic unit test proving trigram keys never cross story boundaries.

The full teacher-held-out ablation remains pending because this checkout has no compiled corpus/model fixture under `.uor-models/`; the next run against the pinned fixture will provide the storage and accuracy decision data.

### comment 5161543015 — 2026-08-03T01:58:48Z — auser

## Review — the premise is well supported by fresh measurement, and that same measurement should change the ablation

Measurements landed today in #358 / #360 bear directly on this PR.

### The diagnosis supports the premise

On the Gate C fixture corpus, split by resolution status:

- the graph slice sits at **1.96% top-1, 17.10 bits/token** (exact_context: 42.47%, 6.83);
- candidate recall on that slice is **63%**, but **active regions emit the teacher
  only ~4.3% of the time** (1.35% on the selected chain, 2.95% off it);
- so the teacher is reaching the candidate list through `root_top` (the global
  prior's top-B) or predicted cloud F — **not through region emissions**.

The graph path is being carried by a *context-free* prior. That is precisely the
gap this PR names: "the graph compiler currently has a root prior and region
emissions … it does not retain explicit bigram/trigram rows." Context-conditioned
next-token evidence is the missing ingredient, and the diagnosis is independent
of this PR.

### Ablation should be per-status, not blended

The DoD calls for an ablation on "top-1, top-8 recall, bits/token." Measured
blended, a graph-path gain will be diluted roughly 6× — exact_context is 85,429
of 100,306 positions and is largely answerable by exact-context lookup, which
trigram rows cannot improve and should not be credited for.

`rule12_per_status` (#358) and the residual-structure fields (#360) are on `main`
now, so the per-slice numbers come out of the same run at no extra cost. I would
report the graph slice as the headline and keep blended as a guard against
regression elsewhere.

### Report bits, not just top-1

The coherence property against the teacher / matmul path is distributional. The
graph slice's problem is as much its 17.10 bits/token as its top-1, and a change
can lift top-1 while leaving the distribution as wrong — I measured exactly that
distinction earlier today and it mattered.

### One condition worth adding: the graph's marginal contribution

Trigram → bigram → root is a precedence chain that can bypass region emissions
entirely. If it works, metrics improve — but today's data says region emissions
carry ~4% of the answer, so it is entirely possible to see a solid win **while
the geometric routing contributes nothing measurable**.

The non-goals say this does not replace the router, and I take that at face
value. But as specified the ablation cannot distinguish "trigram backoff plus a
working graph" from "trigram backoff routing around a graph that is not pulling
its weight." Suggest adding a condition isolating the graph's marginal value —
e.g. trigram-only versus trigram + region emissions — so the answer is on the
record either way. If the marginal contribution is ~0, that is a significant
finding about the geometric path and better learned here than later.

### Mechanical

- currently **BEHIND**; will need a rebase.
- touches `score.rs` and `score_runtime.rs`, both of which #358 changed on `main`
  and #360 changes again (new instrumentation structs in `score.rs`;
  `candidate_components` and `node_emits` in `score_runtime.rs`). Expect conflicts
  in those two files specifically.

### Not a blocker

Nothing above argues against the change. The structure — optional canonical NGR1
section, borrowed accessor, integer-only lookup, story-boundary-safe counts,
deterministic bytes — matches how context evidence should enter the artifact, and
the staged commits are easy to follow. The asks are about what the ablation
measures, not about the implementation.


### comment 5169814834 — 2026-08-03T17:45:08Z — auser

## A/B run: trigram rows are a large win, and they **replace** the geometric path rather than enhance it.

Ran the ablation with `--context-order 0|1|2` on the Gate C fixture corpus, with
`--gate-c-context-window true` (#378), `--emission-selection probability` and
`--emission-shrinkage contrast` (#364/#370).

### First attempt was a null — worth recording

Without `--gate-c-context-window`, all three orders produced **byte-identical**
results and `exact_context_ngram = 0`. The n-gram rows never fired, because plain
Gate C scoring does not supply the preceding tokens the lookup needs. Read only
as a metrics table, that run says "trigrams show no benefit" — which would have
been a wrong claim about this PR. Anyone running this ablation needs the context
window on.

### Result (window on)

| context-order | blended top-1 | blended bits/token | ngram-resolved |
| --- | ---: | ---: | ---: |
| 0 (no NGRAM rows) | 26.96% | 17.98 | 0 |
| 1 (bigram) | 31.56% | 13.26 | 99,427 |
| **2 (bigram+trigram)** | **41.00%** | **10.78** | 99,427 |

**+14.0 pp top-1 and −7.2 bits/token** over no context rows; **+9.4 pp and
−2.5 bits** over bigram alone, so trigram earns its keep over bigram and not
merely over nothing.

### The marginal-contribution condition

This is the condition I asked for in the earlier review, and it answers cleanly.

| | order 0 | order 1 | order 2 |
| --- | ---: | ---: | ---: |
| graph slice population | 14,860 | **292** | **292** |
| graph conditioning margin (bits) | −0.09 | −0.15 | −0.15 |

N-gram rows resolve **99.1%** of positions. The geometric path is left with 292
positions — **0.3%** of the corpus — and its conditioning margin is negative even
there.

So on this evidence trigram context is a **replacement** for geometric emission
conditioning, not an enhancement to it. That is consistent with #364's finding
that the geometric path tops out at parity with a unigram prior: a component at
parity is straightforwardly displaced by one that is 7 bits better.

### Two caveats on reading these numbers

1. **Window-on and window-off are different evaluation regimes.** Order 0 with the
   window on gives graph 2.50% / 23.80 bits, against 5.42% / 12.01 measured with
   it off. Do not line these up against previously quoted window-off figures; the
   comparisons above are all within the window-on runs.
2. **Slice populations move.** The graph slice goes 14,860 → 292 between orders,
   so cross-order *graph-slice* deltas are not apples-to-apples. The blended row
   is the valid comparison because it covers all 100,306 positions, and the
   per-order margin is valid because both of its terms come from the same run.

### What this implies

The geometric cover remains useful as a **router** — it routes (1.41× own-region
lift, #369) and predicted cloud F contributes real candidates. What this measures
is that its **emission conditioning** does not pay for itself once lexical context
is available, and n-gram rows cover almost the whole corpus.

Worth deciding explicitly rather than by default: whether emission residuals stay
in the serving path at all, and what that implies for #359/#361, which optimize
that machinery.


### comment 5169981515 — 2026-08-03T18:01:55Z — auser

## Correction: the trigram gain does not replicate on the real bundle.

I posted an A/B above showing trigram beating bigram by **+9.4 pp top-1 / −2.5
bits** on the Gate C **fixture** corpus. Replicated on the real
`SmolLM2-135M-Instruct-7e27bd9f9532` bundle — 362-region pinned cover
(`cover.r4g1`), vocab 49,152, same flags — that does not hold.

| context-order | fixture blended | **real bundle blended** |
| --- | ---: | ---: |
| 0 (none) | 26.96% / 17.98 | 9.29% / 57.85 |
| 1 (bigram) | 31.56% / 13.26 | **22.31% / 16.72** |
| 2 (bigram+trigram) | 41.00% / 10.78 | **21.58% / 16.60** |

On the real bundle **trigram is slightly worse than bigram on top-1** (−0.73 pp)
with only a marginal bits gain (16.72 → 16.60). The fixture's +9.4 pp does not
replicate, and this PR is specifically the trigram change, so my earlier
"trigram earns its keep over bigram" is withdrawn.

### What does replicate

**Context rows themselves are a large win.** Order 0 → 1 gives +13.0 pp top-1 and
**−41.1 bits/token** on the real bundle. And n-gram rows absorb 92% of positions
(37,130 of 40,342), collapsing the graph slice from 10,784 to 653 — so the
*replacement* finding holds at 362 regions as it did at 38.

The case for context rows is therefore stronger than before; the case for the
**trigram order specifically** is not supported by the real bundle and rests on
the fixture alone.

### Suggested follow-up for this PR

Worth measuring on the real bundle before the trigram order is locked in:

1. whether `--context-entries` (default 64) is the binding constraint at
   order 2 — trigram contexts are sparser, so a table sized for bigrams may
   truncate trigram rows disproportionately;
2. whether trigram rows help on the subset where a trigram context is actually
   *present*, which the blended figure dilutes;
3. bits alongside top-1 as the criterion, since the two disagree in sign here.

Bigram alone is the safe, replicated win. Trigram may still be right, but the
evidence for it currently comes from a 38-region fixture and reverses at 362.

### Also observed, flagged not asserted

The geometric emission conditioning is far worse on the real bundle than on the
fixture: graph slice 126.96 bits/token against 22.53 root-only, a −104 bit
margin, where the fixture showed roughly parity. 126 bits against a 49,152 vocab
(uniform ≈ 15.6) implies P(teacher) ≈ 2⁻¹²⁷ — confidently wrong rather than
uninformative. That resembles the "pathological high-confidence gibberish" the
single-chain restriction was introduced to prevent, and should be verified
independently before it is relied on.

Correspondingly, the real cover has genuinely distinctive regions — contrast max
**0.9531** (fixture: 0.75), min 0.156 (fixture: 0.391) — so the "no region
approaches high specialization" observation I recorded on #364 was
fixture-specific and does not describe the shipped cover.



---
_Comment/review ids in this file: 5161442876, 5161463820, 5161503003, 5161543015, 5169814834, 5169981515_

