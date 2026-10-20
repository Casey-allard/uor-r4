# #488 — Phase timing for the stages BEFORE Gate C — that is where a real 2.11M run's hours live

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T22:18:47Z
- **Updated:** 2026-08-08T16:20:36Z
- **Closed:** 2026-08-08T16:20:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/488
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#471 added a per-phase wall clock to `evaluate_gate_c` because "the Gate C phase took eighty-five minutes" was an unattributable number that two proposals blamed on two different passes, neither measured. It worked: the profile overruled the issue's own diagnosis (the named table builds were 0.8% of a sampled run; one uncached pass was 59%).

#483 then used that instrument on a 2.11M corpus and found Gate C there costs **51.94s**, not eighty-five minutes — and that core-time per record *falls* with scale (204 µs at 500k → 152 µs at 2.11M). So the eighty-five minutes was never Gate C.

**Which means the same problem now sits one level up, unfixed.** The stages around Gate C — cover induction, store build, forward-transition compilation, emission compilation, artifact emission — have no timing at all. On a real corpus at 2.11M that is where the hours live, and nobody can attribute them.

## The argument for doing it

It is the identical argument that paid twice already: *a harness whose cost is invisible gets optimized by argument rather than by measurement*. On this repo that is not a hypothetical — we have now blamed the wrong phase twice in a row, once in the issue that filed the complaint and once in the record that answered it.

The instrument is also already written. `PhaseLog` in `crates/uor-r4-graph-certify/src/score.rs` is thirty lines, prints to stderr, and is deliberately not report content (a duration in `score_report.json` would differ between two runs of identical pinned inputs, which the deterministic-rebuild gate exists to catch). Extending it is mostly placing `mark()` calls in `run_score`.

## Deliverable

Phase timing across the whole `score` pipeline, same conventions as #471:

- cover induction
- graded store build (and whether the #469 κ sidecar hit or missed — that alone is a 625s-vs-39s fork on a cold run)
- forward-transition compilation
- context-row and forward-anchor-row compilation
- emission compilation
- R4G1 artifact emission
- Gate C (already instrumented)

stderr only, always on, one line per phase with a cumulative total, matching the existing `gate c phase:` format so both read as one log.

## Definition of done

A `score` run on any corpus prints where every minute went, and the sum of the phases accounts for the wall clock without a large unexplained remainder. The remainder itself is the check: if the phases sum to noticeably less than the run, there is another unnamed stage and the instrument is not finished.

## Worth knowing before starting

The wiki10k 2.11M observation set **no longer exists on disk** — `/tmp` was cleared, `.uor-models/corpora/` tops out near 177k records, and regenerating it means re-fetching through `scripts/fetch_d3_corpus.py`, which warns that upstream drift makes the manifest's recipe non-reproducible. #483 worked around this with the 500k fixture replicated 4.22×, which is valid **only** for content-independent per-record costs like the code pass; it is degenerate for cover induction and scoring (44 regions, every position ExactContext), which are exactly the stages this issue is about.

So this issue probably needs a real corpus at scale, and sourcing one is part of the work. That is worth knowing up front rather than discovering at the measurement step.

**Unassigned.**


## Comments (2)

### comment 5224294167 — 2026-08-08T03:28:52Z — Casey-allard

Instrument delivered in PR #495 (merged, main `b1a4641`). Phase timing now covers the whole `score` pipeline, not just Gate C: `PhaseLog` is generalized (pub + prefix), `score_command` marks every stage, stderr only (a duration in the report would break deterministic-rebuild).

Verified on the 500k fixture — the score phases sum to **359.69s with zero remainder**, so no stage is unnamed (the remainder is the DoD's own completeness check). Attribution it produces immediately: Gate C's **scoring pass dominates overall (247s / 100,306 positions)**, and within the pre-Gate-C stages **cover induction is largest (27.88s)**; the graded store build is only 1.78s because the #469 code sidecar HIT (vs the ~625s cold path).

**KEPT OPEN.** The 500k run verifies the instrument; attributing a real **2.11M** run's hours — the actual point of this issue — still needs a corpus at that scale. The wiki10k 2.11M observation set is gone from disk and `scripts/fetch_d3_corpus.py` warns its recipe is non-reproducible (upstream drift), so sourcing one is the remaining work. Left unassigned so it can be picked up freely.


### comment 5226971435 — 2026-08-08T16:20:35Z — Casey-allard

## Closing — DoD met on `main` (b0f72ec)

The phase-timing instrument is complete and covers the whole `score` pipeline, not just Gate C. In `crates/uor-r4-graph-cli/src/lib.rs`, `score_command` runs a `PhaseLog::new("score phase")` with `mark()` calls at every stage the DoD enumerates:

- `inputs + observations (load, split, observe)`
- `cover induction`
- `graded store build (+ code sidecar)` — the #469 κ sidecar hit/miss is visible here (the 625s-vs-39s fork)
- `forward transitions`
- `context + forward-anchor rows`
- `emission compilation`
- `R4G1 artifact emission`
- `gate c evaluation` (the #471 instrument, `"gate c phase:"`, nested)
- `report build + artifact write`

Same conventions as #471: stderr only, always on, one line per phase with a cumulative total; deliberately **not** report content, so it does not perturb the deterministic-rebuild gate.

**DoD check (the remainder test).** On the 500k fixture the phases sum to the wall clock with **zero unexplained remainder** (359.69s, Gate C scoring dominant at ~247s), verified when the instrument landed in **PR #495** and re-confirmed by code read on current `main`. "A `score` run on any corpus prints where every minute went, and the sum of the phases accounts for the wall clock without a large unexplained remainder" — met.

**Residual, filed not parked.** The *motivating* use — attributing the hours on a real **2.11M-record** corpus — cannot be exercised because that corpus no longer exists on disk, and #483's 4.22× replication is degenerate for the content-dependent stages (cover induction, scoring) this was about. That at-scale measurement is now **#503**, unassigned. Per closure discipline the instrument's DoD is met, so #488 closes here and the scale run lives in its own issue.

Delivered: PR #495 (merged). Follow-up: #503.



---
_Comment/review ids in this file: 5224294167, 5226971435_

