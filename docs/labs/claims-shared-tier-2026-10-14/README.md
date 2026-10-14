# The claims read path was missing its shared tier

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** PM queue item 6 — the claims store

## What was actually wrong

**The write path already respected R10, and this unit did not change it.** Measured from a throwaway
worktree, `team claim` in BOTH implementations writes `docs/harness/claims.json` inside that worktree and
leaves the owner's checkout untouched (modified files before == after == 0).

**The read path was missing a tier.** Python's `load_claims` merges three - live, then `main`'s committed
store (`shared_claims_file`), then the caller's own store - while the Rust port merged two. Isolated
(live tier pointed at an empty directory, the worktree's own store emptied to `{}`):

```
Rust:    no branch claims recorded
Python:  branch claims (4)   <- read from main's committed store
```

**The consequence is the one claims exist to prevent:** a lab in a worktree whose `claims.json` predates a
peer's merged claim cannot see that claim, so it can start work on a branch another lab already holds.

## The fix, and the bug the fix had

`shared_claims_file` now resolves the **main checkout** (`git worktree list --porcelain` names the primary
checkout first; git failing falls back to the caller, which makes the tier equal and therefore skipped), and
`load_claims` merges live -> shared -> caller.

**My first version of that merge was wrong, in a way that would have been much worse than the bug it fixed.**
The skip was written `if *path == shared && shared == local { continue; }` - which is true for the LOCAL
entry as well when the two paths are equal, so **in the main checkout it skipped the caller's own store and
`load_claims` returned only the live tier**: every committed claim on the machine would have disappeared.
Two existing `multilabrules` tests caught it (`a_claim_with_no_lab_is_an_error`,
`a_long_claim_with_no_heartbeat_is_stale`, both reporting zero findings). A temporary probe test printed the
deciding line - `shared==local? true` with `merged = []` - and the fix is to skip **by tier name**, as
Python does:

```rust
for (tier, path) in [("live", &live), ("shared", &shared), ("local", &local)] {
    if tier == "shared" && shared == local { continue; }
```

## Verified

| case | result |
|---|---|
| `team claims`, isolated (live empty, worktree store emptied) | **IDENTICAL, 430 bytes** |
| `team claims`, main checkout | **IDENTICAL, 418 bytes** |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **493 passed, 0 failed, 6 ignored** (was 491: 2 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0 |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

The two new tests are the regression guards: **a real linked worktree whose own store is empty must still
see the claim committed on main**, and **the caller's own store must still be read when it is also the
shared path** (the naming bug).

## Found and measured, NOT fixed here

**`multilab claims` has two pre-existing rendering divergences** with the same data underneath (the tier fix
makes it read the same four claims as Python):

* the heading is `branch claims (4) · current branch main` where Python writes `4 branch claim(s)`;
* a stale row omits Python's ` STALE` suffix.

They are a different concern - the board's renderer, not the claims read path - so they stay out of this PR
rather than ride along in it.

## Stated rather than hidden

* **`main_checkout` is read-only.** R10 forbids writing into the owner's checkout; this only ever reads the
  path it resolves, and the write path is untouched.
* **It is not `budget::main_worktree`.** That one skips the caller and returns `None` when the caller IS the
  main checkout, because it is looking for the pod tool; this one must return the caller's own path so the
  duplicate read can be skipped. Two questions, two functions.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 7 (the stale `~/.cargo/bin/uor-harness` install), then unread-note surfacing. The
`multilab claims` rendering is recorded above as a separate candidate.
