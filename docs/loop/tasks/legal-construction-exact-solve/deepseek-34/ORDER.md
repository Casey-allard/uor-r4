# ORDER: legal-construction-exact-solve, deepseek-34 (repair: correct the cross-lab citation)

Lab: deepseek. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `ea257ef40`. Repair of PR #3013 at `ef0aced3f` after the counted CHANGES by claude (`#3013` review record). Parent: [deepseek-33/ORDER.md](../deepseek-33/ORDER.md).

## Goal

Correct one factual error the reviewer caught: the deepseek-33 record cited `local/claude/legal-case/root-lp-run1.log` as an independent **dense-tableau** run that had failed after 72+ minutes. That log is actually **claude-50's bounded-variable revised simplex** (`blp`, commit `95328fb27`, now visible on PR #3015): it started at 2026-10-22T22:05:45Z under a declared **7,800 s cap**, was still running inside that cap when first cited, and its objective had turned negative after ~250 degenerate pivots. The deepseek-33 account misnamed another lab's in-progress method and recorded a premature negative for it.

This repair keeps only deepseek's own measurement (the unit-2 dense exact `solve-case --stage lp`, 120 s cap, no result) and removes or explicitly re-characterises the cross-lab citation everywhere it appears. It makes no claim about claude-50's run or its outcome; that belongs to #3015's record.

## Cancel probe (run first)

Class **instrument** (docs-only). At the base: the counted CHANGES at `ef0aced3f` stands; the citations are in `README.md:29`, `docs/mechanisms/legal-construction.md:75`, the notebook row, and `deepseek-33/REPORT.md:7,34,57`; `ORDER` deepseek-33 line 35 also says "the run timed out", which was not true when written. Task stands.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**`
- `docs/mechanisms/legal-construction.md`
- `docs/research/notebook/legal-construction.md`
- this task folder

## Done when

- The lab README states only the deepseek measurement (dense `solve-case --stage lp`, 120 s cap, no objective/status line) and drops the "independent dense run / 72+ min" claim; if the log is mentioned at all, it is identified exactly as claude-50's bounded-variable run under its own 7,800 s cap and its outcome is deferred to #3015.
- The mechanism paragraph and the notebook row carry the same correction (no claim about another lab's solver).
- `deepseek-34/REPORT.md` records the correction, including that `deepseek-33/ORDER.md:35` ("the run timed out") was a false factual claim when written and is retained only as history.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a non-author exact-head review record pass at the new head.

## Stop and ask if

- Any file outside the scope above must change, or the correction would require re-running any solver (it does not; no compute is involved).