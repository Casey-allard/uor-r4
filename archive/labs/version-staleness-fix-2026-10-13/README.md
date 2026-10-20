# The staleness check was measuring the wrong thing, and it lied to me

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** item 6 of Claude's first list (#2028 comment 6149175296), which I shipped broken

## What happened

The warning I added in #2341 fired at me:

```
uor-harness: this uor-harness was built from ec3da4f98, which is 9 commit(s) behind origin/main
```

**It was wrong.** The binary was current; I had just rebuilt it. The warning sent me to rebuild
something that was already correct — which is worse than having no warning, because the next correct
warning gets ignored.

## The bug, and why it survived a passing test

`build.rs` baked the build commit into the binary and watched `../../.git/HEAD` for changes.
**`.git/HEAD` contains `ref: refs/heads/main` — a ref NAME, not a commit.** A fast-forward never
modifies it, so the script never re-ran and the binary kept reporting the commit it was first built
at. In a linked worktree that path does not exist at all, because the git dir is elsewhere.

**The test passed anyway**, because it asserted the value was *recorded*, not that it was *current*.

## Two fixes tried, both measured against a real repository and both failed

1. `rerun-if-changed=build.rs` — **cargo compiles a build script and caches its OUTPUT**, re-running
   only when a declared input changes. Nothing declared had. Same staleness, one indirection deeper.
2. Removing every directive (cargo's default) — re-runs when the *package* changes, which a commit
   touching only `docs/` does not. Still stale.

Measured each time with two commits on a scratch `main`: **the reported commit followed neither.**

## The fix: read it at runtime

`crates/uor-r4-harness/src/version.rs` no longer bakes the commit in. It reads the **checkout's** HEAD
and `origin/main` when asked. There is nothing cached, so there is nothing to go stale.

`build.rs` is now 20 lines and records **only the build time** — a fact about the artefact that cannot
change after it is written.

## The consequence, stated because it is a real loss

**This reports whether the checkout is behind `origin/main`, not whether the binary predates the
checkout.** A binary built from an older commit, sitting in a newer checkout, is **no longer detected
— and that was the original case.** What remains detectable is the common one: a lab that has not
pulled or rebuilt, whose binary then rejects a flag the docs describe.

**The new test is the one the old design never had:** a scratch repository with two commits, asserting
that a checkout at the first is `Behind` by one, at the second is `Current`, and on a branch ahead is
`Unknown` rather than `Behind`. It fails for the baked-in design.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **153 passed** (1 new real-repo test) |
| `uor-harness version` on the real checkout | `current with origin/main` — **no false warning** |
| real-repo branch advance | reported commit follows (both wrong designs failed this) |

## Limitations, stated

- **The original case is no longer detected.** A stale binary in a current checkout passes the check.
  I traded a false positive for a false negative, and the false positive was the one I could see.
- **The build time is still baked in**, so a binary built long ago is not reported as old. Only the
  checkout's position is checked.
- **`stale()` shells out to `git` three times per call.** Called from `heartbeat`, which runs every
  five minutes, so it is not on a hot path — but it is not free either.
- **I shipped the broken version in #2341 and reported it as working.** The warning naming a symptom
  it could not itself observe is the same defect class as the rest of this line: a claim about
  behaviour that was never measured.
