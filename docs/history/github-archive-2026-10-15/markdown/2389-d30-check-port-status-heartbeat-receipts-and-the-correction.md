# #2389 — D30 check-port: status.heartbeat_receipts, and the correction that made it small (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T22:26:30Z
- **Updated:** 2026-10-13T22:29:03Z
- **Closed:** 2026-10-13T22:28:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2389
- **Merged:** 2026-10-13T22:28:32Z
- **Merge commit:** c5cccad22434e42d317d87eacdf6603247fcf837
- **Base / Head:** main / UOR-Foundation:deepseek/heartbeat-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port: `status.heartbeat_receipts`.** Tooling only.

Decision: **KEEP** (tooling).

## Agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `status.heartbeat_receipts` | warn | all pass | 0 findings |

Coverage reads **32 of 40 rules.**

## This port is the test of my own correction

I told the owner and recorded in `current-state.md` that this rule and `registry_vs_github` are **"the two rules whose subject is the GitHub conversation"**, needing `gh` pagination and comment parsing — **and I used that estimate to defer this rule three times.**

**The correction said it reads the local ledger.** This port is what the corrected estimate predicted: **one receipt read, one age comparison, one file.** No GitHub call appears anywhere in it.

**The lesson is not that I was wrong — it is that the wrong estimate had a cost measured in deferrals.** A rule I could have finished in one unit sat unported for three **because I described it instead of reading it.**

## What the rule asks

A milestone marked **in-progress** should show a heartbeat receipt within **`STALE_HOURS = 24`**.

**The window is a day, not the five-minute presence window**, and that is deliberate: **presence is live-tier and ad-hoc, while a heartbeat receipt is the durable record.** A milestone is not abandoned because nobody wrote a receipt this afternoon.

**Only `in-progress` milestones.** A finished one owes no check-in — **and a check that nags about completed work gets ignored when it is right.**

## Three details the port had to get right

**A receipt may name its milestone three ways** — short key, display id, or **issue number**. **Requiring one spelling would report a milestone as silent while its receipts sat in the ledger.** Asserted for all three.

**A heartbeat is not any receipt.** A `skill` receipt naming the milestone does **not** satisfy it. Asserted.

**The boundary is asserted rather than assumed** — exactly 24h satisfies a `<= 24` window, 25h is reported — **because a boundary written from memory produced two of my earlier mistakes in this port.**

`now` is a parameter, so **the same question can be asked about a fixed instant**, which the Python cannot do.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 311 passed (7 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
python3 scripts/check_no_softmax.py             -> OK, no softmax added
parity.py                                       -> exit 0, rule inventory: 32 implemented
```

Record: `docs/labs/port-check-heartbeat-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** Eight rules remain.
- **This reads the whole ledger; the Python reads a bounded current-plus-previous month.** On the current ledger the counts agree (0 and 0), but **on a ledger large enough for that to matter the two could differ** — a deliberate simplification rather than parity.
- **A receipt with an unparseable timestamp is ignored rather than reported**, so a milestone whose only heartbeat is corrupt reads as having no heartbeat — **the right verdict from the wrong evidence**, and neither implementation says which.
- **`registry_vs_github` is still unported, and its GitHub dependency is unverified** — I am not repeating the mistake of describing it before reading it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

