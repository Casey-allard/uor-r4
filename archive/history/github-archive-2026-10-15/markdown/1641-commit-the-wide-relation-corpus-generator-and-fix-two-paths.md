# #1641 — Commit the wide-relation corpus generator, and fix two paths I broke on main

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T03:27:05Z
- **Updated:** 2026-10-09T13:17:58Z
- **Closed:** 2026-10-03T11:43:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1641
- **Merged:** 2026-10-03T11:43:40Z
- **Merge commit:** 62b242660a45215f3c2e76381a25dd6af7c53ecd
- **Base / Head:** main / UOR-Foundation:codex/wide-relation-corpus
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two independent fixes from this lane, both verified. Reviewable separately.

## 1. The binding result's instrument was not reproducible

This lane's headline result — **14/120 hits on unseen relations vs 0/120 baseline, p~4e-5** — could **not be regenerated from the repository**. The 76-relation table and the manifest fix existed only as an uncommitted change in a worktree. `origin/main`'s generator has 10 relations and emits **1,547,087 tokens** for `rows=30000 seed=17`, against the **1,717,822** the result was measured on. Caught by an independent investigator, not by me.

**A result whose instrument is not reproducible is not a result**, so this commits the instrument.

- Relation table **10 → 76 relations**, **verified disjoint** from the 10 the held-out panel scores. Disjointness is **asserted at generation time** — that assertion caught two collisions (`degree`, `allergy`) that would otherwise have silently made the panel partly in-distribution.
- Manifest gains **`drops` and `files`**. Without them `Split::load` refuses with `Invalid("a source holds literal special-token text")` — an error about **missing metadata** that reads as an error about the **tokens**. `origin/main`'s generator omits both, so **this is also a latent bug fix**: the committed generator produces a store the committed trainer refuses.
- `OVERLAP` kept but emptied (it referenced `colour`/`car`, absent from the wide table).

**Verification — byte-identical:**

```
$ synthetic-memory-corpus out=... tokenizer=... rows=30000 seed=17
30000 documents, 1717822 tokens, 296423 response tokens (17.3%)
regenerated sha: 9bae7202ca4a9455
original    sha: 9bae7202ca4a9455
```

## 2. Two hardcoded paths that PR #1633 (mine) committed onto main

Both no longer exist, so `launch-fulldose.sh` and `post-epoch-eval.sh` **fail before doing anything**.

**Deleted build cache.** `CARGO_TARGET_DIR="$HOME/.cache/uor-r4-opencode-target"` was removed by a storage cleanup. The symptom is:

```
.../geometric-stack: No such file or directory
```

which reads as a missing file rather than a stale build path. I hit it myself and first misdiagnosed it. The scripts now resolve the binary in explicit order — `STACK_BIN`, then `CARGO_TARGET_DIR`, then a prebuilt shared binary, then a build from source into a private cache. **A `STACK_BIN` that is not executable is ignored rather than trusted**, and resolution failure exits non-zero with a clear message instead of proceeding to a confusing downstream error.

**Trashed reports root.** `~/uor-r4-worktrees/stack-prose-reports` was moved to `~/.Trash/uor-cleanup-20261002/`, so the launcher's `init=`/`out=` and `post-epoch-eval`'s `REPORTS` pointed at non-existent paths. Both now default to `~/uor-r4-worktrees/reports` and honour `$REPORTS`.

Verified: `bash -n` on both, plus a resolver test covering the explicit, fallback, and non-executable-`STACK_BIN` cases.

## Scope

No served kernel, no model change, no training change. The distance-curriculum work in flight keeps its control arm on corpus `9bae7202` so `distance` remains the only variable.

References #1512


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

