# #39 — Transformerless runtime: bounded top‑M memberships per depth with nearest-prefix fallback

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-21T23:11:10Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-07-21T23:46:05Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/39
- **Merged:** 2026-07-21T23:46:05Z
- **Merge commit:** 889213b7f82a44a522fd55e9d58d1c35d51cb9ee
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundationuor-r4-16
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This updates transformerless class assignment from single nearest-class selection to bounded multi-membership per depth. It preserves nearest-class prefixes as a guaranteed fallback floor at every depth.

- **Runtime assignment semantics**
  - Added `assign_memberships_plain(...)` to compute:
    - nearest class code (`[u8; STAGES]`) for compatibility/backoff,
    - bounded top‑M membership sets per depth (`M=3`).
  - Kept `assign_plain(...)` as the compatibility entrypoint by delegating to the new multi-membership path and returning top‑1.

- **Store evidence write path**
  - Added `add_evidence_multi(...)` to write evidence across bounded memberships at each depth.
  - Enforced nearest-prefix fallback when a depth has no explicit membership candidates.
  - Updated `build_store(...)` to use multi-membership outputs while still returning nearest codes for existing callers.

- **Focused behavior coverage**
  - Added tests asserting:
    - per-depth membership bound is respected,
    - nearest-prefix fallback is always present,
    - evidence insertion falls back correctly when membership sets are missing.

```rust
let (nearest, by_depth) = runtime::assign_memberships_plain(&art, &sig);

// nearest-class fallback floor must exist at each depth
for depth in 1..=STAGES {
    assert!(by_depth[depth]
        .iter()
        .any(|key| key.as_slice() == &nearest[..depth]));
}
```

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3626359238 — 2026-07-21T23:23:31Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime.rs:677_

`add_evidence` now allocates a temporary `Vec<Vec<u8>>` on every call via `&[vec![Vec::new()]]`. This is avoidable overhead (especially if `add_evidence` is used in an online indexing/hot path). You can pass an empty slice for `by_depth` and rely on the fallback behavior for depths > 0.

### review 4749756417 — 2026-07-21T23:23:32Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR extends the transformerless runtime’s class assignment from single nearest-class selection to bounded multi-membership (top‑M, currently 3) per depth, while preserving nearest-prefix membership as a guaranteed fallback at every depth. It also updates the store-building evidence insertion path to write evidence across these bounded memberships.

**Changes:**
- Added `assign_memberships_plain(...)` to compute both nearest code (compat/backoff) and bounded memberships per depth.
- Added `add_evidence_multi(...)` and updated `build_store(...)` to insert evidence across bounded memberships, with nearest-prefix fallback when memberships are missing.
- Added focused tests validating membership bounds, nearest-prefix fallback behavior, and evidence insertion fallback.

### Reviewed changes

Copilot reviewed 2 out of 2 changed files in this pull request and generated 2 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/runtime.rs | Introduces bounded per-depth memberships, adds multi-evidence insertion, and wires store construction to use memberships while keeping nearest-code compatibility. |
| crates/uor-r4-core/tests/window_paths.rs | Adds tests for bounded memberships, nearest-prefix fallback guarantees, and fallback evidence insertion when membership sets are missing. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626359254 — 2026-07-21T23:23:32Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime.rs:718_

`build_store` stores `memberships` for every corpus element, but it is only used during store construction and then immediately dropped. This can be a large, unnecessary memory spike (O(n·STAGES·M·depth) allocations). You can build `store` in the same pass that computes `(code, by_depth)` and still return `codes`, avoiding the extra `memberships` vector entirely.


---
_Comment/review ids in this file: 4749756417, 3626359238, 3626359254_

