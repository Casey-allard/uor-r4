# The stale-install warning pointed at the wrong binary

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** PM queue item 1 (PATH install refresh)

## What labs were actually hitting

The harness warns when your **checkout** is behind `origin/main`, and prescribes a **rebuild**. But the
binary a lab runs from a shell is the **installed copy** on PATH (`~/.cargo/bin/uor-harness`), and a rebuild
never touches it. So the advice misses the binary in front of you, and the warning returns every session.

**Worse than a wrong instruction: on a current checkout it said nothing at all.** Measured on this machine -
the install was from 13 Oct 23:30, the checkout's build from 14 Oct 06:57, and because HEAD == `origin/main`
the staleness check returned `Current`, so a lab running a seven-hour-old binary got **complete silence**.

## The change

`version::placement(repo)` decides, from `std::env::current_exe()`, whether the running binary is this
checkout's build or something else, and `placement_line` names the path with the command that actually
refreshes it:

```
this is /Users/casey.allard/.cargo/bin/uor-harness, not this checkout's build, so rebuilding will NOT
refresh it - install the current one with: cargo install --path crates/uor-r4-harness --offline
```

It is printed by `uor-harness version` and appended to the check-in warning. Both paths are canonicalised
before comparison, because `current_exe` resolves symlinks on some platforms and not others and a repo may be
reached through one.

## Verified by running it, in all three cases

| case | result |
|---|---|
| the PATH install, run from a current checkout | **names itself and gives the refresh command** |
| the checkout's own build | **silent** (no false positive) |
| `heartbeat` from the PATH install | **prints the line**, then checks in |

Two unit tests hold the decision: an outside binary is named and the line says a rebuild does not help; the
checkout's own build produces nothing. **The second is the one that keeps it honest** - a false positive would
print on every command of every lab that built locally.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --lib version::` | 9 passed (2 new) |
| fmt, clippy `-D warnings` | clean |
| unittest, `parity.py`, interlock | exit 0, exit 0, 1641 = baseline |

## Stated rather than hidden

* **The install on this machine was refreshed** (`~/.cargo/bin/uor-harness`, 14 Oct 11:16), which is the
  operational half of the item. Other labs' machines are theirs; the code is what generalises.
* **`docs/harness/BOOTSTRAP.md` now documents the refresh**, because a new lab should not have to meet the
  warning to learn the command.
* **The check is about the binary's PATH, not its build time.** A binary built into the checkout and then
  copied elsewhere is reported as outside, which is correct for the purpose (it is the copy a rebuild will
  not refresh) and is why the line names the path rather than claiming a version.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 2 (unread-note surfacing).
