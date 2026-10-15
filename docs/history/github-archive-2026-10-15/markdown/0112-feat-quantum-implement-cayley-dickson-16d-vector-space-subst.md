# #112 — feat(quantum): implement Cayley-Dickson 16D vector space substrate and nested inclusions (#104)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T16:01:58Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-07-23T17:20:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/112
- **Merged:** 2026-07-23T17:20:03Z
- **Merge commit:** 6b4eb21dd56d2fcc9a4a3380fc401403c4d38acf
- **Base / Head:** main / UOR-Foundation:issue-104-cayley-dickson-substrate
- **Reviews / inline review comments:** 7 / 12
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #104

This PR implements Phase 1 of the Quantum Geometric Transformerless Engine master roadmap (#103), based on N. Furey (2026) 'Standard Model Symmetries and the Nested Embeddings of R ⊂ C ⊂ H ⊂ O' (arXiv:2607.18450v1).

### Implementation Details
- **Cayley-Dickson Vector Space V**: Implemented 16D real / 8D complex vector space  = e_i \mathbb{O} \oplus e_5 \mathbb{H} \oplus e_6 \mathbb{C} \oplus e_7 \mathbb{R} \oplus \mathbb{R}$ in `cd_space.rs`.
- **Division Algebra Structs**: Added `Octonion`, `Quaternion`, and `ComplexNumber` structs with explicit Fano plane multiplication tensors ({ijk} = +1$).
- **Nested Embeddings**: Implemented `embed` ($\iota_V$) and projection canonical forms for the nested inclusion chain:
  50993\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O} \subset V50993

### Verification & Quality Gates
- **Workspace Test Suite**: `cargo test --workspace --offline` passed **47/47** test suites cleanly.
- **Clippy Code Quality**: `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` passed with **0 warnings**.
- **Formatter Check**: `cargo fmt --check` passed cleanly.

## Timeline (25 entries; chronological: conversation comments, reviews, review comments)

### comment 5060607190 — 2026-07-23T16:02:01Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5060633388 — 2026-07-23T16:04:41Z — Casey-allard

@copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

### review-comment 3639623310 — 2026-07-23T16:10:16Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:1000_

`MarginWeighted` computes `scaled` as `i128` but then casts with `scaled as i32`, which will wrap/truncate on overflow and can silently corrupt candidate scores. Clamp to the `i32` range (or use a checked conversion) before constructing `ScoreQ`.

This issue also appears on line 1649 of the same file.

### review-comment 3639623345 — 2026-07-23T16:10:16Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1145_

The integer-core source scan now treats `BEGIN/END EXPERIMENTAL VARIANT` regions as if they were the compiler-side float block, which means `score_runtime.rs` can contain `*`/`/`/`%` and/or floats without the test enforcing the invariant. This weakens the machine-checked “integer-only core” guarantee; consider moving experimental arithmetic into a different file/module (so the scan remains strict) or enforcing the same operator restrictions inside the experimental regions too.

### review-comment 3639623370 — 2026-07-23T16:10:17Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1343_

`rule12_interaction_residuals` is currently computed as an alias of `rule12_precedence_no_f` (`hits_interaction` increments with `rule12_no_f_hit` and `interaction_bits` is set to `rule12_no_f_bits`). That makes the metric misleading and can mask regressions if readers assume it reflects interaction-residual logic from issue #69.

### review-comment 3639623408 — 2026-07-23T16:10:17Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:278_

`ScoreConfig`/CLI now carry `scoring_variant`, but `evaluate_gate_c` ignores `config.scoring_variant` for the primary Rule 1 / Rule 1+2 scorers (it only hard-codes extra scorers for normalized/margin). As written, `--scoring-variant` doesn’t affect the main Gate C metrics, which is surprising for users and makes the config field easy to misuse.

### review-comment 3639623436 — 2026-07-23T16:10:17Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:624_

New scoring variants (`CloudSizeNormalized`, `MarginWeighted`) are exposed via CLI and used in Gate C evaluation, but there are no unit tests covering their behavior (e.g., that they keep candidate ordering stable under known fixtures and don’t overflow/produce invalid `ScoreQ`). Adding variant-specific tests would help prevent silent metric drift.

### review-comment 3639623471 — 2026-07-23T16:10:17Z — Copilot

_on crates/uor-r4-core/src/transformerless/cd_space.rs:224_

`CayleyDicksonVector::embed` hard-codes the final scalar component (`v[15]`) to 0.0, so the embedding cannot represent the full `… ⊕ ℝ` term described in the module/docs and can’t round-trip via projections. If the scalar component is meant to be part of the Phase-1 substrate, `embed` should accept it (and a corresponding projection accessor should exist).

### review 4765997091 — 2026-07-23T16:10:18Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces an initial “quantum geometric” substrate into `uor-r4-core` by adding a Cayley–Dickson-inspired vector-space module (octonion/quaternion/complex components with embedding/projection helpers), plus accompanying tests and roadmap documentation. It also extends the transformerless scoring/evaluation pipeline with new (currently experimental) candidate scoring variants and additional Gate C reporting fields/CLI options.

**Changes:**
- Added `transformerless::cd_space` with Octonion/Quaternion/ComplexNumber types and a 16D `CayleyDicksonVector`, plus unit tests for multiplication properties and embedding/projection.
- Added `ScoringVariant` plumbing (runtime enum, CLI flag, score config field) and Gate C evaluation outputs for variant comparisons.
- Added new model metadata JSON and a Phase roadmap document describing the broader “quantum geometric transformerless” plan.

### Reviewed changes

