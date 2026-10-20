# #363 — certify: test whether chain selection discards usable evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T02:17:43Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-03T02:43:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/363
- **Merged:** 2026-08-03T02:43:24Z
- **Merge commit:** 6f2000d5872dc43a23798d496c9b7dd176b538b6
- **Base / Head:** main / UOR-Foundation:graph-chain-exoneration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #360, on a fresh branch off `main`. Tests whether chain selection is
the graph path's defect. **It is not**, and the measurement corrects a framing I
recorded in #358.

## Chain selection is exonerated

Chain selection ranks by chain **length** first, with membership margin only
breaking ties among equal-length chains — so a deep chain the context barely
belongs to beats a shallow one it fits well. The hypothesis was that this
discards better-fitting regions which could have supplied the teacher token.

| graph slice | |
| --- | ---: |
| teacher emitted by an active region OUTSIDE the selected chain | **2.95%** |
| …and that region also fits the context better than the chain | **0.59%** |
| mean selected-chain depth | 2.73 |
| mean depth of best teacher emitter | 2.32 |

Usable evidence is discarded in 0.59% of positions. Chain selection is doing
about as well as the cover allows; the depth-first rule is not the defect.

## The correction

Teacher on chain is 1.35%, off-chain active 2.95% — so **active regions emit the
teacher only ~4.3% of the time**, against the 63% candidate recall measured in
#358. The candidate set is `active ∪ predicted ∪ root_top` (global prior top-B,
B = 256), so the teacher arrives via the prior or predicted cloud F, **not via
region emissions**.

I recorded in #358 that "retrieval works, ranking does not," attributing the 63%
to the graph. That attribution was wrong: retrieval works because of the
*context-free* global prior. The graph's regions rarely emit the answer, and the
scorer then boosts those emissions over a candidate list whose useful content
came from elsewhere.

## Open sub-question

This measured off-chain **active** regions only. The remaining ~59% is not yet
split between predicted cloud F and `root_top`, and that split locates the
defect: mostly `root_top` → region emissions carry almost none of the answer, and
the question is compiler-side emission construction; substantially predicted F →
the transition machinery is finding what the regions do not emit.

Relevant to #362 (trigram context rows), which adds context-conditioned evidence
against exactly this gap.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
and the certify suite (11 files) green. Figures from the Gate C fixture-corpus
run.

Adds one measurement-only accessor, `GraphScorer::node_emits`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5161695557 — 2026-08-03T02:29:01Z — auser

## Attribution: 82% of graph-slice retrieval is the context-free prior. Predicted cloud F is the one graph component that works.

Candidate recall on the graph slice is 63% while active regions emit the teacher
only ~4% of the time. The candidate set is `active ∪ predicted ∪ root_top`, so
attributing the remainder locates the defect.

| graph slice (recall 62.97%) | share of positions |
| --- | ---: |
| teacher from active regions | 3.82% |
| teacher from **predicted cloud F** | **11.10%** |
| teacher from `root_top` (global prior top-B) | 52.91% |
| **teacher ONLY from the context-free prior** | **51.78%** |

**82% of graph-slice retrieval (51.78 / 62.97) comes from the bare unigram prior
with no graph contribution at all.** Region emissions add roughly 4 pp.

### The one positive finding

Predicted cloud F contributes **11.10% — nearly 3× the active regions**. The
transition machinery is finding tokens that the regions themselves do not emit.
That is a component doing genuine work, and it is worth separating from the
emission construction that is not.

So the answer to "compiler-side emissions or transition machinery?" is: emissions
are the weak part, F is the strong part, and the prior is carrying the rest.

### Consequences

1. **The graph path is largely a context-free unigram prior wearing a graph.**
   On this slice the geometric machinery supplies ~15% of retrieval (4 pp regions
   + 11 pp F) against 52 pp from the prior.
2. **#362 (trigram/bigram context rows) is aimed at the right gap.** Replacing a
   context-free prior with context-conditioned evidence is precisely what this
   measurement says is missing. It also sharpens the marginal-contribution
   condition I asked for there: with regions supplying ~4 pp, a trigram win could
   easily coexist with the geometric path contributing nothing, and the ablation
   should be able to tell those apart.
3. **Emission construction is the compiler-side question**, not chain selection
   (exonerated in #363) and not the residual arithmetic (exonerated by the α
   sweep in #358).

### Two caveats, recorded rather than glossed

- The pre-registered consistency check **does not match exactly**:
  `teacher_from_active` is 3.82% against 4.31% from the earlier on-chain +
  off-chain figures. Explainable — `covered_chain` walks ancestor nodes outside
  the active set, so "on chain" and "active" overlap without nesting — but it is
  a definitional gap, not a clean tie-out.
- **exact_context attribution is incomplete.** It reports 0% from active and
  predicted because under Rule 2 the chain is empty and retrieval comes through
  the EXCT probe, which is not among the three sources counted here. Its 79.6%
  recall is real; this breakdown simply does not cover its origin. Graph-slice
  figures are unaffected.



---
_Comment/review ids in this file: 5161695557_

