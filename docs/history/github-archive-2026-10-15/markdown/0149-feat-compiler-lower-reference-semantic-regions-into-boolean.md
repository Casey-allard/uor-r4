# #149 — feat(compiler): lower reference semantic regions into Boolean, mask, popcount, and fixed-point programs (#130)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T15:20:22Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-07-24T17:05:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/149
- **Merged:** 2026-07-24T17:05:07Z
- **Merge commit:** b5276ed6a423c214cea0d8122892f154e1921e80
- **Base / Head:** main / UOR-Foundation:issue-130-lower-semantic-regions
- **Reviews / inline review comments:** 4 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #130. Implementation follows PDF §§5, 7, 17 and `docs/formal_vocabulary.md` §6.

## Key Changes
1. **Boolean & Popcount Region Lowering**: Implemented `LoweredBooleanRegion` lowering reference floating-point region center vectors and Hamming radiuses into 64-bit bitmasks, expected signatures, and integer-only predicate evaluation via `XOR` and `POPCOUNT`.
2. **Fixed-Point Q8.8 ScoreQ Quantization**: Implemented `LoweredFixedPointScore` quantizing floating-point transition and emission scores into Q8.8 (`i16`) representation with explicit saturation boundaries (`i16::MIN`..`i16::MAX`).
3. **Traceable Lowering Witness**: Implemented `LoweringWitness` tracking `LoweringWitnessEntry` items to map each runtime table index/bit back to a reference IR CID and record quantization deltas.
4. **Unrepresentable Region Guard**: Implemented explicit `LoweringError::UnrepresentableRegion` for signatures exceeding 64 bits or invalid Hamming bounds.
5. **Cucumber BDD Suite**: Added `features/suites/lower_semantic_regions.feature` with 3 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 28 BDD scenarios passed (90 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (20 entries; chronological: conversation comments, reviews, review comments)

### comment 5071461169 — 2026-07-24T15:20:26Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3646423173 — 2026-07-24T15:25:06Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:84_

`quantize_q88` marks values that round exactly to `i16::MAX` as `saturated: true` because the comparison is `>=`. That makes the `saturated` flag ambiguous (exactly representable vs clamped). Use a strict `>` check so only out-of-range values are considered saturated.

This issue also appears on line 89 of the same file.

### review-comment 3646423218 — 2026-07-24T15:25:06Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:152_

Radius validation currently allows values up to 64 even when the signature is shorter. If `radius_float` exceeds `signature_bits.len()`, the predicate becomes trivially true for all inputs (since Hamming distance is bounded by the signature width), which is likely an invalid lowering that should be rejected as unrepresentable.

### review-comment 3646423254 — 2026-07-24T15:25:06Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:175_

`reference_contribution_id: 101` and `runtime_table_index: 0` are hard-coded, so multiple lowerings would produce indistinguishable IDs and the witness cannot accurately map runtime layout back to reference contributions. These IDs should be passed in or derived from the lowering context (and the witness should record the real table index assigned during packing).

### review 4774554068 — 2026-07-24T15:25:07Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Implements the compiler-side lowering boundary for reference semantic regions and scores, introducing an integer-only (bitmask/XOR/popcount) predicate representation plus Q8.8 fixed-point quantization, and adds BDD coverage to exercise the new lowering behavior.

**Changes:**
- Added `lower_semantic_regions` module with `LoweredBooleanRegion` runtime-evaluable predicates, Q8.8 `LoweredFixedPointScore` quantization, and witness structs/errors.
- Exposed the new lowering module from `uor-r4-graph-compiler`’s `lib.rs`.
- Added a new Cucumber feature suite + step definitions covering boolean-region lowering, score quantization saturation, and unrepresentable region rejection.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| tests/bdd.rs | Adds BDD step definitions and world fields to exercise semantic-region lowering and Q8.8 quantization behavior. |
| features/suites/lower_semantic_regions.feature | Introduces enforced Gherkin scenarios for region lowering, quantization saturation, and rejection of oversized signatures. |
| crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs | New lowering implementation: boolean bitmask/XOR+popcount predicate, Q8.8 quantization, witness types, and unit tests. |
| crates/uor-r4-graph-compiler/src/lib.rs | Exports the new `lower_semantic_regions` module. |




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:89**
* Same boundary issue on the negative side: `scaled <= i16::MIN` treats exactly representable `i16::MIN` as saturated. Use a strict `<` comparison so `saturated` only means clamped.
```
        } else if scaled <= i16::MIN as f32 {
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review 4774683177 — 2026-07-24T15:43:16Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3646525263 — 2026-07-24T15:43:16Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:84 · in reply to review-comment 3646423173_

Fixed in 3d5ad55: `quantize_q88` now uses strict `>`/`<` comparisons so values rounding exactly to `i16::MAX`/`i16::MIN` are not marked saturated.

### review 4774683768 — 2026-07-24T15:43:21Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3646525725 — 2026-07-24T15:43:21Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:152 · in reply to review-comment 3646423218_

Fixed in 3d5ad55: the valid range is now `[0..signature_bits.len()]` instead of a fixed `[0..64]`, so a radius exceeding the signature width is rejected as unrepresentable.

### review-comment 3646526102 — 2026-07-24T15:43:25Z — Copilot

_on crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs:175 · in reply to review-comment 3646423254_

Fixed in 3d5ad55: `lower_region` now takes `reference_contribution_id` and `runtime_table_index` as explicit parameters instead of hard-coding them; call sites updated.

### review 4774684244 — 2026-07-24T15:43:26Z — copilot-swe-agent[bot] — COMMENTED

### comment 5071735633 — 2026-07-24T15:50:03Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071784219 — 2026-07-24T15:55:32Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved in 3f224d5: merged `main` and fixed conflicts in `tests/bdd.rs` by keeping both the Lower Semantic Regions (#130) and Semantic State Space (#124) world fields/step definitions. Full workspace build, clippy, fmt, and test suite (including all 33 BDD scenarios) pass.

### comment 5071907526 — 2026-07-24T16:12:46Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071948970 — 2026-07-24T16:18:29Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071958340 — 2026-07-24T16:19:44Z — Copilot

@Casey-allard Unfortunately I hit an unexpected error while processing your comment. I've automatically reported this to GitHub.

You can ask me to try again later by mentioning me in a new comment.

If you want to contact GitHub about this error, please mention the following identifier so they can better serve you: `6c65a0f4-68b3-4fce-ae5f-ad1863614824`

Sorry for the inconvenience!

<!-- copilot-coding-agent-error: comment-generic-error -->

### comment 5071962869 — 2026-07-24T16:20:23Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5072026430 — 2026-07-24T16:28:36Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Already resolved in 4a6ec14: the branch is merged with `origin/main` (including #128) and has no outstanding conflicts. Full workspace build, clippy, fmt, and test suite pass.

### comment 5072112171 — 2026-07-24T16:38:23Z — Casey-allard

@copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

### comment 5072155785 — 2026-07-24T16:43:29Z — Copilot


> @copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

Resolved in 6b5293e: merged `main` (which had picked up #129) and fixed conflicts in `tests/bdd.rs` by keeping the Lower Semantic Regions (#130), Reference Compiler IR (#129), Behavioral Probes (#128), and Semantic State Space (#124) world fields/step definitions. Full workspace build, clippy, fmt, and test suite pass.


---
_Comment/review ids in this file: 5071461169, 5071735633, 5071784219, 5071907526, 5071948970, 5071958340, 5071962869, 5072026430, 5072112171, 5072155785, 4774554068, 4774683177, 4774683768, 4774684244, 3646423173, 3646423218, 3646423254, 3646525263, 3646525725, 3646526102_

