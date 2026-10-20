# #2374 — D30 check-port 3 of 10: the multilab rules are Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T20:39:57Z
- **Updated:** 2026-10-13T20:42:18Z
- **Closed:** 2026-10-13T20:42:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2374
- **Merged:** 2026-10-13T20:42:01Z
- **Merge commit:** a9499538a0e8b9e1e9ff762471fc0529251bf464
- **Base / Head:** main / UOR-Foundation:deepseek/check-multilab
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 3 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The three rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `multilab.presence_fresh` | warn | 2 warn | **2 warn** |
| `multilab.claims_one_owner` | error | all pass | 0 findings |
| `multilab.claim_stale` | warn | all pass | 0 findings |

Measured against the real repo **and the real live presence tier** — which is the point of the finding below.

## The 3.9 timestamp note the port inherits

`datetime.fromisoformat` rejected the `Z` suffix before Python 3.11 while the harness supports 3.9 — **and the Rust implementation writes `Z` timestamps.** A parser accepting one form would make a claim written by one implementation unreadable to the other, and **the staleness rule would silently stop firing on exactly those claims.**

`parse_claimed_at` accepts `Z`, `±HH:MM` and a bare time, with tests asserting the two zero-offset forms are the **same instant**, that `-04:00` is genuinely four hours later in UTC, and that **a bare date is refused rather than read as midnight** — inventing a time makes a claim's age wrong in the direction that **hides** staleness.

## `claim_stale` keeps "holding while working" apart from "holding and gone"

The whole rule. A lab that checks in keeps its claims; one that does not shows up here. **Both directions pinned on the same 48-hour-old claim**, with the message saying which of the two it is.

## Two of my own tests were wrong, and one nearly became a false bug report

**The day count.** I asserted **20744** from memory; the code produced **20739**, and **Python's `datetime` and BSD `date` both independently confirm 20739.** The code was right, the test was wrong — **the third time in this port.** The lesson is not "be careful": **an expected value written from memory is a guess wearing a test's clothes.**

**The probe, which is the more instructive one.** My first probe pointed the live tier at a **non-existent path** and read **3** findings against Python's **2** — and **I briefly believed I had found a bug in the port. I had not.** With an empty live tier the only presence visible was the **three committed records** in `docs/harness/presence/`, all old, all correctly reported stale.

**Pointing at an empty live tier does not test the rule; it tests a machine where nothing is running.** With the real tier, Rust reads **2**, exactly as Python does.

**That is the same failure this line keeps producing in a new place: I verified against a fixture I had invented rather than against the system, and the fixture's silence looked like the system's answer.**

## Three of the six rules are deliberately not here

- **`claim_not_merged`** asks GitHub for a merged PR, and the Python carries a note about a **false positive on live work** — `kimi/plan-refresh-2026-10-13` flagged as merged within minutes, fixed by asking GitHub **first** and answering "not established" rather than guessing, because **a rule that cries wolf on a lab's live branch is worse than no rule.** **A hasty port would reintroduce exactly the bug those comments exist to prevent.**
- **`no_shared_ledger`** and **`ledger_lab_recorded`** read receipt shards and belong with `receipts`.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 236 passed (15 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 11 implemented
```

Record: `docs/labs/port-check-multilab-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 3 of 10 modules; the coverage line now reads **11 of 40 rules**.
- **The agreement measurement is a hand-run probe I read and removed.** **Nothing in CI compares per-rule finding counts**, so a future edit could diverge from Python without a test failing — the inventory guard checks that an id is *claimed*, not that it behaves the same.
- **Nothing pins the 5-minute boundary against a real clock.** A lab sitting exactly at the threshold is **undefined between the two implementations**, and that is not resolved here.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

