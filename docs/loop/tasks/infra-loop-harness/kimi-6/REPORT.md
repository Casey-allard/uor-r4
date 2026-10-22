# REPORT: infra-loop-harness, kimi-6 (umbrella gains loop_prcheck.rs)

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

Nothing failed. A one-line scope addition requested by claude on #2028: claude-22 (#2934) makes repair chains legal in `pr-check`, and an existing case in `crates/uor-r4-harness/tests/loop_prcheck.rs` must change under this umbrella ("`claude-1` + `claude-2` is refused" becomes "`claude-1` + `codex-2` is refused"); new tests move to `tests/loop_repair_orders.rs` (already scoped in kimi-5), but the three-line edit cannot move.

## What changed

- `docs/roadmap/items/infra-loop-harness.md`: appended `crates/uor-r4-harness/tests/loop_prcheck.rs` to the umbrella scope. Verified the file appears in no other item's scope, so no new overlap; future `pr-check` behaviour changes get the same home.

Size by class: docs 1 file, one scope line; no code, no tests, no generated files. The ORDER set no estimate (D48).

## Measurements

- `grep -l 'tests/loop_prcheck.rs' docs/roadmap/items/*.md` → only `infra-loop-harness.md`.
- `cargo run -q --locked -p xtask -- roadmap check` → 87 items OK; `render --check` → current.

## Checks run at the head

- `xtask roadmap check` and `xtask roadmap render --check`: pass.
- `uor-harness loop gates --quick`: fmt-claim-wording, fmt-harness, claim-wording GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: see the PR body for the result at the head; expectation is a full PASS (single ORDER first commit, item file inside the umbrella's scope at the base).

## Skills used

- scope-guard: written edge, `git diff --stat` before content. The ORDER and claude's #2028 request were the opener.

## Still open

- claude-22 (#2934) itself: the repair-chain legalization and shared-nine exemption in `pr-check`, under D59 review like every setup PR.
- Lane's standing question to Casey: whether kimi takes loop PR 8 (#2913) next; a pending owner question means wait.

## Plain summary

The architect's next change fixes the pull-request checker so repair chains are legal, and one existing test in the checker's own test file has to change with it. That file belongs to the umbrella item, and this change says so in the roadmap, so the checker will not flag the architect's own edit.

References #2904.
