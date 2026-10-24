# REPORT: infra-loop-harness, claude-73

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- One PR for all of phase 0 was the first plan. It cannot pass: `loop pr-check`'s scope rule reads item scopes at the merge base, and no item scoped `CHARTER.md` or `docs/rules/changes.md`, so the records PR would fail scope, and `loop queue` refuses a PR whose pr-check fails. This unit lands the scope first; the records PR (claude-74) follows on top of it.

## What changed

The `infra-loop-harness` item's `scope:` gains `CHARTER.md` and `docs/rules/changes.md`, placed after `AGENTS.md`. Nothing else in the item changes. Both paths stay in `docs/harness/owner-paths.json`, so a change to either still needs an `## Owner approval` section in its ORDER.

Size: docs 3 files (the item line, this ORDER and this REPORT); tests, code, generated and other 0. Against the ORDER's estimate (one line plus the task folder): 1.0.

## Measurements

Nothing was measured; this is a record change.

## Checks run at the head

- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check`: `roadmap check: 91 items OK`.
- The cancel probe at `3ac553790`: `git grep -n "^scope:" -- docs/roadmap/items | grep -c -E "CHARTER|docs/rules"` printed `0`.
- `git diff --check origin/main...HEAD`, `uor-harness loop report --check`, `uor-harness loop pr-check --local` and the claim-wording gate: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- The records PR itself (claude-74) lands next on top of this one: the charter amendment, decision rows D65–D71, the AGENTS.md summary and the change-log entry for the owner's 24 Oct direction.

## Plain summary

Each roadmap item lists the files its work may touch, and the harness refuses a pull request that edits a file outside that list. The rule book (`CHARTER.md`) and its change log were on no item's list, so the owner's new direction could not be written into them through the normal path. This change adds both files to the list of the item that owns the loop's rules. They remain files only the owner may approve changes to.
