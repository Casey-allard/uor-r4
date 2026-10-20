# #2354 — The staleness check was lying; version now reads at runtime (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:48:17Z
- **Updated:** 2026-10-13T17:51:06Z
- **Closed:** 2026-10-13T17:50:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2354
- **Merged:** 2026-10-13T17:50:50Z
- **Merge commit:** 969f1a5dee4ea888b00274bbc341d10513e0bfc2
- **Base / Head:** main / UOR-Foundation:deepseek/version-buildfix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Decision: **KEEP** (tooling). Fixes a defect I shipped in #2341.

## The warning I added fired at me, and it was wrong

```
uor-harness: this uor-harness was built from ec3da4f98, which is 9 commit(s) behind origin/main
```

**The binary was current — I had just rebuilt it.** The warning sent me to rebuild something already right. **That is worse than having no warning**, because the next correct warning gets ignored.

## The bug, and why a passing test missed it

`build.rs` baked the build commit in and watched `../../.git/HEAD`. **`.git/HEAD` contains `ref: refs/heads/main` — a ref *name*, not a commit.** A fast-forward never modifies it, so the script never re-ran and the binary kept reporting the commit it was **first** built at. In a linked worktree that path does not exist at all, because the git dir is elsewhere.

**The test passed anyway, because it asserted the value was *recorded*, not that it was *current*.**

## Two fixes tried, both measured, both failed

1. **`rerun-if-changed=build.rs`** — cargo compiles a build script and **caches its OUTPUT**, re-executing it only when a declared input changes. Nothing declared had. Same staleness, one indirection deeper.
2. **Removing every directive** (cargo's default) — re-executes when the *package* changes, which a commit touching only `docs/` does not.

Measured each time against a scratch `main` with two commits: **the reported commit followed neither.**

## The fix

The commit is read **at runtime, from the checkout being inspected**. There is nothing cached, so there is nothing to go stale. `build.rs` is now 20 lines and records **only the build time** — a fact about the artefact that cannot change after it is written.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 153 passed (1 new real-repo test)
uor-harness version on the real checkout        -> "current with origin/main" - no false warning
```

Record: `docs/labs/version-staleness-fix-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **The original case is no longer detected.** A binary built from an older commit, sitting in a newer checkout, is not reported. **I traded a false positive for a false negative**, and the false positive was the one I could see.
- **The build time is still baked in**, so a binary built long ago is not reported as old. Only the checkout's position is checked.
- **`stale()` shells out to `git` three times per invocation.** It runs from `heartbeat` every five minutes, so it is not on a hot path — but it is not free.
- **I shipped the broken version in #2341 and reported it as working.** A warning naming a symptom it cannot itself observe is the same defect class as everything else on this line: a claim about behaviour that was never measured.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

