# ORDER: infra-loop-harness, claude-97 (loop guard: an item's own notebook page is always writable, #3189)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (harness maintenance). Base: `b19b15f00`.

## Goal

Fix #3189. `docs/loop/README.md` ("Scope, without a line cap") lists "new rows on its notebook page" as always allowed, and `loop pr-check` accepts a notebook edit when the item names the page and the edit only appends (`loopprcheck.rs`, the scope rule). The PreToolUse guard's always-allowed list (`loopguard.rs`, `check_scope` path) covers only the ORDER folder, the item file, `docs/harness/mistakes/`, FILEMAP and the shared files. So an item whose `scope:` omits its `notebook:` page cannot add its own row. claude-95 (#3186) had to leave its row in its REPORT, and claude-94 needed a separate scope unit (claude-96, #3191).

The fix: the guard also allows the active item's `notebook:` page, read from the item on `origin/main`, as the scope already is. A `notebook:` line a branch writes itself grants nothing. `loop pr-check` still enforces append-only at PR time.

**Why the infra-loop-harness item.** Its scope holds `crates/uor-r4-harness/**`. `loop claim` refuses it while Codex's idle #3169 holds an order on it, as it did for claude-96, so this ORDER is written by hand.

## Cancel probe (run first)

Class: instrument (the guard). At `b19b15f00`, all three must hold, or the unit is cancelled:
- `loopguard.rs`'s always-allowed list has no notebook term;
- `docs/loop/README.md` lists notebook rows as always allowed;
- no open PR changes `crates/uor-r4-harness/src/loopguard.rs` or `crates/uor-r4-harness/tests/loop_guard.rs`.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- this task folder

## Done when

- The guard allows a write to the active item's `notebook:` page as it is on `origin/main`, refuses another notebook page outside the scope, and ignores a `notebook:` line written only on the branch.
- A test pins all three, and it fails with the fix set aside.
- The harness suite passes with `--no-fail-fast`, apart from failures that also fail without this change; those are named in the REPORT.
- `loop report --check` and `loop pr-check --local` pass.
- A fresh-context review counts at the exact head (D66).
- The REPORT says `Closes #3189` and `References #2904`.

## Stop and ask if

- The fix would let a branch widen what it may write by editing its own item.
- The fix would change any refusal other than the notebook page.
