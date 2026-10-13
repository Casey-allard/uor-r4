---
name: panel-scoring
description: Score a run on the milestone's frozen acceptance panel — locate the panel and bar, check its noise, power the run, and report per-category results and transfer instead of totals.
version: 1
harness: 1
kind: research
applies-to: [docs/harness/panels/*.json, docs/harness/milestones.json, docs/labs/*/README.md]
when: [score the panel, acceptance panel, panel noise, seeds, fresh rows, per-category, fresh transfer, headline number]
requires: [experiment-prereg]
outputs: [a scored panel result with its grader identity, per-category rows including failures, a noise statement, a fresh-transfer number for any development KEEP]
forbidden: [changing a criterion, panel or grader without an owner decision, a decisive claim from one seed or fewer than 200 fresh rows, reporting a bare total, claiming fresh transfer from development rows, a bar narrower than panel noise]
milestones: [m1, m2, m3, m4]
---

## When to use

Use when a run finishes and has to be turned into a number that can move a
headline, and again before any `KEEP`. The bar comes from D22 §3 (minimum test
power) and §4 (transfer before KEEP); the panel itself is frozen in
[docs/harness/panels/](../../panels/) and mapped per milestone in
[milestones.json](../../milestones.json).

## Do this

1. **Locate the panel and its bar before scoring anything.**
   `uor-harness panel show --milestone m1` prints the panel id,
   grader, bars, noise, seeds and rows for the milestone's
   `panel` field in [milestones.json](../../milestones.json). Copy the current
   headline from the milestone issue, not from memory.
2. **Check the panel is the frozen one and is the one pre-registered.** The
   sheet's panel id and the scored panel id must match. A run scored on a panel it
   did not pre-register against is a reading, and gets recorded as one.
3. **Read the noise before claiming any difference.** The noise is the panel's
   seed-to-seed spread, or a binomial standard error (`sqrt(p(1-p)/n)`), stated in
   the panel file. Apply D22 §3: **no bar is narrower than that noise and no bar
   is a single row.** A difference inside the noise is not a difference, whichever
   way it points.
4. **Power the run.** A decisive run uses **≥ 2 seeds**, or **≥ 200 fresh rows**,
   or both, and the mechanism must be trained in — from pretraining, or a
   fine-tune long enough for the base to adapt. Report per-seed values, not only
   their mean, so the spread can be seen. Use the frozen grader identity and
   record it; a new grader is a new panel and needs an owner decision.
5. **Score it.**
   `uor-harness panel score --panel v5-reply --value <measured> --bar <bar> --label <what was measured>`
   Compare against the bar: `KEEP` only if the pre-registered bar is met outside
   its noise. A result that improves the headline but misses its bar is kept as
   the line's new base (D22 §2), not discarded.
6. **Report per-category results and failure rows, not totals.** Break the panel
   into its categories: memory/copy, arithmetic, coding, abstention, and so on.
   Name the rows that failed and what they failed on. A total hides both the
   mechanism's success and its characteristic failure; a gain concentrated in one
   category is not a general gain.
7. **Report fresh transfer for every development KEEP (D22 §4).** A development
   gain with zero transfer is recorded as **panel fitting**, not a milestone move.
   The transfer set must be genuinely fresh — new combinations, opened after the
   design was frozen — and scored on the frozen panel's criteria.
8. **Count and report.** A run that meets its own pre-registered mechanism target
   without significant headline loss is KEEP and iterate and resets the line's
   count. A run that meets neither the headline nor the mechanism target adds one
   to the D21 count. Move the headline only when the acceptance number itself
   moves on the saved model.
9. **Record the score with its scope** in the dated record and the current-state
   entry (`evidence-record`): panel id, grader, seeds, rows, split, artifact
   identity, and the failures.

## Output

- One scored result per run: panel id and grader identity, bar, noise, measured
  value per seed, and the pass/fail row.
- Per-category results plus the failure rows.
- For a development KEEP, a fresh-transfer number from rows the design never saw.
- The headline before → after, or an explicit "unchanged".

## Do not

- Do not change a criterion, a panel, a grader or a threshold to make a result
  land; that needs an owner decision, and until it exists the bar is the frozen one.
- Do not claim a difference inside the noise, and do not quote a bar narrower
  than the panel's own spread.
- Do not report a bare total as the result, and do not omit the rows that failed.
- Do not call development rows "fresh transfer", and do not report transfer from
  a set the design was tuned on.
- Do not let a single seed carry a decisive claim, and do not present a short
  add-on to an untrained base as a powered run.
- Do not move a headline on `UNAVAILABLE` data or on an unverified adapter state;
  state the adapter status explicitly.
