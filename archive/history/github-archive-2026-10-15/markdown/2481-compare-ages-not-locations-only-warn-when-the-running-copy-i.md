# #2481 — Compare ages, not locations: only warn when the running copy is older (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T17:57:04Z
- **Updated:** 2026-10-14T17:59:59Z
- **Closed:** 2026-10-14T17:59:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2481
- **Merged:** 2026-10-14T17:59:39Z
- **Merge commit:** ae3aa4dc36b7b947270c66d91f263e0abeb7a436
- **Base / Head:** main / UOR-Foundation:deepseek/install-age
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What was wrong with my own fix (#2476)

#2476 made the check-in name the binary you are actually running, instead of prescribing a rebuild of a
checkout you are not using. But its condition was **location** — any binary outside the checkout produced the
line — so a **freshly installed, newer** copy triggered it on every command (**measured: 12:59 installed
against a 06:57 build**). That is a permanent nag for every lab using the documented `uor-harness` on PATH,
**worse than the silence it replaced**. A location is not a staleness; a comparison is.

## The change

`is_stale_install(exe_time, built)` — pure, `false` when either time is unknown, because with nothing to compare
silence beats a guess. `newest_build` finds the newest binary the checkout has built (`local/harness-target`
first, then `target`); `placement_line` reports only when the running copy is **older** than that, naming both
paths and the install command — now with `--target-dir local/harness-target`, because **`target/` here is a
broken symlink to a volume that no longer exists**, so a plain `cargo install` dies with
`Not a directory (os error 20)`.

## Verification: run in both directions

| case | result |
|---|---|
| install newer than the checkout's build | **silent** — the nag is gone |
| a build later than the install | **warns**, naming both paths |
| the checkout's own build | silent, unchanged |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | **501 passed, 0 failed, 6 ignored** |
| `cargo test --lib version::` | 10 passed (1 new) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| unittest, `parity.py`, interlock | exit 0, exit 0, 1641 = baseline |

## Checklist

- [x] own worktree, fix with tests, record at `docs/labs/install-age-2026-10-14/README.md` plus a newest-first current-state entry, protected PR
- [x] Checks run at this exact head: `cargo test -p uor-r4-harness --offline` -> 501 passed/0 failed; `cargo fmt -- --check` -> clean; `cargo clippy --all-targets -- -D warnings` -> 0 findings; `python3 -m unittest discover -s tests/harness -t .` -> exit 0; `python3 scripts/harness/parity.py` -> exit 0; `python3 scripts/check_no_softmax.py` -> 1641 = baseline; plus the two-direction run above
- [x] limitations stated below

### Stated rather than hidden

* **The comparison is between file mtimes**: it answers whether the copy you are running is older than
  something this checkout built, not which source is newer. That is the honest proxy for the question the line
  asks, and it is stated rather than implied.
* **No build in the checkout means no line** — nothing to compare against, and a guess would be worse than
  silence.
* **The tooling fought this one**: two edit attempts were lost (an edit tool whose view of a just-created
  worktree lagged, and a branch reclaimed by another lab's `uor-hygiene --apply`, which removes clean worktrees
  sitting at `origin/main`), and my first splice ate a closing brace. The work was redone in one uninterrupted
  pass and the brace was caught by the compiler.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

