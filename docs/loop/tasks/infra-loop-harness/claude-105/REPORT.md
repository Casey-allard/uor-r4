# REPORT: infra-loop-harness, claude-105

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2819.

## What did not work, first

- **claude-104's scope fix locked the whole discrete crate for hours.** It added `crates/uor-r4-discrete/**` to `carrier-credit-hard-selection` so claude-103 could build. But `loop claim` reads an open order's item scope, so every discrete item became unclaimable, for every lab, while claude-103's GPU arms ran. Nine ready items were blocked, five of them on the M1 path. claude-104's decision card and its review did not foresee this.
- **This repair has a cost.** #3203's crate edits now show as an advisory `scope` WARN. If its exact-head review asks for a code fix, the glob has to be restored first, through this umbrella.

## What changed

`docs/roadmap/items/carrier-credit-hard-selection.md`:
- `scope:` no longer holds `crates/uor-r4-discrete/**`;
- one note gives the reason and the way to restore the glob.

`docs/roadmap/README.md` does not render scope, and `roadmap render` left it unchanged.

## Measurements

None. This is a records unit, so there is no run, no GPU and no model.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 92 items OK. `roadmap render` changed no file.
- `loop report --check`: PASS. `loop pr-check --local`: PASS (the review-record and required-checks rules report N/A in a local run).

## Skills used

- scope-guard: the item keeps exactly the docs globs its remaining work needs.

## Still open

- **After the merge,** `loop next` must no longer list discrete items as blocked by this item. That is checked in claude-103's next session step.
- **A candidate harness change, for the harness's owner:** `loop claim` could count only the paths an open order has not yet merged, or an order could release paths explicitly, so a long GPU run would not hold a code lock it no longer needs. Not done here: the harness's own files are outside this unit.

## Plain summary

The carrier-credit work had reserved the whole discrete code folder, and that stopped every lab from starting other discrete tasks while its GPU runs ground on. Its code is finished, so this hands the folder back.
