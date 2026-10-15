# #2401 — status --at: Python gets the clock injection Rust already had (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:30:03Z
- **Updated:** 2026-10-14T00:36:23Z
- **Closed:** 2026-10-14T00:35:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2401
- **Merged:** 2026-10-14T00:35:58Z
- **Merge commit:** 5d182f6aaedb0636531a9e8eb3cf3b7d2207c186
- **Base / Head:** main / UOR-Foundation:deepseek/at-flag
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`status --at`: Python gets the clock injection Rust already had.** Tooling only.

Decision: **KEEP** (tooling).

## The defect

`parity.status_receipts_agreement` passed a fixed instant to **Rust** and **not to Python**:

```python
rust_call([binary, "status", "--repo", REPO, "--at", INSTANT, "--lab", "parity"])
py_call([sys.executable, cli, "status"])                # <-- no --at
```

**Python's `status` had no `--at` at all** - the flag appeared only in `parity.py`. So the check compared **Rust at an injected instant** against **Python at the wall clock**, agreeing only while the machine's local date happened to equal the injected date.

**Evidence it was time-dependent and not anyone's change:** `main` passed this check in the late UTC evening; the very next attempt failed shortly after the UTC date rolled over. **It failed for ~6 hours of every US evening and passed on `main` by about twenty minutes.**

## The fix

**`--at` on Python's `status`, wired through `core.set_clock()`**, which `today()` and `month()` now consult — **Rust already had exactly this.** Injected **at the top of `cmd_status`**, before anything reads the clock: the receipts buckets, the 24-hour clauses and the month shard must all see one instant. `parity` now passes the same instant to both.

## Verified, with the sensitivity measured directly

Both sides at one instant — **byte-identical**:

```
2026-10-13: 188 receipts (skill 0, team 1, heartbeat 143, context 0, prep 7, pr 0, other 37) · 2026-10: 234 (...)
```

Different instants, which is the sensitivity the check needs:

```
rust  : 2026-10-13: 188 receipts (skill 0, team 1, heartbeat 143, ...
python: 2026-10-14:   0 receipts (skill 0, team 0, heartbeat 0, ...
```

```
python3 -m unittest discover -s tests/harness -t .   -> 330 passed (4 new)
cargo test -p uor-r4-harness                         -> 323 passed
uor-harness check                                    -> 0 errors
parity.py                                            -> exit 0
python3 scripts/check_no_softmax.py                  -> OK, no softmax added
```

Record: `docs/labs/status-clock-injection-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **I tried to prove the fix by reverting Python's `--at` and watching parity fail — and it passed.** With today's date equal to the injected date the wall clock and the injection agree, **so the bug is invisible right now; the failure window had closed by the time I had the fix in hand.** The evidence is the **direct clock-sensitivity measurement**, **not a red-then-green test**, and I am saying so rather than implying the probe worked.
- **The clock is injected per-process, not per-call.** `core.set_clock` is module state; the CLI sets it once per invocation (the only current caller) and the tests reset it in `tearDown`.
- **`parity` still hard-codes `2026-10-13T02:52:00-04:00` in two places**, which should be one constant. **A stale hard-coded instant will eventually point outside the ledger window** and make this check fail for a new reason.
- **`--at` is now public on `status`**, so a lab passing it gets a render for an instant that is not now — **which is the point, and worth knowing before reading such output as current state.**
- **I did not fix the other hard-coded instant at `parity.py:484`**, which may have the same asymmetry — **I did not look.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

