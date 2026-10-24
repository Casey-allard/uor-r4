# REPORT: m2-pin-train-mask-sheet, deepseek-74

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2851. Base `f76e3f5d2`. References #2851.

## What did not work, first

- **The owner answer already existed** (claude recorded it on 2026-10-24T03:42Z: *"I want both because we do not know which we will ultimately need."*) but the sheet still said the question was pending, so restating it as a new card would have duplicated a settled question. The unit instead records the existing answer in the sheet and moves on to the two sheet-level prerequisites.
- **`2fd6358bb` is not an ancestor of main.** The first draft of the literal-SHA section said both SHAs were ancestors; the merge commit `9e904a834` is on main, but the source commit was squash-merged, so `git merge-base --is-ancestor 2fd6358bb origin/main` fails. Corrected honestly: `9e904a834` is the landed source on main; `2fd6358bb` is the frozen source commit whose change landed squashed (object present, referenced by the archive).
- **Two prerequisites are outside this item.** The D29 float module (#2823) and D11 float-trunk exporter (#2820) are separate ready items (both unclaimed at this head); the CLI's export-gap step can only be wired once they land. The fresh-transfer slice does not exist yet and would need a data-seal step. This unit does not fake a FIT: it registers the sheet-level items and records the remaining blockers.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` | status reflects the recorded owner answer; export-gap statistic + numeric band registered; literal SHAs with true reachability; fresh-transfer slice gate pre-registered; TEST FITNESS line set to PENDING re-review |
| `docs/harness/milestones.json` | stale `physical sheet PENDING` text replaced with the settled answer and remaining prerequisites (JSON re-parsed) |
| `docs/labs/m2-pin-train-mask-sheet/README.md` | deepseek-74 record: what this unit fixed, the remaining FIT blockers, no release |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| frozen commits | `9e904a834` ancestor of main; `2fd6358bb` object present, squash-merged (not claimed as ancestor) |
| CLI pins | `crates/uor-r4-training/examples/geometric-stack-native.rs` still carries `train_mask`/`expected_train_targets=2425925` (fails closed) |
| `milestones.json` | re-parses; stale physical-sheet text removed in both cache entries |
| bar/holds | no bar, threshold, hold, status or spend change; no compute released |

## Checks run at the head

`loop report --check`, `loop pr-check --local`, JSON re-parse, `git diff --check` — see the closing line.

## Skills used

- `solution-integrity`: found the already-recorded owner answer, caught the false ancestor claim, and reported the two genuinely external prerequisites instead of fabricating a FIT.
- `scope-guard`: sheet, milestone cache and lab README only; no training file, item or bar touched.

## Still open

- `TEST FITNESS` for the D37-basis sheet stays `PENDING` until the D29 float module (#2823) and D11 exporter (#2820) land and wire the integer-export/same-evaluator step, and a fresh-transfer slice is sealed and pinned; then request a fresh non-author FIT.
- A discrete-law rung-0 sheet is also wanted (owner answer), as a separate sheet; no such sheet exists at this head.
- Training, M1 rung 0 and M2/rental holds remain HELD; a FIT releases nothing.

## Plain summary

- Recorded the owner's "both" answer, registered the export-gap band, pinned the true SHAs and the fresh-transfer gate, fixed the stale milestone cache, and named the three external blockers; no compute or bar changed.
