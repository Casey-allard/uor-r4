# REPORT: infra-loop-harness, deepseek-27 (repair: header-named PLAN columns and the ordering test)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 1. Base: `12ac5b7ce`. Loop setup PR; exact-head review at the head.

## What did not work, first

- **Positional parsing was wrong, and the review proved it.** The first version matched any cell equal to `open` and took `cells[0]` for the direction, so `| D3 x | done | 2026-10-22 | open |` was offered as a live owner direction. The parser now reads the header row and finds the Status, Probe and Direction columns by name; a note cell cannot be a status.
- **The premise source regressed from `main`.** It took the first backticked cell in any column (dropping `main`'s status-column exclusion) and returned `None` at the first malformed open row, hiding later valid rows and lying "no open probe". It now uses the Probe column, skips and names malformed open rows, and still offers a later valid row.
- **Nothing pinned "before repairs".** The ordering test used the empty GitHub fake, so moving the block below the repair source kept every test green. The repair adds a test with a failing own PR; with the block moved below repairs it fails on "the open owner direction must be offered before repairs".
- **The missing-table case claimed "no open" rather than naming the table**, and the dead header-skip compared cells against the heading text. Both are replaced by one header-named table reader that names a missing PLAN.md, a missing `## <heading>` table, a missing column header, and malformed open rows.
- **Owner directions are global by design and that is a PM question, not a repair.** The loop page orders owner direction first for every lab; per-lab ownership would change PLAN's schema and stop rules. The ORDER now records the intent and leaves the card to Kimi/Casey.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | code | +~230/−90 | `PlanTable` (header row + rows), header-named `column`, `open_rows`; `premise_probe` and `owner_direction` rewritten on it, with malformed-row notes and named missing tables; the owner branch passes the skips into the reason |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | test | +~150 | a scripted-fake helper; the before-repairs ordering test with a failing own PR; a note-cell-is-not-a-status test; malformed owner/premise row tests; a missing-table test |
| `docs/harness/gates.json` | generated | 2 lines | the gate fault now removes the owner-direction row check (`if text.is_empty()`) |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +4/−2 | the header-named contract and malformed-row behaviour |
| `docs/loop/tasks/infra-loop-harness/deepseek-26/REPORT.md` | docs | small | stale counts/gates fixed, the positional-parser negative added |
| `docs/loop/tasks/infra-loop-harness/deepseek-27/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 390 changed lines (ORDER estimate about 370, 1.05).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_next_sources`:** 9 passed (was 4): ordering with a failing own PR, note-cell status, malformed owner row skipped and named, malformed premise row named, missing table named, plus the original four.
- **`cargo test -p uor-r4-harness --test loop_next`:** 28 passed (claude-31's WAIT tests included; the updated not-built line still pinned).
- **Mutation check:** moving the owner-direction block below the repairs source now makes the ordering test red; removing the row check makes the gate red at "the open owner direction must be offered before repairs".
- **No GPU work, no owner path, no spend:** reads are `git show origin/main:PLAN.md` and the registry; nothing writes.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: nine GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: nine CAUGHT.
- `cargo test -p uor-r4-harness --test loop_next --test loop_next_sources`: all green.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the parser was rebuilt around the data's own headers rather than patched position by position, and the regression is now pinned by a test that fails if the ordering moves.
- **scope-guard**: only the item's scoped files, shared registration files and the two task folders.

## Still open

- **Per-lab owner directions** (the PM question): a PLAN schema/lab-column change is a PM/owner card; today the first open direction is global, as the loop page orders.
- **`gpu-collections`** still needs the watcher to write a collection record (`loopdrive.rs`, outside this item).
- **Lane/PLAN activation** waits on the owner's lane field, the PM's item ranks and PLAN.md itself.

## Plain summary

- **The reviewer was right and the parser is rebuilt.** PLAN tables are read by header name, so a note cell cannot be mistaken for an open status; malformed open rows are skipped and named, and a missing table is named. The premise source keeps `main`'s column semantics instead of regressing.
- **The ordering is now pinned by a failing-own-PR test.** Moving owner-direction below repairs makes the test red, so the gate's "before repairs" claim is proven, not asserted.
- **Numbers are honest again:** nine source tests, 28 `loop_next` tests, nine gates GREEN and nine CAUGHT at the head.