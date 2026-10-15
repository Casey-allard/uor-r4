# #2353 — D30 port 1: team is Rust, and the front door stops saying "not ported" (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:39:53Z
- **Updated:** 2026-10-13T17:43:32Z
- **Closed:** 2026-10-13T17:43:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2353
- **Merged:** 2026-10-13T17:43:13Z
- **Merge commit:** a6e73c63bf54c4dde59ae2292762d0e0356d03a4
- **Base / Head:** main / UOR-Foundation:deepseek/port-team
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 1 of the order posted on #2028.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`crates/uor-r4-harness/src/team.rs` — new, with **twenty tests**. `uor-harness team claim|release|claims` is now Rust and **`team` is out of the delegated set**, so the *"not ported to Rust yet"* notice no longer prints for it — verified by running it.

`roster` and `dispatch` still delegate **by name**, so each remaining action says so individually. **The progress measure shrinks action by action, not command by command** — the rule set in the D30 record, applied here for the first time.

## Rules the port had to keep

- **One arbiter per branch, and it is the committed one** (live → committed, weakest first). An earlier Python version merged newest-wins and that was wrong: `claim` stamps one entry into two tiers, the copies tie, then diverge under a hand edit — and **newest cannot choose between two claims that are both current**.
- **Re-claiming your own branch succeeds and refreshes** — it is how a claim is renewed after a rebase. Only **another** lab is refused.
- **Release republishes the live mirror.** A mirror only ever added to **resurrects released claims.** Tested on both tiers.
- **A refused claim writes nothing.**
- **Empty branch or lab is a usage error (exit 2), not a refusal (exit 1).**
- **A missing or corrupt store reads as empty** — a lab must be able to claim on a machine whose *advisory* tier is absent, or the advisory tier becomes load-bearing.

## Three bugs the port found, all in the new code

**The first two would have shipped as "it compiles and the happy path worked":**

1. **`%:z` is a GNU extension.** BSD/macOS `date` printed the literal `:z`, producing `2026-10-13T13:37:12:z` — a timestamp every reader of the committed store would have to cope with.
2. **`serde_json` preserves insertion order; the Python store wrote `sort_keys=True`.** An unsorted writer would have reordered every existing claims file on first touch, turning a one-line claim into a **whole-file diff in review**.
3. **`--repo` beat the subcommand word, twice.** First `args[0]` made `--repo` the action; then "the first word without a dash" made the repo **path** the action, because a path is a bare word too.

**The fixture capture is what caught the first two** — they are only visible when the output is compared against what Python actually produced.

## Trap 1 from the D30 record, applied

`tests/fixtures/team/behaviour.json` — the golden behaviour **captured from the Python harness before it was replaced**: refusal texts, the exit-code distinction, and the non-obvious rules (same-lab reclaim, release-without-lab, mirror-on-release). **Captured while Python still existed**, which was the point.

**This is a regression lock, not parity.** It cannot catch a bug in the interpretation of the fixture itself, and after the port there is no implementation left to disagree with it.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 152 passed (20 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
Rust vs captured Python output                  -> identical (sorted keys, -04:00 offset, same refusal)
delegation notice for `team`                    -> gone
```

Record: `docs/labs/port-team-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`roster` and `dispatch` still run Python.** Two of five actions moved, not five.
- **The Python `team` is untouched and still reachable** through `cli.py`. It is **not retired** until those move and the Python tests covering them port — deleting it now would leave those tests testing nothing.
- **Claims remain deliberately not a lock.** Two labs claiming one branch in different worktrees both succeed locally; the collision surfaces as a `claims.json` conflict. The port preserves that accepted cost rather than quietly changing coordination semantics.
- **`now_iso` shells out to `date`.** A machine without it gets the epoch — a wrong timestamp rather than a failure. Not measured on a system without `date`.
- **I did not test concurrent claims from two processes.** The store is written whole, so a race would lose one claim. The Python version had the same shape and I have not changed it, but **I have not proven it either.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

