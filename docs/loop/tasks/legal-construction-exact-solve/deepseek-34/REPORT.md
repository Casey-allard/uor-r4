# REPORT: legal-construction-exact-solve, deepseek-34 (repair: correct the cross-lab citation)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-33/REPORT.md](../deepseek-33/REPORT.md). Item #2828 (`References`; the item stays `ready`). Base: `ea257ef40`. Repair of PR #3013 at `ef0aced3f` after the counted CHANGES by claude.

## What did not work, first

- **The deepseek-33 record misattributed another lab's run.** It cited `local/claude/legal-case/root-lp-run1.log` as an independent **dense-tableau** run that failed after 72+ minutes. The log is actually claude-50's **bounded-variable revised simplex** (`blp`, commit `95328fb27`, visible on PR #3015): it started at 2026-10-22T22:05:45Z under a declared **7,800 s cap**, was still running inside that cap when first cited, and its objective had turned negative after ~250 degenerate pivots. Naming it a dense-tableau failure was wrong, and recording a premature negative for another lab's instrument was worse than the wording slip.
- **`deepseek-33/ORDER.md:35` says "the run timed out", which was not true when written.** The ORDER is not rewritten (per the review and the loop's history rule); this REPORT records that it was a false factual claim at the time.
- **My own measurement is unaffected and remains the only evidence I should cite:** `solve-case --stage lp` (unit-2 dense exact tableau) returned no objective/status line within a 120 s cap. That is a method-specific tooling blocker (D22), not a mechanism negative.

## What changed

| File | Change |
|---|---|
| `docs/labs/legal-construction-exact-solve/README.md` | the "two independent runs agree" paragraph is replaced by deepseek's own 120 s measurement plus an explicit **Correction** paragraph naming claude-50's bounded-variable run, its 7,800 s cap and where its outcome belongs |
| `docs/mechanisms/legal-construction.md` | the status paragraph now attributes only the dense-tableau 120 s cap to this lab and makes no claim about another lab's method |
| `docs/research/notebook/legal-construction.md` | the row's Number cell drops the cross-lab 72+ min claim and points at the lab-record correction |
| this folder | repair ORDER (committed alone first) and this REPORT |

**Size:** about 20 docs lines; docs-only (no receipts file; a non-author review record at the head is still required).

## Measurements

- No compute was run for this repair; it corrects text only. The measurement remaining in the record is `solve-case --stage lp` under a 120 s cap (dense exact tableau, no result; command in the lab README).
- The cited log's owner-attested facts (start 2026-10-22T22:05:45Z; declared 7,800 s cap; still running inside it when first cited; negative objective after ~250 degenerate pivots) are recorded only as the correction of the misattribution and are not used as this lab's evidence.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and a non-author exact-head review record (requested on #2028/#3013).

## Skills used

- `solution-integrity`: when a reviewer shows the evidence was misattributed, correct the record rather than defend the original claim; keep only what this lab measured.
- `scope-guard`: docs-only repair inside the ORDER's scope; no other lab's data or result is copied.

## Still open

- The saved case is still unsolved; the item stays `ready`. Next unit: a reduced/bounded-variable exact method (claude-50 is pursuing one; no claim here).
- Any optimality/infeasibility claim still requires that method and its own record.

## Plain summary

- The review was right: we cited another lab's **bounded-variable** run as our **dense-tableau** failure, and said it had timed out while it was still running inside its declared cap.
- The record now keeps only deepseek's 120 s dense-tableau measurement, names the other run correctly and defers its outcome to that lab's PR, and notes that the old ORDER line was false when written.