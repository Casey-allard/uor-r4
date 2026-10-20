# #2385 — D30 check-port 9 of 10: the receipts rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:55:08Z
- **Updated:** 2026-10-13T21:57:11Z
- **Closed:** 2026-10-13T21:56:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2385
- **Merged:** 2026-10-13T21:56:58Z
- **Merge commit:** 527ffcfb1e954b91fc9fe2eef3199e84438d3889
- **Base / Head:** main / UOR-Foundation:deepseek/receipts-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 9 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `receipts.parse` | error | all pass | 0 findings |
| `receipts.ledger_present` | warn | all pass | 0 findings |
| `receipts.skill_usage_recorded` | info | — | **not ported** |

Coverage reads **29 of 40 rules** (arena 4, briefing 3, budget 2, harness 3, mistakes 2, multilab 6, prep 5, receipts 2, status 2).

## Two rules over one file — and I checked before assuming they overlapped

| | `receipts.parse` | `budget.receipts_parse` |
|---|---|---|
| not JSON | error | error |
| missing `ts`/`kind` | **one finding: "has no ts/kind"** | **separate: "is missing ts" / "is missing kind"** |
| non-object JSON | *not checked* | **error** |
| unknown `kind` | **error, against the known set** | *not checked* |

**Complementary, not duplicates.** A bad line can produce **two findings from two rules, and that is correct** — they answer different questions. **Porting only one would lose a check; merging them would change what `check` reports.**

**The kind check is an error, not a warning:** a `kind` outside the set means **the ledger holds something nothing reads** — either a typo, so the record is filed under a heading no reader looks at, or a new kind added without a reader.

**Measured against the real ledger: 7 of the 10 known kinds** are in use, so the set covers practice rather than being aspirational.

## Why `skill_usage_recorded` is not ported

**Its body returns an empty list on both branches** — the Python says so in a comment. **It is a rule that cannot fail**, and porting it would add a function whose only observable effect is to be counted. Same call as `budget.no_new_spending_class`: **a rule that cannot fail reads as coverage.**

## `ledger_present` is deliberately narrow

**A directory that exists and holds nothing reads as a working ledger** — that is the case it reports. **A directory that does not exist returns no finding**, because a fresh checkout has none and reporting it would fire on every clone before any work happened.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 295 passed (10 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 29 implemented
```

Record: `docs/labs/port-check-receipts-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 9 of 10 modules — only `prlint` (2 rules) plus `harness.state_formats` and the two GitHub-conversation `status` rules.
- **`KINDS` is a hand-kept copy of Python's set, and nothing in CI compares them.** A kind added to Python and not here would make Rust report **a valid receipt as unknown — a finding that is wrong rather than missing.**
- **Both ledger rules read the committed ledger only**, not the live tier — as Python does.
- **A non-object JSON line is not reported by `receipts.parse`**; it falls into the "has no ts/kind" branch, so the message gives an imprecise reason for the right verdict, while `budget.receipts_parse` names the real problem. Python's behaviour, kept.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

