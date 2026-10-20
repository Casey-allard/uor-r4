# #148 — feat(compiler): introduce reference floating-point semantic compiler and intermediate representation (#129)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T15:07:05Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-07-24T16:34:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/148
- **Merged:** 2026-07-24T16:34:21Z
- **Merge commit:** d1691002a18c6d6c4f5b568981790e6f83f2001e
- **Base / Head:** main / UOR-Foundation:issue-129-reference-compiler-ir
- **Reviews / inline review comments:** 4 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #129. Implementation follows PDF §§7, 11, 17 and `docs/formal_vocabulary.md` §5.

## Key Changes
1. **Reference Graph Intermediate Representation (`ReferenceGraphIr`)**: Implemented explicit IR containing observations, semantic states, regions, transition scores, emission distributions, and provenance metrics.
2. **5-Stage Reference Pipeline (`ReferenceCompilerPipeline`)**: Implemented standalone compilation pipeline: Stage 1 (Teacher Probing) -> Stage 2 (Region Induction) -> Stage 3 (Transition Discovery) -> Stage 4 (Objective Optimization) -> Stage 5 (Lowering Preparation).
3. **Standalone Inference & Differential Harness**: Enabled direct state transition ($T(s, a) \to S$) and emission prediction queries without loading packed R4G1 binary artifacts. Added `DifferentialCompilerHarness` for baseline evaluation comparison.
4. **Cucumber BDD Suite**: Added `features/suites/reference_compiler_ir.feature` with 3 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 28 BDD scenarios passed (89 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (15 entries; chronological: conversation comments, reviews, review comments)

### comment 5071334177 — 2026-07-24T15:07:10Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3646386083 — 2026-07-24T15:18:49Z — Copilot

_on crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:158_

`transition` selects the best transition via `partial_cmp(...).unwrap()`, which will panic if any transition score is NaN (since `partial_cmp` returns `None`). This makes reference-IR inference brittle and can crash tests/CLI if a score becomes non-finite during optimization.

### review 4774505810 — 2026-07-24T15:18:50Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces a new *reference* floating-point “semantic compiler” surface in `uor-r4-graph-compiler`, including a `ReferenceGraphIr` intermediate representation, a 5-stage compilation pipeline, and a cucumber/BDD suite that exercises deterministic compilation, transition queries, emission predictions, and a basic differential comparison harness.

**Changes:**
- Added `crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs` with IR types, a 5-stage pipeline API, and a differential harness.
- Registered the new module in `crates/uor-r4-graph-compiler/src/lib.rs`.
- Added BDD coverage via `features/suites/reference_compiler_ir.feature` and new step implementations in `tests/bdd.rs`.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| `crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs` | New reference IR + pipeline + harness implementation and unit tests. |
| `crates/uor-r4-graph-compiler/src/lib.rs` | Exposes the new `reference_compiler_ir` module. |
| `tests/bdd.rs` | Adds cucumber world fields and step definitions exercising the new reference pipeline/IR. |
| `features/suites/reference_compiler_ir.feature` | Adds 3 BDD scenarios covering compile determinism/CIDs, transitions+emissions, and differential comparison. |




<details>
<summary>Comments suppressed due to low confidence (2)</summary>

**crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:301**
* After switching CID generation away from `simple_cid`, this helper becomes unused and will trigger `dead_code` warnings under `-D warnings`. It should be removed (or replaced with the new CID implementation if you prefer keeping a helper).
```
fn simple_cid(input: &str) -> u32 {
    let mut h = 0x811c9dc5u32;
    for b in input.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}
```
**crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:317**
* This unit test asserts the CID starts with `cid_`. If the IR is moved to the repo-standard `blake3:<hex>` CID format (as used elsewhere in the graph compiler), this assertion should be updated accordingly.
```
        assert!(ir.provenance.content_cid.starts_with("cid_"));
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646386108 — 2026-07-24T15:18:50Z — Copilot

_on crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:260_

The content CID is currently computed from `corpus.join("\n")` and a 32-bit non-cryptographic hash (`simple_cid`). This can collide (e.g., `["a", "b"]` vs `["a\nb"]`) and is inconsistent with the repo’s existing content-addressing (blake3 CIDs are used throughout the graph compiler/certifier). Consider hashing a structured encoding (count + length-prefixed entries) with blake3 and emitting a `blake3:<hex>` CID; also populate `compiler_version` from `config.version` so the config is actually reflected in provenance.

This issue also appears in the following locations of the same file:
- line 294
- line 317

### review-comment 3646386131 — 2026-07-24T15:18:50Z — Copilot

_on tests/bdd.rs:507_

This BDD step asserts `content_cid` starts with `cid_`. If the reference IR adopts the repo’s standard `blake3:<hex>` CID format, update this expectation to match so the scenario continues to validate the intended behavior (stable, content-addressed CIDs).

### review 4774550427 — 2026-07-24T15:24:33Z — copilot-swe-agent[bot] — COMMENTED

### review 4774550433 — 2026-07-24T15:24:33Z — copilot-swe-agent[bot] — COMMENTED

### review 4774550437 — 2026-07-24T15:24:33Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3646420325 — 2026-07-24T15:24:33Z — Copilot

_on crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:158 · in reply to review-comment 3646386083_

Fixed in 12742ad: replaced `partial_cmp(...).unwrap()` with `total_cmp`, which is NaN-safe and won't panic.

### review-comment 3646420326 — 2026-07-24T15:24:33Z — Copilot

_on tests/bdd.rs:507 · in reply to review-comment 3646386131_

Updated in 12742ad to assert `content_cid.starts_with("blake3:")`.

### review-comment 3646420332 — 2026-07-24T15:24:33Z — Copilot

_on crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs:260 · in reply to review-comment 3646386108_

Fixed in 12742ad: content CID is now blake3 over a structured, length-prefixed encoding of the corpus (count + entry length + bytes), emitted as `blake3:<hex>`. `compiler_version` is now populated from `config.version`.

### comment 5071606814 — 2026-07-24T15:36:29Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071653217 — 2026-07-24T15:41:22Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved in 4d9dc11: merged `origin/main` and fixed the conflict in `tests/bdd.rs` by keeping both the Reference Compiler IR (#129) and Semantic State Space (#124) world fields and BDD step definitions. Verified with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, and `cargo test --test bdd` (33 scenarios / 107 steps) — all pass.

### comment 5071843924 — 2026-07-24T16:03:41Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071887031 — 2026-07-24T16:09:52Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved in df5338a: merged the latest `origin/main` (which now also includes PR #128's behavioral probes) and fixed the conflict in `tests/bdd.rs` by keeping all three BDD step blocks/world fields side by side (Reference Compiler IR #129, Behavioral Probes #128, Semantic State Space #124). Verified with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, and `cargo test --test bdd` (36 scenarios / 117 steps) — all pass.


---
_Comment/review ids in this file: 5071334177, 5071606814, 5071653217, 5071843924, 5071887031, 4774505810, 4774550427, 4774550433, 4774550437, 3646386083, 3646386108, 3646386131, 3646420325, 3646420326, 3646420332_

