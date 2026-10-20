# #483 — Attribute the unexplained 2.11M Gate C residual (#471 follow-up)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T18:26:52Z
- **Updated:** 2026-08-07T21:31:31Z
- **Closed:** 2026-08-07T21:14:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/483
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#471 shipped a knob that removes a measured 59% of a sampled Gate C run, and closed **without explaining the number that provoked it**. This issue owns that residual.

## The gap, precisely

The #460 STAGES=5 decision run reported the Gate C phase at **~85 minutes** at 2.11M records with `R4_GATE_C_SAMPLE=10000`. The #471 profile (`docs/gate_c_arm_skip_471.md`) measured the dominant pass at two corpus sizes:

| records | right-context code pass | µs/record |
|---:|---:|---:|
| 250,000 | 26.42s | 105.7 |
| 500,000 | 51.05s | 102.1 |

Linear to within 3.5%. Extrapolated to 2.11M on the same two-core box: **~215s — about four minutes.** Even allowing STAGES=5 its 1.25× on stage count, and even attributing the whole rest of the Gate C phase generously, the model does not reach 85 minutes. Something is missing by more than an order of magnitude.

## What has already been ruled out

**Memory pressure — measured negative.** It was the leading suspicion and it did not survive contact. Peak RSS over the whole `score` process, 500k fixture, sampled at 10,000:

| | control | `right_context` skipped | delta |
|---|---:|---:|---:|
| peak RSS | 1,382 MB | 1,305 MB | 77 MB (5.6%) |

Seventy-seven megabytes at 500k scales to a few hundred at 2.11M even with STAGES=5's extra levels. That is not the difference between four minutes and eighty-five on a 16GB machine. Recorded as a negative so nobody re-proposes it.

## Candidates, in the order I would check them

1. **The 85 minutes may not all have been inside `evaluate_gate_c`.** Nothing at the time could separate the Gate C evaluation from the cover induction, store build and emission stages around it. This is now the leading candidate *because* it is the one nobody could rule out — which is also the reason the instrument below exists.
2. **Per-record cost at 2.11M may simply not be 102 µs.** The #469 record has a whole-corpus code pass at 2.11M costing 625s pre-lever-B — roughly 11× more core-time per record than #471 measured at 500k. Either that figure covers more than the code pass, or per-record cost depends on something a 2× corpus range does not expose. Both are worth knowing; neither is settled.
3. **STAGES=5 per-record cost.** Worth 1.25× on stage count alone, so not sufficient by itself, but it compounds with (2).

## The instrument already exists

`evaluate_gate_c` prints a per-phase wall clock to stderr on every run (#471, always on, not report content — a duration would make `score_report.json` differ between two runs of identical pinned inputs, which the deterministic-rebuild gate exists to catch). The phases are: scorer construction, forward-anchor table, right-context code pass, left code pass, two-sided build, latent build, unigram null, scoring N positions, reduction.

So this issue does not need new tooling. It needs **one real 2.11M run** with the timing on.

## Definition of done

A 2.11M Gate C run whose phase lines are recorded against the 85-minute report, and one of:

- the residual lands inside `evaluate_gate_c` and a named phase owns it → that phase becomes the next optimization target, with the arithmetic in hand;
- the residual lands outside `evaluate_gate_c` → the 85 minutes was never a Gate C number, `docs/gate_c_arm_skip_471.md` gets that correction, and the cost moves to whichever stage actually holds it;
- the numbers reproduce the linear model and the 85 minutes does not recur → the original report was environment-specific, which is itself worth pinning down before the next long run is planned around it.

All three outcomes change what happens next, so the run has decision value under the `AGENTS.md` long-run discipline. Cost: one 2.11M compile + score. Nothing else blocks on it.

**Unassigned.** Filed per #471's closing record rather than left in a doc, so it has a home.


## Comments (1)

### comment 5222207868 — 2026-08-07T21:31:28Z — Casey-allard

Answered in **#485**, folded into `docs/gate_c_arm_skip_471.md` (the record that opened this question, so the loop closes where a reader looks for it).

## The measurement

2.11M records, `R4_GATE_C_SAMPLE=10000`, STAGES=4, 8 cores, #471 phase timing on:

| phase | seconds |
|---|---:|
| scorer construction | 0.59 |
| forward-anchor table (#399 M2) | 0.12 |
| **right-context code pass (#446)** | **40.07** |
| left code pass (#469 lever A) | 0.50 |
| two-sided table build (#446 M1) | 0.19 |
| latent right-context tables (#446 M2) | 0.36 |
| unigram null (#390) | 0.05 |
| scoring 10,000 positions | 9.74 |
| reduction, rollups, replay probes | 0.32 |
| **Gate C total** | **51.94** |

Whole `score` pipeline: about 5m52s.

**Fifty-two seconds, not eighty-five minutes.**

## Against the three candidates this issue listed

**Candidate 2 — "per-record cost may simply not be 102 µs at 2.11M" — is DEAD.** In core-time per record the right-context pass costs 204 µs at 500k on two cores and **152 µs** at 2.11M on eight. It got *cheaper* per record at 4.2× the corpus. There is no scale cliff in this pass, and the linear model from the #471 record transfers.

**Candidate 3 — STAGES=5 — cannot close the gap.** Worth 1.25× on stage count. Against a ~100× discrepancy it is not a contributor worth chasing.

**Candidate 1 stands, by elimination.** The 85 minutes was not the Gate C evaluation. At the time nothing could separate `evaluate_gate_c` from the cover induction, store build, transition and emission stages around it, and the number was attributed to the phase that happened to have a name.

That reframes #471 honestly, and the record now says so: the arm-skip knob removes a real 62.9% of a sampled Gate C run, and it was **never the fix for the reported 85 minutes**. Both of those are true at once, and shipping the first is not a reason to stop saying the second.

## Scope limit, and it is a real one

The original wiki10k observation set no longer exists on disk (`/tmp` was cleared; `.uor-models/corpora/` holds nothing near that size, and regenerating it means re-fetching 10k articles through a script that warns about upstream drift). So this 2.11M corpus is **the 500k fixture replicated 4.22× with renumbered story ids**.

That corpus is degenerate for quality — 44 regions, every scored position ExactContext, candidate recall 99.8% — so the **scoring** and **cover induction** costs it reports are unrepresentative of a real corpus at that size, and I am not quoting them as such.

The right-context code pass is not affected: it is a fixed amount of work per record (`bundle_window_plain` + assign) with no dependence on corpus content. That is exactly why a replicated corpus is a valid probe for the one quantity this issue was filed to settle, and it is the only quantity I am claiming from it.

## What this hands to whoever picks it up next

The phase timing covers Gate C, and Gate C is now measured not to be where the time goes at scale. The useful next move is to extend the same treatment to the stages around it — cover induction, store build, transitions, emissions — because that is where a real 2.11M run's hours actually live, and right now nobody can attribute those either. Same argument as #471's: a harness whose cost is invisible gets optimized by argument rather than by measurement, and we have now been wrong about which phase to blame twice.

Not filing that myself — it wants an owner and a scope decision.



---
_Comment/review ids in this file: 5222207868_

