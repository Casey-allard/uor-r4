# `check` module 3 complete: the `multilab` rules are all Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028

## The two ledger rules

| rule | severity | Python | Rust |
|---|---|---|---|
| `multilab.no_shared_ledger` | warn | all pass | 0 findings |
| `multilab.ledger_lab_recorded` | warn | **1 warn** | **1 warn** |

**Module 3 of 10 is now complete — all six `multilab` rules are Rust.** Coverage reads
**14 of 40 rules (harness 3, multilab 6, prep 5).**

## The off-by-one that made the rule match nothing

`legacy_ledgers` filters for unsharded `<YYYY-MM>.jsonl`, and I wrote its length guard as **`len == 12`**.
**`"2026-10.jsonl"` is 13 bytes.** Every other predicate was right — digits, dash, digits, `.jsonl`
suffix — and the wrong length **silently matched nothing**, so the rule would have reported no legacy
ledgers while a leftover unsharded ledger sat in the tree.

**This is the second time in this port that a hand-written constant of mine was wrong rather than the
code** (the first was the day count, 20744 vs 20739). Both were the same mistake: **I wrote a value from
memory and it wore a check's clothes.** The length is now **derived from the shape**
(`4 + 1 + 2 + ".jsonl".len()`) instead of restated, and the test asserts the length directly so it
cannot regress.

**Finding it required printing each predicate part rather than re-reading the condition.** I read that
condition three times and it looked correct each time; the probe showed `len=13` on the first line.

## Why sharding matters, restated because the rule depends on it

**Sharding exists so concurrent PRs never conflict on one file.** A leftover unsharded ledger is
therefore not clutter: it is a file **two labs will both append to** — the conflict the sharding removed,
still writable. That is why the rule warns rather than ignoring old files.

**`ledger_lab_recorded` is the one that fired.** A receipt with no lab is worse than an absent one:
**absence is visible, an unattributed record is not** — it looks like evidence while proving nothing
about who did the work, which is what the ledger is for.

## Verification

| check | result |
|---|---|
| Rust vs Python, both rules | **0 = all pass, 1 = 1 warn** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **243 passed** (4 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **14 implemented** |

## Limitations, stated

- **`check` still delegates.** 3 of 10 modules; seven remain (`harness` 3 rules, `mistakes` 4, `status` 4,
  `arena` 4, `briefing` 3, `receipts` 3, `budget` 3, `prlint` 2 — minus the three already done).
- **The date pattern is a shape check, not a calendar check.** `9999-99.jsonl` matches. That is Python's
  behaviour too, the port agrees with it, and a file with an impossible month is a different problem.
- **`ledger_lab_recorded` reads every `.jsonl` in the ledger directory**, including a shard for a lab
  that no longer exists. Python does the same.
- **The agreement measurement is a hand-run probe**, read and removed. **Nothing in CI compares per-rule
  finding counts.**
