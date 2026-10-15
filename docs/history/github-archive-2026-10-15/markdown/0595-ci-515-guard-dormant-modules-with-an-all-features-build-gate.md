# #595 — ci(#515): guard dormant modules with an --all-features build + gated-test job

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T21:17:51Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-10T21:36:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/595
- **Merged:** 2026-08-10T21:36:57Z
- **Merge commit:** 6323a46f61855794897dfb233e3a44ed5229ab06
- **Base / Head:** main / UOR-Foundation:r5/ci-all-features
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-2 — CI guard so dormant modules can't bit-rot

The preserve-and-gate work compiles each dormant mechanism behind a **default-off** cargo feature, so the shipped default build and the `gates` nextest run (default features) never touch that code. `gates` already type-checks every feature via `clippy --all-targets --all-features`, but it never **runs** the gated tests and does no codegen/link of the feature-on build.

New **`all-features`** job:
- `cargo build --workspace --all-features` — real codegen + link of every gated module.
- `cargo test --workspace --all-features` — executes the gated dormant-mechanism tests that default nextest skips.

Runs only on `merge_group` / `push` (not `pull_request`), in **parallel** with `gates` in the queue — whose binding verdict is bounded by the heavier κ-reproduction / trend ladder — so it adds no wall-clock to the author's wait. A same-named `pull_request` stub keeps the check resolving in both contexts (mirrors `wasm-pr-stub` / `fuzz-smoke-pr-stub`, #240).

Promoting it to a required status check is a repo-admin branch-protection toggle.

Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

