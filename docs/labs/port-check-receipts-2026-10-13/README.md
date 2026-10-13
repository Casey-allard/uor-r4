# D30 check-port 9 of 10: the `receipts` rules

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 9 of 10

## The rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `receipts.parse` | error | all pass | 0 findings |
| `receipts.ledger_present` | warn | all pass | 0 findings |
| `receipts.skill_usage_recorded` | info | — | **not ported, see below** |

Coverage reads **29 of 40 rules** (arena 4, briefing 3, budget 2, harness 3, mistakes 2, multilab 6,
prep 5, receipts 2, status 2).

## Two rules over one file, and why they are not duplicates

`receipts.parse` and `budget.receipts_parse` both read the ledger. **I checked before assuming they
overlapped**, and they are complementary:

| | `receipts.parse` | `budget.receipts_parse` |
|---|---|---|
| not JSON | error | error |
| missing `ts`/`kind` | **"has no ts/kind"** — one finding | **"is missing ts" / "is missing kind"** — separate |
| non-object JSON | *not checked* | **error** |
| unknown `kind` | **error, against the known set** | *not checked* |

So **a bad line can produce two findings from two rules, and that is correct** — the two rules are
answering different questions. **Porting only one would lose a check; merging them would change what
`check` reports.**

**The kind check is the one that distinguishes them**, and it is an error rather than a warning for a
reason: **a `kind` outside the set means the ledger holds something nothing reads.** Either a typo —
so the record is filed under a heading no reader looks at — or a new kind added without a reader,
which is the same problem with better intentions.

**Measured against the real ledger:** it carries **7 of the 10 known kinds**
(`claim`, `heartbeat`, `prep`, `receipt`, `release`, `skill`, `team`), so the set covers practice
rather than being aspirational.

## Why `skill_usage_recorded` is not ported

**Its body returns an empty list on both branches** — the Python says so in a comment: *"informational
only: the engine prints `ok` for an empty list."* **It is a rule that cannot fail.**

Porting it would add a function whose only observable effect is to be counted. That is the same call I
made for `budget.no_new_spending_class`, and for the same reason: **a rule that cannot fail reads as
coverage.** If the intent is to report whether skills have been used, that belongs in the `receipts`
command's output, where a reader sees it.

## `ledger_present` is deliberately narrow

**A ledger directory that exists and holds nothing reads as a working ledger**, which is the case it
reports. **A directory that does not exist returns no finding** — a fresh checkout has none, and
reporting it would fire on every clone before any work happened.

## Verification

| check | result |
|---|---|
| Rust vs Python, both rules | **identical — all pass in both** |
| real ledger kinds | 7 of 10 known kinds present |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **295 passed** (10 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **29 implemented** |

## Limitations, stated

- **`check` still delegates.** 9 of 10 modules; only `prlint` (2 rules) remains, plus `harness.state_formats`
  and the two GitHub-conversation `status` rules.
- **`KINDS` is a hand-kept copy of Python's set.** The agreement probe compares the two counts, but
  **nothing in CI does** — a kind added to Python and not here would make Rust report a valid receipt
  as unknown, which is **a finding that is wrong rather than missing.**
- **The two ledger rules read the committed ledger only**, not the live tier. Python does the same; a
  corrupt line in a live shard is seen by neither.
- **A non-object JSON line is not reported by `receipts.parse`.** It falls into the "has no ts/kind"
  branch, so the message says the keys are missing rather than that the line is not an object —
  **the right verdict with an imprecise reason.** `budget.receipts_parse` names the real problem. That
  split is Python's behaviour, kept.
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts.
