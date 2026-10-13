# D30 check-port 3 of 10: the `multilab` rules are Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; `check` module 3 of 10

## The three rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `multilab.presence_fresh` | warn | 2 warn | **2 warn** |
| `multilab.claims_one_owner` | error | all pass | 0 findings |
| `multilab.claim_stale` | warn | all pass | 0 findings |

Measured against the real repo **and the real live presence tier** — see below, because getting that
wrong is what happened first.

## The 3.9 timestamp note the port inherits

`_parse_claimed_at` accepts both offset forms because `datetime.fromisoformat` rejected the `Z` suffix
before Python 3.11 while the harness supports 3.9 — **and the Rust implementation writes `Z`
timestamps.** Without that, a Rust-written claim is unreadable by the Python harness on 3.9 and **the
staleness rule silently stops working.**

`parse_claimed_at` here accepts `Z`, `±HH:MM` and a bare time, with tests asserting the two zero-offset
forms are the **same instant** and that `-04:00` is genuinely four hours later in UTC. **A bare date is
refused rather than read as midnight** — silently inventing a time for a claim would make its age
wrong in the direction that hides staleness.

## `claim_stale` keeps "holding while working" apart from "holding and gone"

That distinction is the whole rule. **A lab that checks in keeps its claims; one that does not shows up
here.** Both directions are pinned: the same 48-hour-old claim is *not* reported when its lab has a
one-minute-old heartbeat, and *is* reported when its lab has published no presence at all — with the
message saying which of the two it is.

## Two of my own tests were wrong, and one of them nearly became a false bug report

### The day count

`days_from_civil(2026, 10, 13)` — I asserted **20744** from memory. The code produced **20739**, and
**Python's `datetime` and BSD `date` both independently say 20739.** The code was right; the test was
wrong. **That is the third time in this port**, and the lesson is not "be careful" — it is that
**an expected value written from memory is a guess wearing a test's clothes.** The corrected assertion
names both cross-checks.

### The probe, which is the more instructive one

My first probe passed `StatePaths::resolve(..., Some("/nonexistent-live"))` and read **3** findings for
`presence_fresh` against Python's **2** — and I briefly believed I had found a bug in the port.

**I had not.** With the live tier pointed at a path that does not exist, the only presence the rule
could see was the **three committed records** in `docs/harness/presence/` — all old, all correctly
reported stale. **Pointing at an empty live tier does not test the rule; it tests a machine where
nothing is running.** With the real tier, Rust reads **2**, exactly as Python does.

**This is the same failure this line keeps producing, in a new place: I verified against a fixture I
had invented rather than against the system, and the fixture's silence looked like the system's
answer.**

## Three of the six rules are deliberately not in this unit

- **`multilab.claim_not_merged`** asks GitHub whether a branch has a merged PR, and the Python carries a
  hard-won note about a **false positive on live work**: `kimi/plan-refresh-2026-10-13` was claimed and
  flagged as merged within the same couple of minutes, and the warning vanished once the remote-tracking
  ref arrived. The fix was to ask GitHub **first** and to answer "not established" rather than guess,
  because **a rule that cries wolf on a lab's live branch is worse than no rule.** That ordering, the
  unknown case and the `gh` dependency need porting together — **a hasty port would reintroduce exactly
  the bug those comments exist to prevent.**
- **`multilab.no_shared_ledger`** and **`multilab.ledger_lab_recorded`** read receipt shards and belong
  with the `receipts` module.

## Verification

| check | result |
|---|---|
| Rust vs Python, all three rules | **identical, measured with the real live tier** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **236 passed** (15 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `parity.py` | exit 0, **rule inventory: 11 implemented** |

## Limitations, stated

- **`check` still delegates.** 3 of 10 modules; the coverage line now reads **11 of 40 rules**.
- **The agreement measurement is a hand-run probe, not an automated comparison.** I ran it, read it, and
  removed it. **Nothing in CI compares per-rule findings counts**, so a future edit to these rules could
  diverge from Python without a test failing — the inventory guard checks that an id is *claimed*, not
  that it behaves the same.
- **Nothing pins `presence_fresh`'s boundary behaviour against a real clock.** The rule classifies at a
  5-minute boundary, and two implementations sampled seconds apart can legitimately disagree about a lab
  on the line — which is why the probe had to pin an instant. **A lab sitting exactly at 5 minutes is
  undefined between the two implementations**, and that is not resolved here.
