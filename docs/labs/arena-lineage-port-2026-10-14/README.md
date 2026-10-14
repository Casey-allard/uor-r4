# `arena lineage` is Rust — and Python crashes on the path this port completes

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site (`arena`/`budget`), first of `arena`'s three subcommands

## What landed

**`uor-harness arena lineage <id>` runs in Rust**, and it brought `arena::load` with it — the reader that
`board` and `compare` will reuse, so the next unit is rendering rather than reading.

**Delegation: live call sites 1 → 2; unported commands 5 → 4.** The general fallback now carries
`budget` x2, and `arena board|compare` have their own named line. Same shape as the `panel` unit: the
site count rises while the unported count falls, and both are stated.

## Python crashes on this path

`arena lineage <unknown>` does not print the message its own code intends:

```python
emit(f"arena: no lineage for {args.id!r}", file=__import__("sys").stderr)   # arena.py:465
def emit(text: str, out: Any = None) -> None:                              # core.py:411
```

**There is no `file` parameter**, so the call raises `TypeError` and the command exits 1 with a
traceback. This port prints **the message the Python code intended** and exits **1** — the same exit
code the crash produces. Reproducing a traceback is not a fidelity target.

## Verified: both implementations, eight case groups

| case | result |
|---|---|
| `arena lineage <id>` for **all five** rows in `docs/harness/arena.json` | MATCH |
| a root row and a row with a parent chain | MATCH |
| `--json arena lineage <id>` (Python ignores the flag, so this does too) | MATCH |
| `arena board` (still delegated) | stdout + exit MATCH; stderr carries the delegation notice, by design |
| `arena lineage <unknown>` | exit 1/1; **stderr differs because Python tracebacks** |
| `arena lineage` with no id | exit 2/2, argparse usage text (the recorded divergence) |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **472 passed, 0 failed, 6 ignored** (was 468: 4 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## What this unit is for

An arena is **a regression check, not a leaderboard**: every row names its **parent**, and the report is
row by row against that parent rather than an aggregate, because "a candidate that gains 0.02 BPB while
losing three memory rows is a regression with a headline". `lineage` is what makes that possible — it
walks from a root down to the candidate, and **it stops on a repeated id**, because the arena file is
hand-written JSON and a parent loop must not hang the command.

## Stated rather than hidden

* **`load` treats a missing or unreadable `docs/harness/arena.json` as an EMPTY arena**, exactly as
  Python does: the arena starts empty, and a lab with a broken file gets the same answer as one that has
  measured nothing.
* **Delegation resolves `scripts/harness/cli.py` relative to `--repo`**, so `arena board` cannot run
  against a directory that has `docs/harness` but no `scripts/`. That is pre-existing behaviour of
  `delegate_to_python`, not something this unit changes, and it is why the `board` comparison runs in
  the real checkout.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `arena board` (rendering, `arena::load` is in place), then `arena compare` (91 lines), then
`budget show|scan`. Four subcommands left before the general fallback can be deleted.
