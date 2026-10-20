# One heartbeat: Python delegates to Rust, and the five-minute cadence is the default

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the gap named in #2326; owner direction 13 October; D27

## The gap this closes

#2326 built the live-tier five-minute heartbeat — but only in Rust. **`cli.py heartbeat` kept its own
implementation**: a durable presence copy written into the checkout, and a 30-minute window. So the
system worked and **no lab was on it**, because every lab runs the Python command.

Keeping both was the worse option, and not for tidiness: two check-ins that record different things,
with no way to tell which one a lab ran. **`cli.py heartbeat` is now a thin transport to
`uor-harness heartbeat`.**

## What changed

| | before | after |
|---|---|---|
| `cli.py heartbeat` | its own implementation, 30-minute window, checkout write | **delegates to `uor-harness heartbeat`** |
| Rust flags | `--milestone --doing --next` | **`--line --count --headline`** too, so the session contract works |
| receipt fields | `line` dropped, no `count` | **both recorded** — the D21 count is what the owner reads |
| binary missing | n/a | **usage error naming the build command** |

**No fallback, deliberately.** A fallback would run the old behaviour while the lab believed it had
checked in — invisible divergence, which is the failure mode this removes. A missing binary now says
so and names the fix.

## Measured

```
$ python3 scripts/harness/cli.py heartbeat -m m1 --line "retire python heartbeat" --count 0/3
checked in: ~/.local/share/uor-r4/harness/presence/deepseek.json

$ git status --short          # no coordination write
$ tail -1 ~/.local/share/uor-r4/harness/ledger/deepseek-2026-10.jsonl
  kind=heartbeat line='retire python heartbeat' count='0/3' lab=deepseek
```

And with no binary built, the command explains rather than silently doing something else:

```
heartbeat now runs in the Rust harness, which is not built here.
  build it once:
    CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness
```

## Documentation

`AGENTS.md`, `BOOTSTRAP.md` and `docs/harness/README.md` now say **five minutes** and name the
Rust path. `AGENTS.md` already said *"heartbeat every five minutes"* while the code used thirty —
**the rule and the code now agree**, which is the real reason this unit matters.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | 113 passed |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed **on 3.15 and 3.9** |
| `cli.py check` | 0 errors |
| `check_no_softmax.py` | OK — 1648, unchanged |

## Limitations, stated

- **Every lab must build the binary once.** `cli.py heartbeat` is now useless without
  `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`. That is a real adoption
  cost and it falls on each lab, not on this PR. The error message names the command.
- **The old Python heartbeat function still exists** in `status.py`, unreached. It is dead code that
  a reader will find and might call; deleting it and its tests is the honest completion and is not
  done here.
- **`--print` (the status card) is not carried over.** The Rust heartbeat records and publishes but
  does not render the card, so a lab that used `heartbeat --print` loses that output.
- **Nothing enforces the cadence.** The board reports staleness; no hook makes a lab check in.
