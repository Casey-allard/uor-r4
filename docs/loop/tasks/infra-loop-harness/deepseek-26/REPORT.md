# REPORT: infra-loop-harness, deepseek-26 (loop next: owner-direction source)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 1. Base: `a472d2781`. This is a loop setup PR: its exact-head review comes from Claude's fresh-context reviewers (D59).

## What did not work, first

- **The owner-direction source was the last one the plan defines as readable now.** The previous harness unit built `lane` and `premise-probes`; `gpu-collections` needs a collection record the watcher does not write and the watcher file is outside this item, so only `owner-direction` was left. It is built here with the same documented PLAN table contract.
- **One reader, two sources.** The premise reader is refactored into `plan_open_row(root, heading)` (first row of the named PLAN section whose cells include `open`), and both `premise-probes` and `owner-direction` call it; the source-specific parsing is small (backticked command for a probe, first cell for a direction). This keeps the accepted table shape in one place.
- **The existing not-built test pinned the old string**, so it was updated for the changed specified behaviour (the source now exists; with no PLAN.md it names the missing file).
- **The first version of the reader was positional and the review caught it** (`deepseek-27`): it took the first cell of any row whose cells contained `open` and the first backticked cell anywhere, which read a note cell as a status and could hide a malformed probe. The repair parses the table once, finds the Status/Probe/Direction columns by header name, skips malformed open rows and names them, and pins "before repairs" with a test that carries a failing own PR.
- **`gpu-collections` remains a named gap** (as in #2969): defining the collection record means changing `loopdrive.rs`, outside this item's scope.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | code | +~90/−45 | `plan_open_row` shared reader; the owner-direction source (first open PLAN row, before repairs); `premise_probe` rewritten on the shared reader |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | test | +~25 | the owner-direction offer test; missing-PLAN line now also asserted for owner-direction |
| `crates/uor-r4-harness/tests/loop_next.rs` | test | 1 line | the not-built test asserts the new owner-direction reason |
| `docs/harness/gates.json` | generated | +30 | the `loop-next-owner-direction` gate and its planted fault |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +2/−1 | the owner-direction contract in the source list and loop page |
| `docs/loop/tasks/infra-loop-harness/deepseek-26/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 180 changed lines against the ORDER's estimate of about 170 (1.06).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_next_sources`:** 4 passed (owner-direction offer, lane ordering, PLAN premise selection, missing-input lines).
- **`cargo test -p uor-r4-harness --test loop_next`:** 28 passed at the repaired head (the main merge added claude-31's WAIT tests; the updated not-built test included).
- **Gates:** nine GREEN including the new `loop-next-owner-direction` and claude-31's `loop-next-wait`; `--break all` nine CAUGHT with the open-row fault red at "the open owner direction must be offered before repairs".
- **No GPU work, no owner path, no spend:** the reads are `git show origin/main:PLAN.md` and the registry; nothing writes.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: nine GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: nine CAUGHT.
- `cargo test -p uor-r4-harness --test loop_next --test loop_next_sources`: all green.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: one reader, two sources, one documented table contract; the source that could not be built honestly (GPU collections) stays named rather than faked.
- **scope-guard**: only the item's scoped files and shared registration files.

## Still open

- **`gpu-collections`** needs the watcher to write a collection record; `loopdrive.rs` is outside this item, so that is a scope decision/follow-up.
- **Lane activation** waits on the owner adding `lane` to `labs.json` and the PM backfilling item `lane:`/`rank:` keys.
- **owner-direction/premise activation** waits on the PM's `PLAN.md` using the documented table shape.

## Plain summary

- **`loop next` now reads owner directions too.** The first open row of PLAN's `## Owner directions` table is offered as the top-priority session before repairs, using the same table reader as the premise probes; absent PLAN.md or no open row is reported plainly.
- **The last in-scope `loop next` source is built.** With `lane` and `premise-probes` from #2969, the only remaining source outside this item is GPU collections, which needs a record definition in the watcher file.
- **The new gate proves it.** Removing the open-row filter turns `loop-next-owner-direction` red with the owner-direction sentence; the whole suite and all eight gates are green at the head.