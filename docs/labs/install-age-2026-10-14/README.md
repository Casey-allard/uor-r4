# The install warning was about location; it should have been about age

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line - tooling · **Count:** 0/3
**Owning work:** follow-up to PM queue item 1 (#2476), found by using it

## What was wrong with my own fix

#2476 made the check-in name the binary you are actually running instead of prescribing a rebuild of a
checkout you are not using. The condition, though, was **location**: any binary outside the checkout produced
the line. So a **freshly installed, newer** copy triggered it on every command - measured here at 12:59
installed against a 06:57 checkout build - **a permanent nag for every lab using the documented `uor-harness`
on PATH, which is worse than the silence it replaced.**

A location is not a staleness. A comparison is.

## The change

`is_stale_install(exe_time, built)` - pure, and `false` when either time is unknown, because with nothing to
compare silence beats a guess. `newest_build` finds the newest binary the checkout has built (
`local/harness-target` first, then `target`), and `placement_line` reports only when the running copy is
**older** than that. The message names both paths and the install command, now including
`--target-dir local/harness-target` - because `target/` in this repo is a **broken symlink** to a volume that
no longer exists, so a plain `cargo install` fails with `Not a directory (os error 20)`.

## Verified by running it, both directions

| case | result |
|---|---|
| install 13:55, checkout build 13:52 | **silent** - the nag is gone |
| a build later than the install | **warns**, naming both paths and the install command |
| the checkout's own build | silent (unchanged) |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | **501 passed, 0 failed, 6 ignored** |
| `cargo test --lib version::` | 10 passed (1 new) |
| fmt, clippy `-D warnings` | clean |
| unittest, `parity.py`, interlock | exit 0, exit 0, 1641 = baseline |

## Stated rather than hidden

* **The comparison is between file mtimes**, so it answers "is the copy I am running older than something
  this checkout built?" and not "which source is newer?". A binary built elsewhere and copied in is judged by
  its mtime, which is the honest proxy for the question the line asks.
* **No build in the checkout means no line** - there is nothing to compare against, and a guess would be worse
  than silence.
* **Two failed attempts and a reclaim are part of this record.** The first two edits were lost to tooling
  (an edit tool whose view of a just-created worktree lagged, and a branch reclaimed by another lab running
  `uor-hygiene --apply`, which removes clean worktrees sitting at `origin/main`); the work was redone in a
  single uninterrupted pass. My first splice also ate the function's closing brace, caught by the compiler.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
