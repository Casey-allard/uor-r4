# #47 — feat: executable proof model crate uor-r4-proof-model (closes #22)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T01:59:13Z
- **Updated:** 2026-07-22T02:13:51Z
- **Closed:** 2026-07-22T02:13:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/47
- **Merged:** 2026-07-22T02:13:31Z
- **Merge commit:** c1bfcc577b4cd6dca185c90f41686f7eea7280a6
- **Base / Head:** main / UOR-Foundation:feature/issue-22-executable-proof-model
- **Reviews / inline review comments:** 2 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #22: Creates uor-r4-proof-model executable spec crate with zero-allocation, bounded range, canonical top-K, Theorem 7 reverse-index proofs, and a machine-checkable ProofStatusMatrix.

## Timeline (10 entries; chronological: conversation comments, reviews, review comments)

### comment 5040969050 — 2026-07-22T01:59:16Z — Casey-allard

@github-copilot review

### comment 5040969184 — 2026-07-22T01:59:18Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3626900873 — 2026-07-22T02:03:52Z — Copilot

_on crates/uor-r4-proof-model/src/allocation_proof.rs:39_

`verify_zero_allocation` cannot actually detect allocations as written: `ALLOC_COUNT`/`ALLOC_BYTES` are only ever stored/loaded in this module and are never incremented by a counting `#[global_allocator]` (or any other hook). As a result, `diff_count` will always be 0 and the proof will pass even when the closure performs heap allocations.

### review-comment 3626900888 — 2026-07-22T02:03:53Z — Copilot

_on crates/uor-r4-proof-model/tests/proof_model_tests.rs:78_

The zero-allocation test only covers a non-allocating closure, so it would still pass even if `verify_zero_allocation` fails to detect allocations. Add a negative case that performs an allocation (e.g., `Vec::push`) and assert it returns `Err`, so the harness is actually validated.

### review-comment 3626900904 — 2026-07-22T02:03:53Z — Copilot

_on crates/uor-r4-proof-model/src/range_bounds_proof.rs:3_

Doc comment says the range is "strictly bounded" within `total_len`, but the implementation allows `end == total_len` (which is typically correct for a `[start..end)` range). Consider updating the wording to match the actual contract (end <= total_len).

### review-comment 3626900917 — 2026-07-22T02:03:53Z — Copilot

_on crates/uor-r4-proof-model/Cargo.toml:13_

`serde_json` is declared twice and is currently unused in this crate (no references outside Cargo.toml). Keeping unused deps (especially duplicated in both dependencies and dev-dependencies) increases compile time and audit surface; consider removing until it’s actually needed.

### review 4750358256 — 2026-07-22T02:03:54Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a new `uor-r4-proof-model` crate to the workspace to act as an executable/spec-style proof harness around key “transformerless” invariants (range bounds, deterministic top‑K ordering, Theorem 7 reverse index), plus a machine-readable proof status matrix.

**Changes:**
- Introduces `uor-r4-proof-model` library modules for theorem/range/top‑K proofs and a `ProofStatusMatrix`.
- Adds initial unit tests exercising the proof modules.
- Registers the new crate in the workspace (`Cargo.toml`) and lockfile.

### Reviewed changes

Copilot reviewed 9 out of 10 changed files in this pull request and generated 5 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-proof-model/tests/proof_model_tests.rs | Adds unit tests for proof matrix, range bounds, deterministic top‑K, Theorem 7 verification, and a zero-allocation harness. |
| crates/uor-r4-proof-model/src/theorem7_proof.rs | Wraps `TransitionGraph::verify_theorem_7()` with a proof-model API and error mapping. |
| crates/uor-r4-proof-model/src/range_bounds_proof.rs | Adds a checked range-bounds verifier for packed section ranges. |
| crates/uor-r4-proof-model/src/proof_matrix.rs | Implements `ProofStatusMatrix` and `ProofStatus` tracking with a simple `verify_all()` gate. |
| crates/uor-r4-proof-model/src/lib.rs | Exposes the crate modules. |
| crates/uor-r4-proof-model/src/deterministic_topk_proof.rs | Implements canonical sorting + verification for deterministic top‑K tie-breaking. |
| crates/uor-r4-proof-model/src/allocation_proof.rs | Adds a “zero allocation” verification API backed by internal counters (currently not wired to actual allocations). |
| crates/uor-r4-proof-model/Cargo.toml | Declares the new crate and its dependencies. |
| Cargo.toml | Adds `crates/uor-r4-proof-model` as a workspace member and default-member. |
| Cargo.lock | Records the new crate in the lockfile. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626900933 — 2026-07-22T02:03:54Z — Copilot

_on crates/uor-r4-proof-model/Cargo.toml:5_

PR description/Issue #22 mention an "executable spec crate", "differential tests", and "Kani proofs", but this crate currently only adds a library (`src/lib.rs` with modules) plus unit tests—there’s no `src/main.rs`/bin target and no Kani harness/config (no `kani` references in the crate). Either add the missing executable/Kani/differential components or adjust the PR description/scope to match what’s implemented.

### review 4750382550 — 2026-07-22T02:10:11Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3626920640 — 2026-07-22T02:10:11Z — Copilot

_on crates/uor-r4-proof-model/src/allocation_proof.rs:39 · in reply to review-comment 3626900873_

Fixed in the latest commit. Added a `CountingAllocator` struct that implements `GlobalAlloc` by wrapping `std::alloc::System` and calling `fetch_add` on `ALLOC_COUNT`/`ALLOC_BYTES` on every allocation. It's registered with `#[global_allocator]`, so the counters are now actually incremented on every heap allocation — `verify_zero_allocation` correctly returns `Err` when the closure allocates (verified by the existing `Vec::push` negative test case).


---
_Comment/review ids in this file: 5040969050, 5040969184, 4750358256, 4750382550, 3626900873, 3626900888, 3626900904, 3626900917, 3626900933, 3626920640_

