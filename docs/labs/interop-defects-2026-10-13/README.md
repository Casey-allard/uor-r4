# Two interop defects: a kind the checker rejected, and two answers about staleness

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** Codex's report (comment `6146770185`); PM routing on #2028; D27

Codex reported both while onboarding to the Rust harness. **Both are real**, both are reproduced
below, and both are the same shape: **two implementations that are each self-consistent and disagree.**

## Defect 1 — the generic receipt command created a record its own checker rejects

`cmd_receipt` defaulted `--kind` to `receipt` (`main.rs:707`); the authoritative kind set
(`receipts.KINDS`) did not include it. So **every run of `uor-harness receipt` without an explicit
kind emitted a record the validator refuses** — including the onboarding exercise. Codex found it as
`unknown kind receipt` failing advisory CI on #2328.

**The fix is in the kind set, not the default, and the reason matters.** `receipt` is documented as a
kind in the Rust source (`receipt.rs:36`), the generic command exists for exactly this, and
`parity.py` already drives it with `--kind parityprobe`. Renaming the default would have been a second
name for the same thing. **A command must never be able to create a record its own checker rejects.**

Two lists moved, and the second was a real bug the first exposed:

- `receipts.KINDS` gains `receipt` — the authoritative set.
- `status.rs::ALL_KINDS` gains it too, because the Rust reader was **silently dropping one receipt the
  Python reader counted.** `parity.py` caught it the moment the Python set was corrected:
  `rust 110 (… other 42) != python 111 (… other 43)`.

## Defect 2 — two answers to "how long is a check-in good for"

`cmd_presence` held `const TTL_MINUTES: i64 = 30` while the presence module and the heartbeat used
**5**. So the command a lab reads to decide whether a peer is still working said thirty minutes, and
the check-in said five.

Now one constant, read from the module:

```
$ uor-harness presence
4 lab(s) known (stale after 5m) · live tier: ~/.local/share/uor-r4/harness
```

## The guard, so this class of defect stops recurring

A new `parity.py` check, `constants_agreement`, compares what the two implementations tell labs:

- the **presence TTL** from the Rust `presence --json` against `multilab.PRESENCE_TTL_MINUTES`; and
- the **`--kind` default** parsed out of `main.rs` against the authoritative kind set.

**Disagreement here is invisible without a check** — each side is self-consistent, so only the lab
reading one of them can tell. That is why both defects survived to be found by a human instead of by CI.

## The existing row

Codex noted the already-emitted row cannot be corrected without violating the append-only guard, and
left it alone. **That was the right call**, and with `receipt` now a legitimate kind the row is valid
where it stands — no correction, no shrinking, guard intact.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | 113 passed |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed **on 3.15 and 3.9** |
| `cli.py check` | 0 errors |
| `parity.py` | **all eleven comparisons ok**, including the new constants check |

## Limitations, stated

- **`constants_agreement` reads the Rust default by regex** on `main.rs`. It catches the current
  spelling; a refactor that computes the default differently would make the check silently skip
  (it returns no finding when the pattern does not match). A skip is quieter than a failure.
- **Only two constants are covered.** `RECEIPT_KINDS`, the panel kinds and the claim staleness windows
  are still unpinned across the two implementations.
- **The `receipt` kind is now accepted but barely used.** Nothing routes it, and the roll-up shows it
  as `other`; it exists for the generic command and for rows already written.
- **The unread-note item is still not started** — this unit took the two defects Codex's report named,
  which the PM routed as high priority.
