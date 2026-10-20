# #394 — Anchor-infill criterion: grade constrained syntax completion between routed content anchors

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T23:31:45Z
- **Updated:** 2026-08-04T15:51:57Z
- **Closed:** 2026-08-04T15:51:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/394
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Follow-up to the R4-native architecture framing issue. The serving stack's product role is syntax completion between routed content anchors — not open next-token mimicry. Certify's current rows structurally reward exact-context memorization (measured: #391 generalization below unigram null; #392 phase negative; #379/#386 distinctiveness dead). This issue adds the criterion that grades the actual job.

## Spec sketch (v0, fixture corpus)

- Teacher-forced reference stream from the pinned fixture corpus (same era/schema as current certify rows).
- **Anchor positions**: every 4th token of the reference stream is pinned (mirrors the original hybrid's injection cadence). Free positions: the intervening 3.
- Task: predict each free position given (a) all anchors, (b) previously produced free tokens — i.e., constrained infill, look-back AND look-forward available by construction.
- Metrics per free position: top-1 exact match vs reference; bits/token where a distribution exists.
- **Null ladder (per #391 discipline)**: (1) unigram null, (2) anchor-copy null (nearest anchor), (3) rows-only arm, (4) window-off arm. A mechanism only counts as carrying syntax if it clears (1) and (2) on held-out.
- Harness: ignored test + results doc, following the #303 `hopf_sector_occupancy` pattern — no certify schema change until the criterion is validated and the gate conversation happens with Alex/Ari.

## Tonight

Day-0 baselines on the fixture: current engine (window-on/rows-on and rows-off) vs the null ladder. Numbers posted here regardless of outcome.

## Open questions

- Anchor cadence sensitivity (every 3rd/4th/5th) — sweep later, fixed at 4 for Day-0.
- Whether free-position prediction should consume future anchors through E_f/E_b edges explicitly (the geometric look-ahead mechanism) — that is the follow-on design work this criterion exists to grade.


## Comments (5)

### comment 5172937871 — 2026-08-03T23:48:01Z — Casey-allard

## Day-0 results (fixture corpus, stride 4) — harness in PR #396

Fixture: 500,000 records, 2,507 stories, 80/20 story cut; artifact κ `blake3:ef6a2035…`. Held-out free targets: 75,385. All tables built from the construction partition only.

| arm | top1 (free) | off1 | off2 | off3 |
|---|---|---|---|---|
| shipped-store (causal) | **34.7%** | 34.5 | 34.8 | 35.0 |
| null: unigram | 6.3% | 6.2 | 6.6 | 6.1 |
| null: bigram | 28.7% | 28.9 | 28.3 | 29.1 |
| null: prev-anchor-copy | 0.5% | 0.0 | 0.4 | 1.0 |
| null: fwd-anchor-table | 21.9% | 14.2 | 19.0 | **32.7%** |

Two headline facts:

1. **The forward anchor is a large, unconsumed signal.** A table conditioned on nothing but (next anchor token, distance) — zero past context — hits **32.7% at offset 3**, statistically tied with the entire shipped causal store at that position (35.0%). The monotone gradient (14.2 → 19.0 → 32.7) is syntactic anticipation: positions immediately before a content anchor are heavily determined by it. No mechanism in the current architecture consumes any forward context; whatever does (E_f/E_b routing per #393) has this margin available before any cleverness.
2. **The store's margin over a bigram table is ~6pp on this slice** (34.7 vs 28.7) — consistent with #391's memorization finding, and a sobering baseline for how much the full geometric store currently adds over the cheapest causal null at the infill task.

Caveats: fixture distribution (teacher-forced stories); single stride (4); top-1 only (no bits ladder yet); the fwd-anchor null backs off to unigram when the (distance, anchor) key is unseen. Stride sensitivity and the natural-corpus rerun (#381's wiki obs, once finalized) are the next batch.


### comment 5173356308 — 2026-08-04T00:56:41Z — Casey-allard

## Stride sweep (gate item 1) — harness now takes `R4_INFILL_STRIDE`

Same fixture protocol; anchor cadence swept 3/4/5:

| stride | free targets | shipped store | fwd-anchor null | fuse:product | fusion gain |
|---|---|---|---|---|---|
| 3 | 67,046 | 35.1% | 25.9% | **46.7%** | +11.6pp |
| 4 | 75,385 | 34.7% | 21.9% | **43.4%** | +8.7pp |
| 5 | 80,436 | 35.0% | 18.6% | **41.3%** | +6.3pp |

Three regularities:

1. **The causal store is stride-invariant** (35.1/34.7/35.0) — as it must be; it never sees anchors. Clean internal consistency check for the harness.
2. **The pre-anchor position is worth ~50% at every stride** (51.1 / 50.5 / 49.1 fused) — the anchor's backward syntactic shadow has fixed strength; what changes with cadence is only how much of the stream sits inside it.
3. **Fusion gain scales smoothly with anchor density** (+11.6 → +8.7 → +6.3pp) and never vanishes: even at stride 5, one forward token is worth +6pp over everything the causal store knows.

Product-region fusion tracks token fusion at every stride (region result on #399 holds across cadence). Anchor cadence is therefore a *dial* — router workload vs syntax quality — not a cliff, which is what the hybrid deployment needs it to be.

Remaining gate items: bits ladder; natural-corpus rerun (queued behind #381).


### comment 5175134655 — 2026-08-04T05:50:00Z — Casey-allard

## Natural-corpus rerun (interim, honest): fixture-compiled geometry does not transfer; the criterion needs naturally-compiled artifacts

Full harness rerun on the wiki obs corpus (667,071 records / 3,000 articles, 102,673 held-out free targets, stride 4), with tables built from the natural construction partition but the store/codes from the FIXTURE-compiled artifacts:

| arm | fixture | natural (fixture geometry) |
|---|---|---|
| shipped store | 34.7% | **0.2%** (34.7 bits vs unigram's 10.2) |
| null: unigram | 6.3% | 4.5% |
| null: bigram | 28.7% | 4.2% |
| null: trigram | 34.7% | 4.1% |
| null: fwd-anchor table | 21.9% (off3 32.7) | 4.1% (off3 4.6) |
| null: fwd-region-d1 | 7.6% | **4.5%** |
| fuse: product | 43.4% | 1.6% |

Three findings, stated plainly:

1. **Transfer collapse.** The fixture-compiled addressing scores BELOW the unigram null on wiki text (0.2% / 34.7 bits): the store geometry is bound to its compile distribution — the architecture-level echo of #391. Any natural-corpus grading of this criterion requires artifacts compiled on the natural corpus (teacher compile over the wiki obs stream); that is the pending decider run.
2. **No token-granular channel beats frequency on open text at this scale** — including the forward-anchor table (4.1% vs unigram 4.5%), whose 33% pre-anchor shadow on stories does not appear here. Either the anticipation signal is absent on encyclopedic text at 534k training positions, or it lives at a granularity these raw token tables can't reach. The stories-vs-wiki contrast is itself informative: the syntax-completion structure the criterion measures is distribution-dependent.
3. **The first genuine sparsity signal for region-vs-token (#399 step 2's open question):** with 17% of token keys unseen (vs 2% on fixture), region-conditioning now *beats* token identity (fwd-region-d1 4.5% vs token 4.1%) — direction confirmed, but at frequency level, so it reads as "geometric bucketing rescues sparse keys," not yet "geometry carries syntax."

Reconciliation item (open): the naturally-COMPILED score pipeline posts 36.5% on this same corpus's held-out positions (#381 close-out table) while these tables sit at 4–5% — different mechanisms (EXCT window matching vs (anchor,distance) tables) and slices; understanding exactly where those 30pp live is the sharpest open analysis question, and the naturally-compiled harness rerun answers it.


### comment 5180902071 — 2026-08-04T15:05:18Z — Casey-allard

## Morning decider campaign — complete record (naturally-compiled artifacts, evidence-source correction, granularity triage)

Runs: naturally-compiled legacy artifacts (κ `afb50d0f…`, compiled on the wiki obs corpus) + observed-evidence arms + M0 region-walk arms + d1/d2/d3 region-match grading, on both corpora. Harness PR to follow.

**1. Last night's "transfer collapse" was mis-attributed — the cause is EVIDENCE SOURCE, not geometry.** Naturally-compiled artifacts score identically to fixture artifacts on wiki (0.1% vs 0.2%). The legacy store's evidence is teacher top-k, and stories15M is off-distribution on wiki. Swapping to observed evidence (actual next-token counts, same codes): store rises 0.1% → 4.0%. The symmetric fixture check: observed 33.6% vs teacher-evidence 34.7% — teacher evidence is mildly better on-distribution, catastrophic off. **This also closes the 30pp reconciliation:** the score pipeline's 36.5% on the same held-out is EXCT corpus-observed continuation matching; the store was aggregating teacher opinions.

**2. Natural-text floor verdict (token-exact, 102,673 free targets):** unigram 4.5% | bigram 4.2 | trigram 4.1 | fwd-anchor 4.1 | region tables 4.1-4.5 | observed store 4.0 | M0 4.1 | best fusion 4.1. **Nothing beats frequency; only EXCT verbatim matching (36.5%, #381 record) rises above it.** Product fusion over-multiplies weak channels (degrades below best-single). The fixture's rich anticipation structure (33% anchor shadow, +8.7pp fusion) is a property of the low-entropy story distribution.

**3. Granularity direction (#393, Casey-directed triage): measured NULL.** d2/d3 region-match is vacuous (code space exceeds vocab — no coarsening; caught by null check). d1 (256 buckets) is discriminating on fixture (trigram +7.0pp, observed store +4.7pp over the 61.2% null). On natural text every mechanism routes BELOW the 25.1% d1 null (trigram 19.4, observed 17.4, M0 14.3, fusion 13.6): no semantic-neighborhood signal invisible to token-exact scoring. Closed by measurement.

**Direction status per the three-way triage:** granularity CLOSED (above); **scale OPEN** (534k open-text positions is small; the signal may need 10-100×); **substrate OPEN** (EXCT's 36.5% marks where predictive mass actually lives — geometry's job may be organizing generalization BETWEEN verbatim anchors, i.e. extending the score-pipeline world, not the legacy store).

M0 verdict on #399.


### comment 5181451586 — 2026-08-04T15:51:57Z — Casey-allard

Closing: the criterion is complete and its record is full. Delivered across #396/#402/#405/#408: the harness (merged, in-tree at crates/uor-r4-graph-certify/tests/anchor_infill.rs with env-configurable corpus/artifacts/stride/partition), Day-0 baselines and null ladder, bits ladder, stride sweep, rows-inclusive baseline, natural-corpus reruns under both partitions, evidence-source correction, granularity triage (d1 valid, closed null on natural text), and the substrate resolution posted on #393 (induced-cover geometry at 36.5% vs legacy at the floor). The criterion did exactly what it was filed to do: it prevented the program from building on fixture-only signal, attributed every collapse to its mechanism, and located the geometry that actually works. Scale-corpus reruns (10k observation in progress) will use the same in-tree harness; further criterion evolution belongs to the #393 program and successor issues.


---
_Comment/review ids in this file: 5172937871, 5173356308, 5175134655, 5180902071, 5181451586_

