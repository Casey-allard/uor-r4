# #2476 — Name the binary that is stale: refresh the PATH install, not just the checkout (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T15:19:49Z
- **Updated:** 2026-10-14T15:22:55Z
- **Closed:** 2026-10-14T15:22:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2476
- **Merged:** 2026-10-14T15:22:39Z
- **Merge commit:** 334eac5be736619dd7fa5b404f75817d72681607
- **Base / Head:** main / UOR-Foundation:deepseek/install-refresh
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What labs were actually hitting

The harness warns when your **checkout** is behind `origin/main` and prescribes a **rebuild** — but the binary a
lab runs from a shell is the **installed copy on PATH** (`~/.cargo/bin/uor-harness`), which a rebuild never
touches. The advice misses the binary in front of you, so the warning returns every session.

**And on a current checkout it said nothing at all.** Measured here: the install was from 13 Oct 23:30, the
checkout's build from 14 Oct 06:57, and because HEAD == `origin/main` the staleness check returned `Current` —
so a lab running a seven-hour-old binary got **complete silence**.

## The change

`version::placement(repo)` decides from `std::env::current_exe()` whether the running binary is this checkout's
build, and `placement_line` names the path with the command that actually refreshes it:

```
this is /Users/casey.allard/.cargo/bin/uor-harness, not this checkout's build, so rebuilding will NOT
refresh it - install the current one with: cargo install --path crates/uor-r4-harness --offline
```

Printed by `uor-harness version` and appended to the check-in warning. Both paths are canonicalised first,
because `current_exe` resolves symlinks on some platforms and not others.

## Verification: run in all three cases

| case | result |
|---|---|
| the PATH install, from a current checkout | **names itself, gives the refresh command** |
| the checkout's own build | **silent** — no false positive |
| `heartbeat` from the PATH install | **prints the line**, then checks in |

Two unit tests hold the decision: an outside binary is named and the line says a rebuild does not help; the
checkout's own build produces nothing. **The second is what keeps it honest** — a false positive would print on
every command of every lab that built locally.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | **495 passed, 0 failed, 6 ignored** |
| `cargo test --lib version::` | 9 passed (2 new) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| unittest, `parity.py`, interlock | exit 0, exit 0, 1641 = baseline |

## Checklist

- [x] own worktree, fix with tests, record at `docs/labs/install-refresh-2026-10-14/README.md` plus a newest-first current-state entry, protected PR
- [x] Checks run at this exact head: `cargo test -p uor-r4-harness --offline` -> 495 passed/0 failed; `cargo fmt -- --check` -> clean; `cargo clippy --all-targets -- -D warnings` -> 0 findings; `python3 -m unittest discover -s tests/harness -t .` -> exit 0; `python3 scripts/harness/parity.py` -> exit 0; `python3 scripts/check_no_softmax.py` -> 1641 = baseline; plus the three-case run above
- [x] limitations stated below

### Stated rather than hidden

* **The install on this machine was refreshed** (`~/.cargo/bin/uor-harness`, 14 Oct 11:16) — the operational
  half of the item. Other labs' machines are theirs; the code is what generalises.
* **`docs/harness/BOOTSTRAP.md` now documents the refresh**, so a new lab need not meet the warning to learn
  the command.
* **The check is about the binary's PATH, not its build time.** A binary built into the checkout and copied
  elsewhere is reported as outside — correct for the purpose (a rebuild will not refresh that copy), and why the
  line names the path rather than claiming a version.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

