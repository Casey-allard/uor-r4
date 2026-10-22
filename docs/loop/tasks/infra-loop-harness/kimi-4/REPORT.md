# REPORT: infra-loop-harness, kimi-4 (scope deltas for the shared-nine decision)

Lab: kimi. Order: folded into [kimi-5/ORDER.md](../kimi-5/ORDER.md) as history; this folder's original ORDER.md was removed so the PR adds exactly one ORDER, which `loop review --check` and `loop pr-check` both require.

## What did not work, first

- claude's kimi-4 list on #2028 said "fold these into #2929 before you merge", but #2929 had already merged (`940fc1730`) when the note landed — the message crossed the merge queue. This follow-up PR carries the deltas instead; nothing was re-opened or rewritten.
- The umbrella's scope listed `docs/roadmap/README.md`, which joined the shared list at 00:05Z (Casey: the shared list is now nine, and no item scope lists them). Dropped; the PM render keeps writing it from the shared-file rule, not from a scope.
- `docs/harness/README.md` had already been dropped from m2-fresh-heldout-panel in kimi-3, so no other item needed a shared-nine removal; verified by grep.

## What changed

- pr11: added `crates/uor-r4-harness/src/looparrive.rs` and `crates/uor-r4-harness/tests/loop_arrive*.rs` (raising arrive's warnings to errors changes them — DeepSeek's #2930 needs them).
- pr12: added `rules/*-hook.sh` and `crates/uor-r4-harness/tests/git_hooks*.rs`.
- infra-loop-harness (umbrella): added the finished loop items' modules (`src/loopreview.rs`, `src/loopnext.rs`, `src/loopreport.rs`, `src/loopclose.rs`, `src/loopclaim.rs` and the matching `tests/loop_*` stems including `loop_repair_orders*.rs` and, per the kimi-5 repair, `loop_shared_files*.rs`) so cross-cutting fixes like claude-22 stay in scope, and dropped `docs/roadmap/README.md`.

Size by class: docs 3 files, +3/-1 scope lines; no code, no tests, no generated files. The ORDER set no estimate (D48).

## Measurements

- `grep '^scope:' docs/roadmap/items/*.md` for the nine shared files (`src/lib.rs`, `src/main.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `src/loopprcheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`): NONE.
- `cargo run -q --locked -p xtask -- roadmap check` → 87 items OK (ready 50, done 7 after loop PR 7c merged). `render --check` → current.

## Checks run at the head

- `xtask roadmap check` and `xtask roadmap render --check`: pass.
- `uor-harness loop gates --quick`: fmt-claim-wording, fmt-harness, claim-wording GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local` at `995286d33`: verdict PASS, every rule green (order, scope, commit-order, report, owner-paths, rule-sources) — the first clean pass, now that the merge base carries scopes. Re-run at the kimi-5 head is recorded in kimi-5's REPORT.

## Skills used

- scope-guard: written edge, `git diff --stat` before content. The ORDER and claude's #2028 list were the opener; the deltas were applied as directed, with the merged-#2929 timing noted above.

## Still open

- claude-22 builds the nine-file exemption into `pr-check`'s scope rule and the item validator; until it lands, `pr-check` may still flag small additive edits to shared files in other labs' PRs — expected per Casey's recorded decision.
- The one-ORDER/`commit-order` repair-chain gap from kimi-3 remains the architect's to answer in the gates.
- Lane's standing question to Casey: whether kimi takes loop PR 8 (#2913) next; a pending owner question means wait.

## Plain summary

The owner decided that nine files every pull request touches — the harness's registration points and the two documentation READMEs — belong to a shared list that no single work item claims. This change finishes applying that to the roadmap's file lists: two loop items gain the files their own work needs, the umbrella item gains the finished loop modules for cross-cutting fixes, and the rendered roadmap page leaves every item's list.

References #2904.
