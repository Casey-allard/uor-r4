# ORDER: infra-loop-harness, deepseek-27 (repair: header-named PLAN columns and the ordering test)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 1. Base: `12ac5b7ce`.

## Goal

Repair the CHANGES record on #2974 at `12ac5b7ce` (Claude, fresh context): the PLAN table parser reads "open" and the direction by position instead of by header, the premise source regresses from `main`, and nothing pins "owner direction before repairs".

## Cancel probe (run first)

Class **instrument** (loop tooling, host only). At `12ac5b7ce`: `plan_open_row` matches `cells.iter().any(|c| c.eq_ignore_ascii_case("open"))` and the owner branch takes `cells[0]`, so `| D3 x | done | ... | open |` is offered; `premise_probe` takes the first backticked cell in any column and returns `None` on the first malformed open row; and `tests/loop_next_sources.rs` uses the empty GitHub fake, so moving the owner-direction block after repairs leaves all tests green. All three reproduce. The task stands.

## Design intent (the PM question in the review)

The owner direction is deliberately global: the loop page orders it first for every lab, ahead of repairs, until the row is closed. Per-lab ownership (a lab column, a check against open PRs) would change PLAN's schema and the loop's stop rules, so it is a PM/owner decision card, not something this repair invents. The ORDER says so explicitly.

## Files in scope

`crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/tests/loop_next*.rs`, `docs/harness/gates.json`, `docs/harness/README.md`, `docs/loop/README.md`, and the task folders `deepseek-26`/`deepseek-27`.

## Done when

- The parser reads PLAN.md as a header-named table: Status/Probe/Direction columns are found by header text; a note cell saying `open` is not a status; malformed open rows are skipped and named, never hidden; a PLAN.md without the table is named.
- The premise source keeps `main`'s semantics: the Probe column is used, not any backticked cell; a valid open row after a malformed one is still offered.
- A test with a failing own PR pins owner-direction **before** repairs, and moving the block below repairs turns it red.
- The `loop-next-owner-direction` gate fault matches the new parser and is CAUGHT with "the open owner direction must be offered before repairs"; `report --check`, `pr-check --local`, `loop gates --full` (nine) and `--break all` (nine CAUGHT) pass; receipts at the final head.
- `REPORT.md` numbers match the head (28 `loop_next` tests, nine gates) and its negatives name the positional parser and the unpinned ordering.

## Stop and ask if

- The PM wants per-lab owner directions (PLAN schema/lab column) now: that is a card to Casey, not a repair choice.