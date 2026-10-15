# #2327 — One heartbeat: Python delegates to Rust, and five minutes is the default (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T13:22:33Z
- **Updated:** 2026-10-13T13:25:07Z
- **Closed:** 2026-10-13T13:24:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2327
- **Merged:** 2026-10-13T13:24:48Z
- **Merge commit:** 2449d6b2cd3af928b8c50c4f4acc60c02bd971b9
- **Base / Head:** main / UOR-Foundation:deepseek/retire-python-heartbeat
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Closes the gap named in #2326.

## Result

Decision: **KEEP** (tooling).

#2326 built the live-tier five-minute heartbeat — **but only in Rust.** `cli.py heartbeat` kept its own implementation: a durable presence copy written into a checkout, and a 30-minute window.

**So the system worked and no lab was on it**, because every lab runs the Python command.

Keeping both was the worse option, and not for tidiness: **two check-ins that record different things, with no way to tell which one a lab ran.** `cli.py heartbeat` is now a thin transport to `uor-harness heartbeat`.

### No fallback, deliberately

A fallback would run the old behaviour while the lab believed it had checked in — **invisible divergence**, which is the exact failure this removes. A missing binary now returns a usage error naming the fix:

```
heartbeat now runs in the Rust harness, which is not built here.
  build it once:
    CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness
```

### The flags and the receipt

The Rust heartbeat now accepts the session contract's flag names for the line, the count and the headline, so a lab following the contract does not meet an unknown option on the command it types every few minutes. And the receipt carries those fields through — **the D21 count is what the owner reads**, and the first version dropped one and never recorded the other.

## Measured

```
$ python3 scripts/harness/cli.py heartbeat -m m1 --line "retire python heartbeat" --count 0/3
checked in: ~/.local/share/uor-r4/harness/presence/deepseek.json

$ git status --short          # no coordination write to the checkout
$ tail -1 ~/.local/share/uor-r4/harness/ledger/deepseek-2026-10.jsonl
  kind=heartbeat line='retire python heartbeat' count='0/3' lab=deepseek
```

## The documentation now agrees with the code

`AGENTS.md`, `BOOTSTRAP.md` and `docs/harness/README.md` say **five minutes** and name the Rust path.

**`AGENTS.md` already said *"heartbeat every five minutes"* while the code used thirty.** That is the real reason this unit matters — the rule and the code now agree.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 113 passed
python3 -m unittest discover -s tests/harness  -> 319 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check           -> 0 errors
python3 scripts/check_no_softmax.py            -> OK (1648, unchanged)
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/heartbeat-single-path-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **Every lab must build the binary once.** `cli.py heartbeat` is useless without `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`. That is a **real adoption cost and it falls on each lab, not on this PR.** The error names the command, but the first check-in after this merges will fail on a lab that has not built.
- **The old Python heartbeat function survives, unreached, in `status.py`.** Dead code a reader will find and might call. **Deleting it and its tests is the honest completion and is not done here** — I would rather name it than half-do it.
- **`--print` (the status card) is not carried over.** The Rust heartbeat records and publishes but does not render the card, so a lab that used `heartbeat --print` loses that output.
- **Nothing enforces the cadence.** The board reports staleness; no hook makes a lab check in.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

