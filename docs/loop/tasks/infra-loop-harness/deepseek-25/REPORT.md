# REPORT: infra-loop-harness, deepseek-25 (loop next: lane and premise-probe sources)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 1. Base: `ce7782a74`. This is a loop setup PR: its exact-head review comes from Claude's fresh-context reviewers (D59).

## What did not work, first

- **Only two of the three sources could be built here.** `gpu-collections` needs a *collection record* for a GPU run; the watcher (loop PR 10) notes renewals but writes no collection record, and the watcher source is outside this item's scope. I left it a named not-built source with the precise reason rather than inventing a record some other unit must later match.
- **The premise-table shape is a contract this PR defines.** `PLAN.md` does not exist yet (the PM writes it), so there is no existing table to read. I implemented the smallest useful contract and documented it in the loop page: a `## Premises` section with a Markdown table whose rows carry a premise, a probe cell with the command in backticks, a status, and a cancel line; the first row whose status is `open` is offered. If the PM wants another shape, this is the one file to adjust.
- **An existing test had to change because the specified behaviour changed.** `tests/loop_next.rs::the_not_built_sources_are_printed_with_their_loop_prs` pinned the three old strings; the source statuses are now different by design (two are live readers), so it asserts the new reasons under the same name. That is a specified-behaviour update, not a weakened test.
- **Lane data does not exist yet.** No item has `lane:`/`rank:` and the owner-only registry has no `lane`, so on today's `main` the lane source reports "no lane is assigned" rather than picking work. The code path and its tests are ready for the owner/PM backfill; the source deliberately does not guess a lane.
- **`loop next` now reads `PLAN.md` and `labs.json` from `origin/main`** (like the item listing), so a dirty working tree cannot change the decision; both reads are optional and their absence is reported.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopclaim.rs` | code | +22 | `ItemFields` parses optional `lane:` and `rank:` keys |
| `crates/uor-r4-harness/src/loopnext.rs` | code | +~190 | `origin_file`, `premise_probe`, `lab_lane`, `ready_items` helpers; the three not-built placeholders become a precise GPU-collection gap, the PLAN premise-probe source and the lane source, with `any` reusing the extracted ready list |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | test | +~190 | lane ordering (rank then id) before `any`, the open premise probe as target, and the missing-input lines |
| `crates/uor-r4-harness/tests/loop_next.rs` | test | +3/−3 | the not-built test asserts the new reasons |
| `docs/harness/gates.json` | generated | +30 | the `loop-next-sources` gate and its planted fault |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +8 | the sources' contracts and what is missing when the data is absent |
| `docs/loop/tasks/infra-loop-harness/deepseek-25/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 440 changed lines against the ORDER's estimate of about 400 (1.1).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_next_sources`:** 3 passed (lane rank ordering, PLAN premise selection, missing-input lines).
- **`cargo test -p uor-r4-harness --test loop_next`:** 27 passed (the updated not-built test included).
- **Whole harness suite:** every target green; the local unit suite is 662 passed / 0 failed / 1 ignored at this head (before this unit's test additions), and the new targets above pass.
- **Gates:** seven GREEN, including the new `loop-next-sources`; `--break all` CAUGHT with the lane-filter fault red at "the top of the lab lane must be offered before any".
- **No GPU work, no owner path, no spend:** reads are `git show origin/main:PLAN.md` and `...labs.json`; nothing writes.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN (`fmt-harness`).
- `UOR_LAB=deepseek uor-harness loop gates --full`: seven GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: seven CAUGHT.
- `cargo test -p uor-r4-harness`: all targets green.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the one source that could not be built honestly (GPU collections) is named, not faked; the one format with no prior art (PLAN premises) is a documented contract rather than hidden guesswork.
- **scope-guard**: only the item's scoped files and the shared registration files.

## Still open

- **`gpu-collections`** needs the watcher to write a collection record; that file (`loopdrive.rs`) is outside this item's scope, so it is a follow-up item/scope decision.
- **Lane activation** waits on the owner adding `lane` to `docs/harness/labs.json` and the PM backfilling item `lane:`/`rank:` keys; until then the source reports the missing input.
- **PLAN.md** is the PM's file; this PR defines only the reader's table contract.

## Plain summary

- **Two of `loop next`'s three missing sources now work.** The lab-lane source reads the owner's lane from the registry and the PM's item ranks and offers the best lane item before falling back to `any`; the premise-probe source reads PLAN's premise table and offers the first open probe command. Both say exactly what is missing when their inputs are absent, and neither guesses.
- **The third source is honestly still not built.** A GPU collection needs a record the watcher does not write; defining that record changes a file outside this item, so it is named as a follow-up instead of being faked.
- **The new gate proves the lane source.** Its planted fault removes the lane filter and turns the gate red with the lane-ordering sentence; the whole harness suite and the seven gates are green at the head.