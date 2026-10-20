# #360 — certify: measure on-chain vs off-chain residual structure

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T01:16:27Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-03T01:40:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/360
- **Merged:** 2026-08-03T01:40:51Z
- **Merge commit:** 35f83a36c51bc1ae4cb4453b4ef23e2753188b75
- **Base / Head:** main / UOR-Foundation:graph-chain-selection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #358, on a fresh branch off `main` as AGENTS.md requires. Adds the
measurement that identifies which subsystem is actually at fault.

## Result

Residual accrues only on `selected_chain` nodes; tokens emitted by active or
predicted nodes keep residual exactly `ZERO`.

| slice | zero-residual candidates | teacher **on** chain | winner **off** chain |
| --- | ---: | ---: | ---: |
| exact_context | 0.00% | **79.60%** | 0.00% |
| **graph** | **89.86%** | **1.35%** | 11.29% |
| novel | 100.00% | 0.00% | 100.00% |

89% of graph-slice winners are on chain; the teacher token is on chain **1.35%**
of the time. With #358's recall figure that gives the whole picture:

- candidate generation finds the teacher **63%** of the time, as an off-chain
  candidate with residual exactly zero;
- the selected chain's emissions contain it **1.35%** of the time;
- the scorer boosts chain emissions, promoting tokens that are not the answer.

**The residual is not broken — it faithfully amplifies a chain that is selecting
the wrong regions.** That is why the α sweep in #358 found no weighting that
helps: tuning, negating, and rescaling all fail because the term is a correct
amplifier of a wrong input.

`exact_context` is the control: 0% zero-residual candidates, teacher on chain
79.6%, same machinery monotonically helpful there.

## The question this opens

**Why does chain selection pick regions whose emissions exclude a token the cover
already retrieved?** Candidate generation and chain selection consult the same
cover and disagree 98.65% of the time on this slice. That is upstream of the
scorer and is where the graph path's ~10-bit coherence gap should be attacked.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
and the certify suite (11 files) are green on this base. Figures come from the
Gate C fixture-corpus run (`scripts/gate_c_trend.sh`'s path).

## Corrections carried over from the #358 thread

- α = −1 was described there as "indistinguishable from absence"; what was
  measured is the same *agreement rate*, not the same decisions.
- A proposed mechanism in `compute_residual_deltas` (missing parent entry
  defaulting to `ScoreQ::ZERO`, breaking telescoping) is withdrawn: that function
  has no callers outside its own tests.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5161418044 — 2026-08-03T01:35:14Z — auser

## Chain selection is exonerated — and the "retrieval works" framing in #358 was wrong.

Chain selection ranks by chain **length** first, with membership margin only
breaking ties among equal-length chains, so a deep chain the context barely
belongs to beats a shallow one it fits well. The hypothesis was that this
discards better-fitting regions which could have supplied the teacher token.

**It does not.**

| graph slice | |
| --- | ---: |
| teacher emitted by an active region OUTSIDE the selected chain | **2.95%** |
| …and that region also fits the context better than the chain | **0.59%** |
| mean selected-chain depth | 2.73 |
| mean depth of best teacher emitter | 2.32 |

Chain selection is discarding usable evidence in 0.59% of positions. It is doing
about as well as the cover allows, and the depth-first rule is not the defect.

### The correction this forces

Teacher on chain is 1.35%; off-chain active is 2.95%. So **active regions emit
the teacher only ~4.3% of the time** — against the 63% candidate recall measured
in #358.

The candidate set is `active ∪ predicted ∪ root_top`, where `root_top` is the
global prior's top-B (B = 256). So the teacher is arriving through the prior, or
through predicted cloud F — **not through region emissions**.

That corrects what I recorded in #358. I wrote "retrieval works, ranking does
not," attributing the 63% to the graph finding the answer. Retrieval works
*because of the global prior*. The graph's regions rarely emit the answer at all,
and the scorer then boosts those regions' emissions over a candidate list whose
useful content came from elsewhere.

### Open sub-question

This measured off-chain **active** regions only, so the remaining ~59% is not yet
split between predicted cloud F and `root_top`. That split is the next
measurement and it decides where the defect lives:

- mostly `root_top` → region emissions carry almost none of the answer, and the
  question is compiler-side emission construction;
- substantially predicted F → the transition machinery is finding the token that
  the regions themselves do not emit, which is a different and more hopeful story.

### Hypotheses refuted on this thread so far

Root dominance; sign error; scale error; the `compute_residual_deltas`
telescoping mechanism (dead code, no callers); and now depth-first chain
selection. Five. The findings that have survived are the ones I did not predict,
which is roughly the expected yield and the reason each step was measured rather
than argued.

### Note on this PR

The commit carrying this measurement is not pushed here — I enqueued this PR and
then committed to its branch, which the merge queue correctly rejects. That is
the second time in this session, after I had already diagnosed and written up the
same mistake on #358. The work goes on a fresh branch off `main` once this lands.



---
_Comment/review ids in this file: 5161418044_

