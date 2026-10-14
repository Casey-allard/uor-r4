# `arena compare` is Rust — and the whole `arena` command is now ported

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site (`budget`), which `arena` has now left entirely

## What landed

**`uor-harness arena compare <id>` runs in Rust**, text and `--json`. **The `arena` command is complete:**
`board`, `compare` and `lineage` are all Rust, and the only live `delegate_to_python` call site left is the
general fallback, carrying **`budget show|scan`**. Unported commands **3 → 2**.

## The comparison is the report the arena exists for

The arena is a regression check, not a leaderboard, so nothing is collapsed into one number. Two rules do
the real work:

* **Direction comes from the file's `bars` map and is never guessed.** An undeclared bar is reported as
  `undeclared` and **cannot be called a gain or a regression** — a 92% increase in candidates scored per
  query would otherwise read as an improvement.
* **A KEEP must report fresh transfer, as evidence rather than as the word.** `"transfer" in scope` is
  wrong because *"no transfer reported"* contains it; the rule is a `transfer` key in `scores`, or the
  regex `(?:fresh\s+)?transfer\s*[:=]?\s*[0-9]` over `scope`/`notes`.

## Verified: both implementations, fourteen case groups

| case | result |
|---|---|
| `arena compare <id>` for **all five** rows, text | MATCH |
| the same five, `--json` | MATCH |
| `arena compare nope`, text and `--json` | MATCH (exit 1/1) |
| `arena compare` with no id | exit 2/2, argparse usage text (the recorded divergence) |

The five rows cover the paths that matter: a **root row** (`comparable: false`, exit **0** — it is still
`ok`), a **KEEP with a transfer bar**, a panel change, an **undeclared bar**, and integer deltas.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **481 passed, 0 failed, 6 ignored** (was 475: 6 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Two details that would have diverged quietly

**`delta = round(b - a, 6)` keeps INT-ness when both sides are ints.** Python's `round(13, 6)` is the int
`13` and prints `13`; a float-only port prints `13.0` on every integer bar in this file. `delta_value`
tries `as_i64` on both sides first.

**The bars compared are the UNION of both `scores` maps, sorted.** A bar only one side reports becomes an
`UNAVAILABLE` row naming which side is missing it, never a delta.

## Stated rather than hidden

* **A decidable `seeds`/`rows` value that is not a number reads as 0**, where Python's `seeds or 0` would
  raise on a string. The arena file writes ints or null.
* **The `--json` result dict's key order is load-bearing** (`reason` last on both early returns), and this
  call has **`ensure_ascii` ON**, unlike `arena board`'s neighbouring one.
* **`budget show|scan` still delegate**, and `budget show` sits behind `collect()` — ~400 lines including
  the resource-ledger pointer-chain walk.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `budget show`, then `budget scan` — **two subcommands left before the general fallback can be
deleted and D30 is done.**
