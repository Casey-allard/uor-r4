# REPORT: infra-loop-harness, claude-108

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2819.

## What did not work, first

- **The crate glob has now changed hands four times in one day** (claude-104 added it, -105 removed it, -106 restored it, and this unit removes it). Each change was a sound local call. Together they show that a lock tied to an item's scope fits a long GPU run badly. The candidate harness change in claude-105's REPORT stands: `loop claim` could count only the paths an open order has not yet merged.
- **The results unit can no longer fix the crate.** If grading the eight arms finds a defect in `carrier-arms grade`, the fix goes to its own unit with its own claim, after claude-107 (#3226, which holds the crate for Stage T) releases it or by agreement with it.

## What changed

`docs/roadmap/items/carrier-credit-hard-selection.md`:
- `scope:` no longer holds `crates/uor-r4-discrete/**`;
- it now holds `docs/research/notebook/m1-discrete-attention.md` (the item's `notebook:`) and `docs/mechanisms/carrier-credit.md` (the tried table), the two pages its results unit writes;
- one note gives the reason.

No ready item's scope holds either added page; only `m1-bvg-k-block-apply-c2` (done) holds the notebook. `roadmap render` left `docs/roadmap/README.md` unchanged.

## Measurements

None. This is a records unit, so there is no run, no GPU and no model.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 92 items OK. `roadmap render` changed no file.
- `loop report --check`: PASS. `loop pr-check --local`: PASS (the review-record and required-checks rules report N/A in a local run).
- A fresh decision review (D68) and exact-head review: APPROVE, no Required findings. It confirmed that the overlap test is a literal prefix (`loopclaim.rs:617-634`), so removing the glob ends the overlap with claude-107. It also confirmed that `counts()` skips notebook paths (`loopclaim.rs:637-638`), so the added notebook page cannot lock anything.

## Skills used

- scope-guard: the item holds exactly the docs its remaining work writes.

## Still open

- **After the merge,** `loop claim carrier-credit-hard-selection` must not be refused for a crate overlap. The results unit's claim checks this.
- **The results unit** grades the arms once CONTROL-U (pod B) and CONTROL-F seed 2001 (pod C) finish.

## Plain summary

The carrier-credit code is merged, so its task gives the code folder back for good. What it still writes, the results write-up, is listed instead.
