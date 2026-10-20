# #503 — Phase-timing attribution on a real 2.11M-record corpus (the #488 instrument, run at scale)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T16:20:15Z
- **Updated:** 2026-08-08T20:51:19Z
- **Closed:** 2026-08-08T20:51:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/503
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

Spun out of #488, which is now complete: the `score` pipeline prints a per-phase wall clock for every stage (`graph-cli::score_command`, `"score phase:"` prefix), and on the 500k fixture the phases sum to the wall clock with no unexplained remainder (PR #495). The instrument covers inputs+observations, cover induction, graded store build (+code sidecar), forward transitions, context+forward-anchor rows, emission compilation, R4G1 artifact emission, Gate C, and report/artifact write.

## What is still not measured

#488's DoD ("a `score` run on **any** corpus prints where every minute went") is met, but the *point* of the exercise — attributing where the hours go on a **real 2.11M-record corpus** — is not, because that corpus no longer exists on disk. #483's 4.22× replication of the 500k fixture is valid only for content-independent per-record costs (the code pass); it is degenerate for cover induction and scoring, which are exactly the content-dependent stages whose per-record cost was shown to *change* with scale (Gate C core-time 204 µs at 500k → 152 µs at 2.11M).

So the open work is: source a real corpus at ≥2.11M scale (regenerate through `scripts/fetch_d3_corpus.py`, noting the manifest's upstream-drift warning, or an agreed substitute), run the instrumented `score` pipeline on it, and post the per-phase attribution — the cover-induction / store-build / emission split at scale that #488's instrument now makes readable but that no corpus currently exists to exercise.

## Definition of done

A `score` phase-log from a run on a ≥2.11M-record corpus, posted here, with the phases summing to the wall clock and the dominant stage named. If sourcing the corpus proves infeasible, record why and what substitute (if any) is defensible for the content-dependent stages.

## Not in scope

The instrument itself — that is #488, done. This issue is only the at-scale measurement it enables.

Unassigned.


## Comments (1)

### comment 5228093333 — 2026-08-08T20:51:18Z — Casey-allard

Closing — not feasible with current resources, and the instrument it depends on is already delivered.

**The instrument is done.** #488 landed the whole-pipeline phase timing and its DoD is met on the 500k fixture (phases sum to wall clock, zero remainder; Gate C scoring dominant, cover induction the largest pre-Gate-C stage).

**The at-scale run is blocked, three ways:**
1. The pinned 2.11M-record observation corpus is gone from disk.
2. `scripts/fetch_d3_corpus.py` fetches only the 3000-article Simple-English-Wikipedia *text*, and its manifest recipe has drifted upstream (non-reproducible — `--allow-drift` gets *a* corpus, not the pinned one).
3. Producing a *scored* 2.11M-record corpus needs teacher-labeled artifacts for 2.11M positions. On this measurement box the teacher runs at ~6.6 tok/s (360M) — tens of hours of generation before cover/score even begin — which is not justified for a profiling-only task.

**What's already known cheaply:** #483 characterized the content-*independent* per-record costs via the 500k×4.22 replication; the content-*dependent* stages (cover induction, scoring) cannot be validly replicated, and no real ≥2.11M corpus is sourceable here to exercise them.

So this is closed as not-feasible-with-current-resources rather than left as a standing blocker. The #488 instrument is ready to run the moment a real ≥2.11M corpus exists — re-file (or reopen) then; it's a one-command run at that point.



---
_Comment/review ids in this file: 5228093333_

