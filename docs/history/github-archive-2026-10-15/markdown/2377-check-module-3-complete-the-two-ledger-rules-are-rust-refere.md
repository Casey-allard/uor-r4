# #2377 — check module 3 complete: the two ledger rules are Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:53:33Z
- **Updated:** 2026-10-13T20:56:14Z
- **Closed:** 2026-10-13T20:55:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2377
- **Merged:** 2026-10-13T20:55:55Z
- **Merge commit:** 262b9b92519ad6ef51112816f5ffc90af756af65
- **Base / Head:** main / UOR-Foundation:deepseek/ledger-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`check` module 3 complete.** Tooling only.

Decision: **KEEP** (tooling).

## All six `multilab` rules are Rust — module 3 of 10 done

| rule | severity | Python | Rust |
|---|---|---|---|
| `multilab.no_shared_ledger` | warn | all pass | 0 findings |
| `multilab.ledger_lab_recorded` | warn | **1 warn** | **1 warn** |

Coverage reads **14 of 40 rules** (harness 3, multilab 6, prep 5).

## The off-by-one that made the rule match nothing

`legacy_ledgers` filters unsharded `<YYYY-MM>.jsonl`. I wrote its length guard as **`len == 12`**. **`"2026-10.jsonl"` is 13 bytes.** Every other predicate was right — digits, dash, digits, `.jsonl` suffix — and **the wrong length silently matched nothing**, so the rule would have reported no legacy ledgers **while a leftover unsharded ledger sat in the tree**.

**Second time in this port that a hand-written constant of mine was wrong rather than the code** (the first was the day count, 20744 vs 20739). Both the same mistake: **I wrote a value from memory and it wore a check's clothes.** The length is now **derived from the shape** and asserted in a test.

**Finding it required printing each predicate part rather than re-reading the condition.** I read that condition three times and it looked correct each time; the probe showed `len=13` on its first line.

## Why sharding matters, because the rule depends on it

**Sharding exists so concurrent PRs never conflict on one file.** A leftover unsharded ledger is **not clutter — it is a file two labs will both append to: the conflict the sharding removed, still writable.** That is why the rule warns rather than ignoring old files.

**`ledger_lab_recorded` is the one that fired.** A receipt with no lab is worse than an absent one: **absence is visible, an unattributed record is not** — it looks like evidence while proving nothing about who did the work.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 243 passed (4 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 14 implemented
```

Record: `docs/labs/port-check-ledger-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 3 of 10 modules; seven remain.
- **The date pattern is a shape check, not a calendar check** — `9999-99.jsonl` matches. Python behaves the same, the port agrees, and an impossible month is a different problem.
- **`ledger_lab_recorded` reads every `.jsonl`**, including a shard for a lab that no longer exists — as Python does.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares per-rule finding counts.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

