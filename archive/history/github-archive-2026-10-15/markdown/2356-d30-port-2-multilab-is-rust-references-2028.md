# #2356 — D30 port 2: multilab is Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:59:19Z
- **Updated:** 2026-10-13T18:01:12Z
- **Closed:** 2026-10-13T18:01:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2356
- **Merged:** 2026-10-13T18:01:11Z
- **Merge commit:** f39a907dfb48e6790328fdd9a606536af0927509
- **Base / Head:** main / UOR-Foundation:deepseek/port-multilab
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 2.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`crates/uor-r4-harness/src/multilab.rs` — new, **11 tests**. `uor-harness multilab presence|claims|touch` is Rust and **out of the delegated set**. `sync-live` and `migrate-ledger` still delegate **by name**, so the progress measure shrinks action by action.

## The contract that mattered most

**`touch` exists for hooks and must not erase the task.** It refreshes the timestamp and **preserves `doing`, `milestone` and `next`**. Blanking them would show a working lab as having nothing to do — **worse than showing it stale, because stale is at least true.** Tested directly.

## Reuse rather than reimplementation

- **Staleness delegates to `PresenceRow::is_stale`/`age_minutes`**, which already answer "cannot tell" as stale. An unparseable timestamp is **not evidence of freshness** — `0m ago` would make a corrupt file look like the most current lab on the board. Tested with a corrupt file.
- **`claims` reads through the `team` port**, not a second reader of `claims.json`. **Two readers of one file will eventually disagree** — the defect class this whole line keeps producing.
- **State is derived at read time and never stored.**

## One bug found — the third instance of the same class

**`--lab` beat the subcommand word.** Global flags consumed by the top-level parser are pushed into `rest` for the subcommand's benefit, so they never appeared in the subcommand value list — and their **values** became candidate actions. `--repo <dir>` did this in the `team` port; `--lab <name>` did it here, producing `unknown action "deepseek"`.

Both are now named in **one `GLOBAL_VALUE_FLAGS` list**, with a comment that a new global flag must be added there. **The fix is a list rather than a special case, which is the only version that stops the fourth.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 163 passed (11 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
multilab touch / presence vs captured Python    -> same shape and fields
delegation notice for the three ported actions  -> gone
```

Record: `docs/labs/port-multilab-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`sync-live` and `migrate-ledger` still run Python.** Three of five actions moved.
- **The timestamp differs from Python's.** Python wrote ISO with the **local offset** (`2026-10-13T13:54:41-04:00`); `touch` here writes **UTC `Z`**. Both parse and the board orders correctly, but a diff across an old and new presence file shows the format change.
- **`GLOBAL_VALUE_FLAGS` is a list a human must maintain.** A new global flag added to the parser and not to this list reproduces the bug a fourth time, and **nothing fails to warn**.
- **The board is not compared byte-for-byte against Python's output** — columns by eye, JSON by field. The fixture records the shape, not the spacing.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

