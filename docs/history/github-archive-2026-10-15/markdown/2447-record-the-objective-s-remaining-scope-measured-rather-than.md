# #2447 — Record the objective's remaining scope, measured rather than estimated (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:37:55Z
- **Updated:** 2026-10-14T05:40:26Z
- **Closed:** 2026-10-14T05:40:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2447
- **Merged:** 2026-10-14T05:40:06Z
- **Merge commit:** b15ba79988984e6757835fded1959d45147800b2
- **Base / Head:** main / UOR-Foundation:deepseek/scope-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The objective's remaining scope, measured rather than estimated.** Documentation only — **no code changed.**

Decision: **KEEP** (tooling).

## Why this is a PR at all

**Because the two metrics this port has been quoting answer different questions, and a reader could reasonably conclude from them that the port is nearly finished.**

The rule inventory says **`53 of 54`** and **`0 of check's own rules still in Python`** — **both about the `check` rule set, not the command surface.** The command surface says **six things are left.** **Reconciling them is worth a record**, because this port already spent a unit correcting exactly this confusion when `cmd_check` ran one rule of forty-one while `parity` reported the set complete.

## The measured scope

**Four `delegate_to_python` sites** in `crates/uor-r4-harness/src/main.rs`:

| line | what reaches it |
|---|---|
| 374 | `Some(other) => …` — **the general fallback** |
| 560 | `mistakes show` \| `capture` |
| 1425 | `multilab sync-live` \| `migrate-ledger` |
| 1563 | `team roster` \| `dispatch` |

**And the fallback's scope**, by differencing Python's argparse top-level choices (14) against `PORTED_COMMANDS` (25):

```
IN PYTHON, NOT PORTED:  arena, budget, panel
```

**So the objective is six items from done**: three unported top-level commands (`arena`, `budget`, `panel`) and three unported subcommand groups.

## What this changes

**Nothing in the code. Everything in what the next unit starts from** — the measured list rather than a re-derivation, **and a reader comparing the rule count to the command count finds the reconciliation instead of inferring one is stale.**

## Checklist

- [x] Documentation only; no code, no capability, no headline
- [x] Record carried in `docs/integration/current-state.md`
- [x] Branch deleted after merge

### Stated rather than hidden

- **`arena`, `budget` and `panel` have Rust rule modules** (`arenarules`, `budgetrules`) **but no ported COMMAND**, so `uor-harness arena` still delegates to Python **while `check` evaluates `arena.*` in Rust.** That is a real split — the rules are ported and the command is not — and it is the most confusing part of the remaining list.
- **I committed this to `main` first and the push was rejected by the remote.** I moved it to a branch and reset `main` to `origin/main`; **`main` is at `c866ceea7` and was never pushed to**, but the near-miss is recorded rather than omitted.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