Copilot reviewed 9 out of 9 changed files in this pull request and generated 6 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| models/smollm2-360m-instruct.json | Adds model metadata (repo/revision/shape/paths) for SmolLM2-360M-Instruct. |
| docs/r4_furey_quantum_geometric_plan.md | Adds master architecture/roadmap document for the “quantum geometric transformerless” direction. |
| crates/uor-r4-core/tests/score.rs | Updates the integer-core source scan to skip newly introduced experimental-variant blocks. |
| crates/uor-r4-core/tests/cd_space.rs | Adds tests for octonion/quaternion/complex operations and Cayley–Dickson embedding/projections. |
| crates/uor-r4-core/src/transformerless/score.rs | Wires in `ScoringVariant` config and expands Gate C outcome reporting with new metric fields. |
| crates/uor-r4-core/src/transformerless/score_runtime.rs | Introduces `ScoringVariant` and applies variant-dependent residual accumulation logic (experimental blocks). |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new `cd_space` module publicly. |
| crates/uor-r4-core/src/transformerless/command.rs | Adds `--scoring-variant` flag and threads it into `ScoreConfig`. |
| crates/uor-r4-core/src/transformerless/cd_space.rs | Implements the Cayley–Dickson vector-space substrate and nested projections. |
</details>





<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-core/src/transformerless/score_runtime.rs:1654**
* Same overflow risk in `step_accumulate_node`: `scaled as i32` can wrap/truncate, producing incorrect residual accumulation for `MarginWeighted`. Clamp (or checked-convert) the `i128` intermediate before `ScoreQ::from_raw`.
```
                            .map(|a| u32::from(self.regions[a.region as usize].radius) as i128)
                            .unwrap_or(1)
                            .max(1);
                        let scaled = (value.raw() as i128 * margin) / radius;
                        ScoreQ::from_raw(scaled as i32)
                        // END EXPERIMENTAL VARIANT
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review 4766167909 — 2026-07-23T16:31:29Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639758398 — 2026-07-23T16:31:29Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:1000 · in reply to review-comment 3639623310_

Fixed via the upstream main merge (cc42540): the margin-weighted scaling now saturates the i128 intermediate into the i32 range instead of truncating with `as i32` (see `score_variant.rs::saturate_i32`).

### review 4766168578 — 2026-07-23T16:31:35Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639758937 — 2026-07-23T16:31:35Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1145 · in reply to review-comment 3639623345_

Fixed via the upstream main merge (cc42540): the experimental variant arithmetic was moved into its own `score_variant.rs` module, so `score_runtime.rs` no longer needs the `BEGIN/END EXPERIMENTAL VARIANT` exemption and the integer-only source scan stays strict.

### review 4766169341 — 2026-07-23T16:31:42Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639759480 — 2026-07-23T16:31:42Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1343 · in reply to review-comment 3639623370_

Fixed via the upstream main merge (cc42540): the `rule12_interaction_residuals` metric (which just aliased `rule12_precedence_no_f`) was removed.

### review 4766170167 — 2026-07-23T16:31:49Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639760143 — 2026-07-23T16:31:49Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:278 · in reply to review-comment 3639623408_

Resolved via the upstream main merge (cc42540): the `--scoring-variant` CLI flag and `ScoreConfig::scoring_variant` field were removed entirely — the cloud-size-normalized/margin-weighted scorers in `evaluate_gate_c` are now internal, fixed comparison scorers only, so there's no config field that can silently fail to apply.

### review 4766170874 — 2026-07-23T16:31:56Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639760767 — 2026-07-23T16:31:56Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:624 · in reply to review-comment 3639623436_

Fixed via the upstream main merge (cc42540): `score_variant.rs` now has unit tests for both variants covering the normal scaling behavior and overflow saturation.

### review 4766171692 — 2026-07-23T16:32:03Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639761383 — 2026-07-23T16:32:03Z — Copilot

_on crates/uor-r4-core/src/transformerless/cd_space.rs:224 · in reply to review-comment 3639623471_

Fixed in 91e1f8c: `embed` now takes a `scalar: f32` parameter for the trailing `⊕ ℝ` term instead of hard-coding it to 0.0, and a new `project_scalar` accessor lets it round-trip.

### comment 5060937490 — 2026-07-23T16:36:33Z — Casey-allard

@copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

### comment 5061000716 — 2026-07-23T16:43:38Z — Copilot

_edited 2026-07-23T16:43:43Z_


> @copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

Merge conflicts resolved in ef6aa16. Conflicts were in `mod.rs`/`score.rs`/`score_runtime.rs`/`tests/score.rs` from PR #114 (which reverted the experimental scoring variants added in #102) — adopted main's reverted state there and removed `score_variant.rs`. Re-added the `cd_space` module declaration that the merge had dropped. All four quality gates pass (fmt, test, clippy, no_std ladder).


> @copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

Merge conflicts resolved in ef6aa16 — see my reply above for details.

### comment 5061089880 — 2026-07-23T16:53:45Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5061141523 — 2026-07-23T16:59:28Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved in f060eea. Main moved forward with PR #113 (#105 Endomorphism algebra), which added its own `cd_space.rs`/`tests/cd_space.rs` based on an earlier revision predating the `embed(scalar)`/`project_scalar()` fix — add/add conflicts. Kept this branch's version and updated `tests/endomorphism.rs` to pass the new scalar argument. All four quality gates pass (fmt, test, clippy, no_std ladder).


---
_Comment/review ids in this file: 5060607190, 5060633388, 5060937490, 5061000716, 5061089880, 5061141523, 4765997091, 4766167909, 4766168578, 4766169341, 4766170167, 4766170874, 4766171692, 3639623310, 3639623345, 3639623370, 3639623408, 3639623436, 3639623471, 3639758398, 3639758937, 3639759480, 3639760143, 3639760767, 3639761383_

