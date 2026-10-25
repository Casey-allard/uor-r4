# REPORT: infra-loop-harness, claude-106

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2819.

## What did not work, first

- **The lock comes back for a short window.** Restoring `crates/uor-r4-discrete/**` makes claude-103's open order lock every discrete item again, until #3203 merges. To keep that short, #3203 now lands as the code and design delivery, and the GPU results follow in a docs-only unit.
- **claude-105's release lasted about an hour.** That unit had foreseen this restore path, but it did not foresee that #3203's review would land a Required code fix while the arms were still running.

## What changed

`docs/roadmap/items/carrier-credit-hard-selection.md`:
- `scope:` holds `crates/uor-r4-discrete/**` again;
- one note gives the reason (the Required release-validation fix needs its negative test) and the short-lock plan.

## Measurements

None. This is a records unit, so there is no run, no GPU and no model.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 92 items OK.
- `loop report --check` and `loop pr-check --local`: PASS (the review-record and required-checks rules report N/A in a local run).

## Skills used

- scope-guard: the crate glob returns only for the review fixes, and the lock ends on #3203's merge.

## Still open

- **#3203** gets its negative test and the advisory fix, then its review, then merges, which ends the lock.
- **The results unit** for #2819 is docs-only. The glob can be removed again before it is claimed, if other labs are waiting on discrete items then.

## Plain summary

The carrier-credit code needs one more small test before it merges, so it gets its code folder back briefly, and merges right after.
