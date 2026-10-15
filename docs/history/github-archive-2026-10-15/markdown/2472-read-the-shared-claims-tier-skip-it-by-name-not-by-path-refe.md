# #2472 — Read the shared claims tier; skip it by name, not by path (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T14:27:55Z
- **Updated:** 2026-10-14T14:46:38Z
- **Closed:** 2026-10-14T14:46:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2472
- **Merged:** 2026-10-14T14:46:17Z
- **Merge commit:** 23bd6128011e6903c62b9c966154e2f9da33a792
- **Base / Head:** main / UOR-Foundation:deepseek/claims-tier
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What was actually wrong (PM queue item 6)

**The write path already respected R10 and is unchanged.** Measured from a throwaway worktree, `team claim`
in both implementations writes `docs/harness/claims.json` inside that worktree; the owner's checkout is
untouched (modified files before == after == 0).

**The read path was missing a tier.** Python merges live, then `main`'s committed store, then the caller's
own; Rust merged two. Isolated (live tier empty, the worktree's store emptied): Rust reported **no claims**,
and Python reported **the four committed on main**.

**The consequence is the one claims exist to prevent:** a lab in a worktree whose `claims.json` predates a
peer's merged claim cannot see that claim, so it can start work on a branch another lab already holds.

## The bug my fix had, because it is the interesting part

The first version of the three-tier merge skipped with `if *path == shared && shared == local`. That is true
for the **local** entry too when the two paths are equal — so **in the main checkout it skipped the caller's
own store and `load_claims` returned only the live tier**: every committed claim on the machine would have
disappeared. Two existing `multilabrules` tests caught it. A temporary probe printed the deciding line
(`shared==local? true` with `merged = []`), and the fix is to skip **by tier name**, as Python does.

## Verification

| case | result |
|---|---|
| `team claims`, isolated (live empty, store emptied) | **IDENTICAL, 430 bytes** |
| `team claims`, main checkout | **IDENTICAL, 418 bytes** |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 493 passed, 0 failed, 6 ignored (was 491) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0 |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Checklist

- [x] own worktree, fix with tests, record at `docs/labs/claims-shared-tier-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **`main_checkout` is read-only**; R10 forbids writing into the owner's checkout and the write path is
  untouched.
* **It is not `budget::main_worktree`**, which skips the caller and returns `None` when the caller IS the main
  checkout (it is resolving the pod tool). This one must return the caller's own path so the duplicate read
  can be skipped. Two questions, two functions.
* **Found, measured and NOT fixed here:** `multilab claims` has two pre-existing rendering divergences with
  identical data underneath — its heading differs from Python's plain count-and-unit form, and a stale row
  omits Python's ` STALE` suffix. That is the board's renderer, not the claims read path, so it is a separate
  candidate rather than cargo in this PR.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6170709584 — 2026-10-14T14:28:06Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 493 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, byte for byte:**

```
team claims, isolated (live empty, worktree store emptied)   430 bytes  IDENTICAL
team claims, main checkout                                   418 bytes  IDENTICAL
```

The isolated case is the one that exposed the defect: before the fix Rust printed *no branch claims* where
Python printed the four committed on `main`, because Python merges three tiers (live, `main`'s committed
store, the caller's own) and the port merged two.

**The part worth a reviewer's attention is the bug my own fix had.** The first three-tier merge skipped with
`if *path == shared && shared == local` — true for the **local** entry as well when the paths are equal, so in
the main checkout it skipped the caller's own store and `load_claims` returned only the live tier. Every
committed claim on the machine would have vanished. Two existing `multilabrules` tests caught it, and the
temporary probe printed the deciding line: `shared==local? true` with `merged = []`. The fix skips **by tier
name**, as Python does.

**Also measured, and deliberately not in this PR:** `multilab claims` has two pre-existing rendering
divergences with identical data underneath — its heading differs from Python's plain count-and-unit form, and
a stale row omits Python's ` STALE` suffix. Different concern (the board's renderer), so it is a separate
candidate rather than cargo here.


---
_Comment/review ids in this file: 6170709584_

