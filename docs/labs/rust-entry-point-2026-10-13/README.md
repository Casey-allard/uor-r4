# `uor-harness` is the entry point. Python is behind it.

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** owner direction 13 October ("why are we using python?"); D27

## What was wrong

**11,566 lines of Python and 8,000 of Rust, and every lab typed `python3`.** Measured: **118
invocations across 44 files**, 79 of them normative.

The port was real — presence, receipts, the D22 gate, shards, brief, notes — but **`cli.py` was still
the front door.** I had even made `cli.py heartbeat` shell out to Rust and presented that as progress.
It was not: it moved the implementation and left the interface, so four labs could not tell whether
they were using the port at all. **The number that mattered was never Rust lines; it was how many
times a lab typed `python3`, and eleven units of work had not moved it once.**

## Result

Decision: **KEEP** (tooling).

`uor-harness` is the interface. **An unported command delegates to the Python implementation and says
so on every call**, to stderr and not suppressible:

```
$ uor-harness check
uor-harness: note - "check" is not ported to Rust yet; running …/scripts/harness/cli.py check
harness check: 57 rules, 19 failed (0 error, 18 warn)
```

**79 normative invocations rewritten** across `AGENTS.md`, `CONTRIBUTING.md`, the PR template,
`docs/harness/**` (including every skill and every lesson), and `docs/labs/session-goal.md`. Dated
`docs/labs/<topic>-<date>/` records are **exempt** — they are evidence of what was run at the time, and
rewriting them would falsify the record.

### A defect caught on the way, which is the reason to be careful here

The Rust `check` validates **state formats — one rule of the 57** the Python `check` runs. Exposing it
as `check` would mean a lab runs `uor-harness check`, sees *"state formats ok"*, and believes it has
validated its work **while 56 rules never ran**. That is the worst possible outcome for the command
labs trust most.

So **`check` stays delegated** and the Rust checker is explicit: **`check-formats`**. Verified: full
`check` through the entry point runs all 57 rules and exits 0.

### Delegation had to be decided BEFORE flag parsing

The first version rejected unknown flags before it knew whether it was delegating, so
`check --only <rule>` failed with *"unknown option --only"* — delegation only worked for commands whose
flags happened to be in the lists, which defeats the point. The command is now classified first, and an
unported command's flags are forwarded verbatim.

Second bug in the same place: forwarding `--only` **without its value** left the value to be read as
the next command word, so the binary tried to run a command called `harness.state_formats`. And
`--repo` was forwarded *and* added, which argparse rejects as a duplicate.

## The guard

New `check` rule **`harness.entry_point`** — **error** severity. No normative doc may invoke
`python3 scripts/harness/cli.py`. A rule rather than a note, because *"remember the new command"* is
exactly the instruction that decays.

`doccheck` also learns the **Rust-only commands** (`onboard`, `presence`, `registry`, `prep-render`,
`prep-check`, `shard-check`, `check-formats`, `ledger-sync`, `notes`); it was reporting
`uor-harness onboard` as `unknown subcommand 'onboard'` in six places, because it validated the
interface against one implementation of it.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | 125 passed |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed **on 3.15 and 3.9** |
| `cli.py check` | **0 errors** (58 rules) |
| `harness.entry_point` | all pass |
| delegated `check` | **57 rules, 0 errors, exit 0** |

## Limitations, stated

- **Most commands are still Python.** This changes the *interface*, not the implementation: `check`,
  `status`, `prep new`, `skill`, `team`, `mistakes`, `arena`, `budget`, `lint` and `git` all delegate
  today. The notice on each call is a rolling measure of how much port remains, and it is meant to be
  uncomfortable.
- **`uor-harness` must be built and on `PATH`.** A lab without the binary has no harness now. The
  docs say how; nothing verifies it, and `onboard` is the only place that would notice.
- **The delegation costs a subprocess per call** and passes `--repo` explicitly, so a lab that relied
  on the current directory rather than `--repo` may see a path difference.
- **`harness.entry_point` scans `discover_docs`**, so a normative doc outside that set could still
  invoke `cli.py` unnoticed.
- **The Rust `presence` shadows the Python `multilab presence`** and they are not the same command:
  the Rust one reads both tiers, the Python one has extra columns. Both are accepted by `doccheck`,
  which is a wart rather than a decision.
