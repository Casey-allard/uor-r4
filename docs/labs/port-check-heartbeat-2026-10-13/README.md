# D30 check-port: `status.heartbeat_receipts` — and the correction that made it small

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028

## Agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `status.heartbeat_receipts` | warn | all pass | 0 findings |

Coverage reads **32 of 40 rules.**

## This port is the test of my own correction

I told the owner and recorded in `current-state.md` that this rule and `registry_vs_github` are
**"the two rules whose subject is the GitHub conversation"**, needing `gh` pagination and comment
parsing — and I used that estimate to **defer this rule three times**.

**The correction said it reads the local ledger instead.** This port is what the corrected estimate
predicted: **one receipt read, one age comparison, one file.** No GitHub call appears anywhere in it.

**The lesson is not that I was wrong — it is that the wrong estimate had a cost measured in
deferrals.** A rule I could have finished in one unit sat unported for three because I described it
instead of reading it.

## What the rule asks

A milestone marked **in-progress** should show a heartbeat receipt within **`STALE_HOURS = 24`**.

**The window is a day, not the harness's five-minute presence window**, and that is deliberate:
**presence is live-tier and ad-hoc, while a heartbeat receipt is the durable record.** A milestone is
not abandoned because nobody wrote a receipt this afternoon.

**Only `in-progress` milestones.** A finished milestone owes no check-in, so reporting it would nag
about completed work — **and a check that nags about completed work is one that gets ignored when it
is right.**

## Three details the port had to get right

**A receipt may name its milestone in three forms** — the short key (`m1`), the display id (`M1`), or
the **issue number**. All three are the same milestone, and **requiring one spelling would report a
milestone as silent while its receipts sat in the ledger.** Asserted for all three.

**A heartbeat is not any receipt.** A `skill` receipt naming the milestone does **not** satisfy the
rule — it asks for a check-in, not for a line mentioning the milestone. Asserted.

**The boundary is asserted rather than assumed.** Exactly 24h old still satisfies a `<= 24` window;
25h is reported. **A boundary written from memory is what produced two of my earlier mistakes in this
port**, so this one is computed from the constant and tested at both sides.

`now` is a parameter rather than read inside the rule, so the tests need no mocked clock — and **the
same question can be asked about a fixed instant**, which is what the Python cannot do.

## Verification

| check | result |
|---|---|
| Rust vs Python | **0 findings; Python all pass** |
| all three milestone spellings | asserted |
| a non-heartbeat receipt does not satisfy it | asserted |
| the 24h boundary, both sides | asserted |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **311 passed** (7 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | exit 0, **32 implemented** |

## Limitations, stated

- **`check` still delegates.** Eight rules remain.
- **The rule reads the whole ledger, newest-first.** The Python reads the current month and, when the
  ledger is short, the previous month — **a bounded read**. This reads every shard. On the current
  ledger the counts agree (0 and 0); **on a ledger large enough for that to matter, the two could
  differ**, and this is a deliberate simplification rather than parity.
- **A receipt with an unparseable timestamp is ignored rather than reported.** The Python has the same
  behaviour via its own parse, so a milestone whose only heartbeat has a corrupt timestamp reads as
  having **no heartbeat** — **the right verdict from the wrong evidence**, and neither implementation
  says which.
- **`registry_vs_github` is still unported**, and **its GitHub dependency is unverified** — I am not
  repeating the mistake of describing it before reading it.
