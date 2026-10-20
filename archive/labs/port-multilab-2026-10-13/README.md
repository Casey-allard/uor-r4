# D30 port 2: `multilab` is Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 2 of the series

## What moved

`crates/uor-r4-harness/src/multilab.rs` — new, **11 tests**. `uor-harness multilab presence|claims|touch`
is Rust and **out of the delegated set**. `sync-live` and `migrate-ledger` still delegate **by name**,
so the progress measure shrinks action by action.

## The contract that mattered most

**`touch` exists for hooks and must not erase the task.** The captured Python contract: it refreshes
the timestamp and **preserves `doing`, `milestone` and `next`**. Blanking them would show a working lab
as having nothing to do — **worse than showing it stale, because stale is at least true.**

Tested directly: a lab with `doing` set, touched, still has it.

## Reuse rather than reimplementation

- **Staleness is delegated to `PresenceRow::is_stale`/`age_minutes`**, which already answer "cannot
  tell" as stale. An unparseable timestamp is **not evidence of freshness**, and reporting `0m ago`
  would make a corrupt file look like the most current lab on the board. Tested with a corrupt file.
- **`claims` reads through the `team` port**, not a second reader of `claims.json`. **Two readers of
  one file will eventually disagree** — that is the defect class this whole line keeps producing.
- **State is derived at read time and never stored**; a stored state is wrong the moment it is written.

## Two vocabulary and interface decisions

- **The board says `committed` where the reader says `durable`.** The tiers are the storage story; the
  board's answer is "real time, or from the last merge". Normalised at the board so one vocabulary
  belongs to each surface.
- **A missing `doing` prints `doing` as the column header rather than an empty format slot** — a
  header that renders as nothing is a header nobody can read.

## One bug found, same class as twice before

**`--lab` beat the subcommand word.** Global flags consumed by the top-level parser are pushed into
`rest` for the subcommand's benefit, so they never appeared in the subcommand value list — and their
**values** became candidate actions. `--repo <dir>` did this in the `team` port; `--lab <name>` did it
here, producing `unknown action "deepseek"`. Both are now named in one `GLOBAL_VALUE_FLAGS` list, with
a comment that a new global flag must be added there.

**This is the third instance of the same defect**, and the fix is now a list rather than a special
case — which is the only version that stops the fourth.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **163 passed** (11 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| `multilab touch` / `presence` vs captured Python | same shape and fields |
| delegation notice for `multilab` | gone for the three ported actions |

## Limitations, stated

- **`sync-live` and `migrate-ledger` still run Python.** Three of five actions moved.
- **The timestamp differs from Python's.** Python wrote ISO with the **local offset**
  (`2026-10-13T13:54:41-04:00`); `touch` here writes **UTC `Z`** (`17:57:05Z`). Both parse, and the
  board orders correctly, but a diff between an old and new presence file shows the format change.
- **`GLOBAL_VALUE_FLAGS` is a list a human must maintain.** A new global flag added to the parser and
  not to this list reproduces the bug a fourth time, and nothing fails to warn.
- **The board is not compared byte-for-byte against Python's output** — the columns were checked by
  eye and the JSON by field. The fixture records the shape, not the exact spacing.
