# `status --at`: giving Python the clock injection Rust already had

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the parity defect found while shipping `registry_vs_github`

## The defect this closes

`parity.status_receipts_agreement` passed a fixed instant to Rust and **not** to Python:

```python
run([str(binary), "status", "--repo", str(REPO), "--at", "2026-10-13T02:52:00-04:00", "--lab", "parity"])
proc = subprocess.run([sys.executable, .../"cli.py", "status"], ...)   # no --at
```

**Python's `status` had no `--at` at all** — the flag appeared only in `parity.py`. So the check
compared **Rust at an injected instant** against **Python at the wall clock**, and it could agree only
while the machine's local date happened to equal the injected date.

**Evidence it was time-dependent and not anyone's change:** `main` passed this check at **`23:57Z`**;
the next run failed at **`00:19Z`** when the UTC date rolled over. **It failed for roughly six hours
of every US evening, and it passed on `main` by about twenty minutes.**

## The fix

**`--at` on Python's `status`, wired through `core.set_clock()`**, which `today()` and `month()` now
consult. Rust already had exactly this (`flag_value(args, "--at").unwrap_or_else(now_rfc3339)`).

**Injected at the top of `cmd_status`, before anything reads the clock** — the receipts buckets, the
24-hour clauses and the month shard must all see the same instant, and setting it after
`build_status` would inject the clock for nothing.

`parity` now passes the same instant to both sides.

## Verified, and the sensitivity shown directly

Both implementations, same instant — **byte-identical**:

```
2026-10-13: 188 receipts (skill 0, team 1, heartbeat 143, context 0, prep 7, pr 0, other 37) · 2026-10: 234 (...)
```

And the sensitivity the check needs, measured by asking the two sides for **different** instants:

```
rust  : 2026-10-13: 188 receipts (skill 0, team 1, heartbeat 143, ...
python: 2026-10-14:   0 receipts (skill 0, team 0, heartbeat 0, ...
```

**Different clocks produce different buckets — which is exactly the disagreement that made the check
fail near midnight, and exactly what the fix removes.**

## An honest note on what I could not probe

**I tried to prove the fix by reverting Python's `--at` and watching parity fail — and it passed.**
With today's date equal to the injected date, the wall clock and the injection agree, so **the bug is
invisible right now.** The failure window had closed by the time I had the fix in hand.

**So the evidence here is the direct clock-sensitivity measurement above, not a red-then-green test.**
I could not reproduce the original failure on demand, and I am saying so rather than implying the
probe worked.

## I fixed the wrong call site first

**`parity.py` calls Python's `status` from two different checks** — `status_agreement` (line ~496) and
`status_receipts_agreement` (line ~564) — and **I edited the first one.** Local parity passed anyway
**because `status_agreement` does not compare date buckets**, so the edit changed nothing that the
check looks at. **CI then failed the same way it had before, and I found it by reading which check was
failing rather than by re-reading my own diff.**

**This is the same failure mode as the anchor-based insert that reported success while changing
nothing**: an edit that lands in the wrong place looks exactly like an edit that worked. **What caught
it was CI disagreeing with a local pass** — and then reading the failing check's name instead of
assuming my fix was live.

The wrong site is reverted and the right one injects the instant.

## Verification

| check | result |
|---|---|
| both sides at one instant | **byte-identical receipts line** |
| different instants | **different buckets** (the sensitivity) |
| injected instant controls `today()`/`month()` | asserted |
| a malformed instant falls back rather than raising | asserted |
| `python3 -m unittest discover -s tests/harness -t .` | **330 passed** (4 new) |
| `cargo test -p uor-r4-harness` | 323 passed |
| `uor-harness check` | 0 errors |
| `parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |

## Limitations, stated

- **The clock is injected per-process, not per-call.** `core.set_clock` is module state, so a caller
  that injects and does not reset affects everything later in that process. **The CLI sets it once per
  invocation, which is the only current caller**, and the tests reset it in `tearDown`.
- **`parity` still hard-codes `2026-10-13T02:52:00-04:00` in two places.** Those should be one
  constant, and **a stale hard-coded instant will eventually point outside the ledger window** and
  make this check fail for a new reason.
- **The `--at` flag is now public on `status`.** It was added for injection, and a lab that passes it
  gets a render for an instant that is not now — **which is the point, and is worth knowing before
  reading a `--at` output as current state.**
- **I did not fix the other hard-coded instant** at `parity.py:484`, which passes `--at` to Rust for a
  different check. **It may have the same wall-clock asymmetry**; I did not look.
