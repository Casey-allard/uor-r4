# #2383 — D30 check-port 7 of 10: the budget rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:42:26Z
- **Updated:** 2026-10-13T21:44:27Z
- **Closed:** 2026-10-13T21:44:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2383
- **Merged:** 2026-10-13T21:44:18Z
- **Merge commit:** dced9e6738282cb344ac53d8b5a55e1b575309f0
- **Base / Head:** main / UOR-Foundation:deepseek/budget-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 7 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `budget.receipts_parse` | error | all pass | 0 findings |
| `budget.disk_floor` | warn | all pass | 0 findings |
| `budget.no_new_spending_class` | info | — | **not ported** |

Coverage reads **24 of 40 rules** (arena 4, budget 2, harness 3, mistakes 2, multilab 6, prep 5, status 2).

## Why `no_new_spending_class` is not ported

**It has no check function.** Its registration is `info` severity and its entire content is that **the owner caps are printed** so a session cannot silently adopt a new spending class — **the rule is a display.**

Porting it here would mean a function that always returns no findings: **a rule that cannot fail, dressed as a rule.** That is the shape this line keeps flagging — *a guard that stops firing is indistinguishable from a guard that passes* — and it belongs with the `budget` command's rendering, where the caps are actually shown.

## `receipts_parse` protects every other rule's evidence

**A line that will not parse is not a missing datapoint but a corrupt one** — a reader who silently skips it draws conclusions from what remains. So a bad line is an **error**, not a warning.

**Three failures kept apart, because the fixes differ:**

- **not JSON** — repair or drop that line;
- **JSON but not an object** — a bare array is not a receipt;
- **an object missing `ts` or `kind`** — **parseable and meaningless, and the dangerous one**, because it **looks like a record** while carrying none of the fields a reader filters on.

**A null key counts as missing:** `{"ts": null}` is not a timestamp, and testing only for absence would have accepted it. **The line number is in every message and every fix** — *"the ledger has a bad line"* is not actionable; *"line 47 of `kimi-2026-10.jsonl`"* is.

## `disk_floor` gives a stop condition a machine

**Below 30 GB is a stop condition in the owner's rules.** One that lives only in prose is one nobody notices until the disk fills.

**The measurement is real** — `df -Pk`, POSIX-specified output, fourth column. The alternative was `libc::statvfs` behind an FFI bind, and **the harness has no `libc` dependency** — more moving parts than shelling out for a read-only query.

**An unmeasurable disk is a warning, not a pass**: it could be any size, and reporting nothing reads as "fine". Measured during this port: **38.9 GB — inside the band.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 276 passed (9 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 24 implemented
```

Record: `docs/labs/port-check-budget-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 7 of 10 modules.
- **`receipts_parse` reads the committed ledger only**, not the live tier — Python does the same, so a corrupt line in a live shard is seen by neither.
- **`disk_floor` measures the repo's filesystem, not the machine's free space overall.** On a machine where the repo is a small volume it reports that volume — **the port agrees with Python rather than with what a reader might assume "free disk" means.**
- **The `df -Pk` column index is positional.** `-P` is specified to prevent format drift, but the parse has no fallback if a system ignores it.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

