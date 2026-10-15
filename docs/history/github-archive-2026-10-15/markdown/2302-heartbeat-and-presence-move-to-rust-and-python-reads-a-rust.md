# #2302 — Heartbeat and presence move to Rust, and Python reads a Rust check-in (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:47:02Z
- **Updated:** 2026-10-13T05:49:57Z
- **Closed:** 2026-10-13T05:49:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2302
- **Merged:** 2026-10-13T05:49:32Z
- **Merge commit:** 0c8a1722a608f2bed033e5be0a60a02d132cc677
- **Base / Head:** main / UOR-Foundation:harness/rust-heartbeat
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Rust replaces Python.

## What and why

You said: *"you can just not use any python at all."* That's the direction, and this is the next
piece of it. This unit takes the command labs use **most often** — `heartbeat`, at least every 90
minutes, in every lab, in every session.

`uor-harness heartbeat` now publishes presence to **both tiers** and records a receipt, writing them
from one record so they cannot disagree about what a lab is doing. `presence` and the atomic write
live in `crates/uor-r4-harness/src/presence.rs`.

## Result

Decision: **KEEP** (tooling).

### Verified cross-language, on the check-in path

Rust's heartbeat in a scratch repo produced three artifacts, and **Python read all three back**:

```
$ uor-harness heartbeat --repo <scratch> --lab kimi -m m1 --doing "interop" --next "..."
checked in: <scratch>/docs/harness/presence/kimi.json
  <scratch>/docs/harness/ledger/kimi-2026-10.jsonl     <- durable receipt
  <scratch>/docs/harness/presence/kimi.json            <- committed record
  <scratch>/live/presence/kimi.json                    <- live tier

Python: lab=kimi milestone=m1 via=durable doing='interop test' next='verify python reads it'
        receipts: kind=heartbeat lab=kimi milestone=m1 next=set
```

Same interop proof the receipt writer got, now for the path every lab runs.

### Three deliberate properties

| property | why |
|---|---|
| write is **atomic via rename** | a reader never sees a partial presence file |
| an unwritable live tier **does not fail the check-in** | the committed record is the commitment; the live tier is an optimisation |
| the writer never asserts `source` | `source` is a *read-time* label saying which tier answered — a writer claiming it would be claiming how it will be read |

### Two bugs of my own, both worth recording

**1. Three earlier edits silently did nothing.** `cargo fmt` had reformatted the subcommand flag
arrays into one-per-line form, so my string-replacement edits asserted against text that no longer
matched — and I did not check the result. `-m` was therefore rejected as an unknown option and
printed a **usage dump instead of an error**, which is exactly why it went unnoticed: it looked like
help, not failure. Editing source by matching formatted text is the mistake.

**2. A latent difference between the implementations.** Python **canonicalises symlinks** in the
repo root (`/var` → `/private/var` on macOS) while Rust uses `--repo` verbatim. Same directory,
different string. It does not break the interop test, but two implementations should not disagree
about the identity of the repository they are reading, and on a symlinked checkout they currently do.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 44 passed + 1 doc-test
python3 -m unittest                   -> 302 passed
python3 scripts/harness/cli.py check  -> 51 rules, 0 errors
python3 scripts/harness/parity.py     -> Rust-written receipt read back by Python: ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden — the port is ~20% done

**2,404 Rust lines against 11,566 Python across 19 modules.** Heartbeat, presence, receipt and the
format check are Rust. **Not ported:** `check`'s 51 rules, `prep`, `status`, `briefing`, `mistakes`,
`prlint`, `arena`, `budget`, and `parity.py`/`doccheck.py` themselves. At this pace the full removal
is many sessions, not one, and I would rather say that than imply otherwise.

The Python harness **stays authoritative** until the Rust path passes the same checks — the labs keep
running Python, and cutting over early would risk the shared record for a speed win nobody asked for.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

