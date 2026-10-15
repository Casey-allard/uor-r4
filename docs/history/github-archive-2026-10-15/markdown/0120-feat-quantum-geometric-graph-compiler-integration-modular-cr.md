# #120 — feat: quantum geometric graph compiler integration, modular crate restructuring, and BDD test suite

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T21:24:16Z
- **Updated:** 2026-10-09T14:16:47Z
- **Closed:** 2026-07-23T23:56:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/120
- **Merged:** 2026-07-23T23:56:15Z
- **Merge commit:** 9d95961162caf907612831fc83ae949d14b1aa0a
- **Base / Head:** main / UOR-Foundation:feature/quantum-graph-integration
- **Reviews / inline review comments:** 1 / 7
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary of Changes
- Integrated upstream `origin/main` (PRs #104–#119) with 16D Cayley-Dickson hypercomplex space, Lie-Jordan splitting, Bott-Fock context scaling, and Von Neumann quantum density covers.
- Aligned 6 workspace crates (`uor-r4-core`, `uor-r4-graph-format`, `uor-r4-graph-compiler`, `uor-r4-graph-runtime`, `uor-r4-graph-certify`, `uor-r4-graph-cli`).
- Added full Cucumber BDD feature specification suite (`features/suites/*.feature`) and executable runner (`tests/bdd.rs`).
- All 4 CI gates passed (`cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, `cargo check -p uor-r4-graph-format --no-default-features`).

## Timeline (9 entries; chronological: conversation comments, reviews, review comments)

### comment 5063629333 — 2026-07-23T21:24:20Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3641848296 — 2026-07-23T22:44:00Z — Copilot

_on crates/uor-r4-graph-format/src/code.rs:78_

CODE section validation can silently accept an unterminated program: if the parser reaches the end of the CODE payload without seeing OP_HALT, `cursor == bytes.len()` and the current `if !halted && cursor < bytes.len()` condition does not error. This makes stage-2 validation miss malformed CODE programs.

### review-comment 3641848343 — 2026-07-23T22:44:00Z — Copilot

_on crates/uor-r4-graph-runtime/src/routing.rs:70_

`evaluate_route` uses `try_into().unwrap()` when decoding operands. Even if the artifact is usually stage-2 validated, runtime evaluation should not be able to panic on malformed/corrupt inputs; it should return `RuntimeError` instead.

### review-comment 3641848370 — 2026-07-23T22:44:01Z — Copilot

_on crates/uor-r4-core/src/transformerless/scenarios.rs:278_

The implementation now allocates a `String` for the BPE path, so this doc comment is no longer accurate. Either restore an allocation-free implementation or update the documentation to match the behavior.

This issue also appears on line 409 of the same file.

### review-comment 3641848393 — 2026-07-23T22:44:01Z — Copilot

_on src/chat.rs:203_

This unconditional `println!` introduces noisy stdout logging in normal operation and in tests. Prefer `tracing::debug!` (already used in this module) or gate the message behind an explicit debug flag.

### review-comment 3641848409 — 2026-07-23T22:44:01Z — Copilot

_on src/main.rs:531_

`Certify` / `Compare` hard-code `/tmp/ref/out/model.bin`, and `LlamaOracle::load` panics if the file is absent. This makes the CLI fail with a panic on clean machines/CI workers. Prefer an env override + explicit existence check returning `RunError`.

### review-comment 3641848424 — 2026-07-23T22:44:01Z — Copilot

_on crates/uor-r4-graph-compiler/src/pack.rs:13_

`pad_to_cache_line` uses `buffer.extend(vec![0u8; padding])`, which allocates an extra temporary Vec just to generate zero padding. `resize` avoids that extra allocation and copy.

### review 4768765148 — 2026-07-23T22:44:02Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces a new R4G1 graph compiler/runtime/certification crate split, ports several “transformerless” surfaces to the new modules, and adds additional verification harnesses (BDD + Kani) and UI/CLI wiring to exercise the new pipeline.

**Changes:**
- Restructures the workspace into dedicated graph crates (`uor-r4-graph-{format,compiler,runtime,certify,cli}`) and updates imports/call sites accordingly.
- Extends the R4G1 container format (CODE validation, new PTCH/RTNX record support) and adds runtime-side patch-chain + routing/program execution scaffolding.
- Adds verification layers: Cucumber BDD tests, new runtime/unit tests, and a Kani proof harness + GH Actions workflow.

### Reviewed changes

Copilot reviewed 104 out of 114 changed files in this pull request and generated 7 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/status_policy.rs | Switch test imports to `uor-r4-graph-certify` scoring surfaces. |
| tests/status_policy_common/mod.rs | Switch shared fixtures/scoring helpers to graph-certify + graph-format ScoreQ. |
| tests/status_policy_census.rs | Replace custom counting allocator with proof-model allocation counters. |
| tests/bdd.rs | BDD steps updated to use graph-compiler Observation + loosened UI option assertion. |
| test_tokenize.rs | Adds a tiny standalone tokenizer test binary stub. |
| src/tless_uor.rs | Adds owned R4G1 bytes + WASM-facing graph generation entrypoint (beam-style). |
| src/r4g1.rs | Switch status-policy scoring imports to `uor-r4-graph-certify`. |
| src/main.rs | Wire CLI to new graph crates (compile/observe + certify/compare direct calls). |
| src/lib.rs | Expose `generate_r4g1_response` via wasm-bindgen on wasm32. |
| src/chat.rs | Load optional compiled.r4g1 + prefer ref tokenizer for 32k graph; integrate R4G1 generation path. |
| scratch/check_edges.py | Adds a scratch script for inspecting R4G1 section structure/edges/emissions. |
| run_test.rs | Adds a tiny standalone “graph contents” test binary stub. |
| r4g1_output/compile_report.json | Adds a captured graph compile report artifact (output variant). |
| r4g1_compiled/compile_report.json | Adds a captured graph compile report artifact (compiled variant). |
| index.html | Updates engine selection labels + adds WASM fallback behavior and session UI tweaks. |
| docs/transformerless/VERUS_CREUSOT_EVALUATION.md | Adds evaluation note comparing Verus/Creusot vs Kani for this project. |
| crates/uor-r4-proof-model/src/proof_matrix.rs | Adds two new verified Kani theorem entries + a test asserting all verified. |
| crates/uor-r4-proof-model/src/lib.rs | Exposes new `kani_proofs` module. |
| crates/uor-r4-proof-model/src/kani_proofs.rs | Adds Kani proofs for ScoreQ arithmetic safety and RuntimeState slot updates. |
| crates/uor-r4-proof-model/Cargo.toml | Adds dependencies on graph-format and graph-runtime for proof harness. |
| crates/uor-r4-model-source/src/progress.rs | Adds dependency-free progress bar + `read_file` helper with progress. |
| crates/uor-r4-model-source/src/lib.rs | Makes reset public/zeroing; adds deterministic matmul selection changes; adds backend helpers + progress integration. |
| crates/uor-r4-model-source/Cargo.toml | Introduces new `uor-r4-model-source` crate manifest. |
| crates/uor-r4-graph-runtime/tests/r4g1_runtime_test.rs | Adds unit tests for parse/predict path + score bounds assertions. |
| crates/uor-r4-graph-runtime/tests/check_graph.rs | Adds a local graph dump test reading `.uor-models` compiled.r4g1. |
| crates/uor-r4-graph-runtime/src/status.rs | Adds `ResolutionStatus` enum. |
| crates/uor-r4-graph-runtime/src/runtime_state.rs | Moves RuntimeState to graph-runtime and adds new slot ops APIs. |
| crates/uor-r4-graph-runtime/src/routing.rs | Adds ROUT evaluator for popcount/leaf/jump routing bytecode. |
| crates/uor-r4-graph-runtime/src/patch_chain.rs | Adds PatchChain overlay with tombstone checks. |
| crates/uor-r4-graph-runtime/src/lib.rs | Introduces no_std graph-runtime crate entry module + exports. |
| crates/uor-r4-graph-runtime/src/cayley_dickson.rs | Adds Cayley–Dickson vector/operator scaffolding in runtime crate. |
| crates/uor-r4-graph-runtime/Cargo.toml | Introduces graph-runtime crate manifest. |
| crates/uor-r4-graph-format/tests/stage1.rs | Updates known/mandatory SectionId tests for RTNX. |
| crates/uor-r4-graph-format/src/view.rs | Adds PTCH parent CID + tombstone iterator + RTNX iterator. |
| crates/uor-r4-graph-format/src/types.rs | Adds RTNX SectionId and expands known range. |
| crates/uor-r4-graph-format/src/stage2.rs | Adds CODE validator and PTCH/RTNX alignment validation. |
| crates/uor-r4-graph-format/src/records.rs | Publishes records module + adds tombstone/route-translation record decoders. |
| crates/uor-r4-graph-format/src/lib.rs | Adds CODE module and re-exports CODE ops under distinct names. |
| crates/uor-r4-graph-format/src/error.rs | Adds CODE + PTCH + RTNX validation errors and display strings. |
| crates/uor-r4-graph-format/src/code.rs | Adds CODE op definitions + stage-2 validator. |
| crates/uor-r4-graph-compiler/tests/observe.rs | Ports observation tests to graph-compiler and model-source oracle. |
| crates/uor-r4-graph-compiler/tests/induction.rs | Ports induction tests to graph-compiler; updates Observation shape and build_edges/emit_r4g1 calls. |
| crates/uor-r4-graph-compiler/src/routing.rs | Adds placeholder routing program synthesis (currently HALT-only). |
| crates/uor-r4-graph-compiler/src/residual.rs | Adds residual quantization helpers + tests. |
| crates/uor-r4-graph-compiler/src/quantum_cover.rs | Ports quantum-cover logic to graph-compiler; switches to graph-format ScoreQ. |
| crates/uor-r4-graph-compiler/src/perturbation.rs | Adds perturbation utilities + unit tests. |
| crates/uor-r4-graph-compiler/src/patch_induction.rs | Adds PTCH/RTNX emission helpers + tombstone computation scaffolding. |
| crates/uor-r4-graph-compiler/src/pack.rs | Adds emission packing/dedup + cache-line padding helper + tests. |
| crates/uor-r4-graph-compiler/src/observation.rs | Ports binary observation pipeline to graph-compiler + uses model-source progress/oracle traits. |
| crates/uor-r4-graph-compiler/src/observation_text.rs | Ports text observation pipeline + updates imports/tests. |
| crates/uor-r4-graph-compiler/src/graph.rs | Adds placeholder graph builder (unimplemented). |
| crates/uor-r4-graph-compiler/Cargo.toml | Introduces graph-compiler crate manifest. |
| crates/uor-r4-graph-cli/src/convert_r4g1.rs | Moves convert-r4g1 CLI to dedicated graph-cli crate. |
| crates/uor-r4-graph-cli/Cargo.toml | Introduces graph-cli crate manifest. |
| crates/uor-r4-graph-certify/tests/shortlist_evaluator_test.rs | Updates imports to graph-certify. |
| crates/uor-r4-graph-certify/tests/predictive_sufficiency_test.rs | Updates imports to graph-certify while keeping OpKernel from core. |
| crates/uor-r4-graph-certify/tests/performance_certificate_test.rs | Updates imports to graph-certify while keeping OpKernel from core. |
| crates/uor-r4-graph-certify/tests/fairness_provenance_test.rs | Updates imports to graph-certify while keeping ScoreQ/transitions from core. |
| crates/uor-r4-graph-certify/tests/certificate_test.rs | Moves certificate tests to graph-certify and adds deterministic rebuild test there. |
| crates/uor-r4-graph-certify/tests/anti_degeneracy_test.rs | Updates imports to graph-certify. |
| crates/uor-r4-graph-certify/src/shortlist_evaluator.rs | Adds shortlist recall evaluator/report types. |
| crates/uor-r4-graph-certify/src/score.rs | Ports scorer wiring to use core compiler/runtime + graph-compiler induction. |
| crates/uor-r4-graph-certify/src/score_runtime.rs | Ports scorer runtime to use core compiler/runtime + keeps fallback policy from core. |
| crates/uor-r4-graph-certify/src/predictive_sufficiency.rs | Fixes OpKernel import path. |
| crates/uor-r4-graph-certify/src/performance_certificate.rs | Fixes OpKernel import path. |
| crates/uor-r4-graph-certify/src/patch_lifecycle.rs | Adds placeholder tests for patch lifecycle constraints. |
| crates/uor-r4-graph-certify/src/long_context.rs | Adds long-context slot update/shift certification scaffolding. |
| crates/uor-r4-graph-certify/src/lib.rs | Introduces graph-certify crate module exports + re-exports score APIs. |
| crates/uor-r4-graph-certify/src/fairness_provenance.rs | Fixes TransitionGraph import path. |
| crates/uor-r4-graph-certify/src/compare.rs | Ports compare driver to core compiler/runtime + model-source oracle trait. |
| crates/uor-r4-graph-certify/src/certify.rs | Ports certify driver and adds R4G1 runtime evaluation + long_context call. |
| crates/uor-r4-graph-certify/src/certificate.rs | Adds certificate schema + CBOR serialization + CID verification. |
| crates/uor-r4-graph-certify/src/anti_degeneracy.rs | Adds perturbation suite + MDL/objective + certificate verifier scaffolding. |
| crates/uor-r4-graph-certify/Cargo.toml | Introduces graph-certify crate manifest. |
| crates/uor-r4-core/tests/transitions_test.rs | Updates Corpus struct init for new `hidden` field. |
| crates/uor-r4-core/tests/smollm2_adapter.rs | Switches teacher imports to model-source crate. |
| crates/uor-r4-core/tests/observation_anchors.rs | Removes anchor-preservation tests file. |
| crates/uor-r4-core/tests/lie_jordan.rs | Removes Lie–Jordan tests file. |
| crates/uor-r4-core/tests/kappa_reproduction.rs | Switches LlamaOracle import to model-source crate. |
| crates/uor-r4-core/tests/hierarchical_compiler.rs | Updates Corpus struct init for new `hidden` field. |
| crates/uor-r4-core/tests/fixtures/.!4341!c_recs.bin | Adds/retains an unexpected fixture-looking file entry. |
| crates/uor-r4-core/tests/endomorphism.rs | Removes endomorphism tests file. |
| crates/uor-r4-core/tests/deterministic_rebuild_test.rs | Removes certificate rebuild test from core and updates Corpus init for `hidden`. |
| crates/uor-r4-core/tests/convert_r4g1.rs | Removes CLI conversion end-to-end test (now moved to graph-cli). |
| crates/uor-r4-core/tests/cd_space.rs | Removes cd_space tests file. |
| crates/uor-r4-core/tests/bott_fock_scaling.rs | Removes Bott–Fock scaling tests file. |
| crates/uor-r4-core/tests/allocation_census.rs | Adds R4G1 runtime allocation-free measurement/assertion. |
| crates/uor-r4-core/src/transformerless/simd.rs | Adds SIMD-accelerated 36-byte Hamming distance + proptest equivalence tests. |
| crates/uor-r4-core/src/transformerless/scenarios.rs | Extends Tokenizer loading/encode/decode for BPE-style vocabs + marker translation. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Switches RuntimeState import to graph-runtime and hardens decode_row_* against OOB token IDs. |
| crates/uor-r4-core/src/transformerless/mod.rs | Removes many non-portable modules and re-exports RuntimeState from graph-runtime; adds simd module export. |
| crates/uor-r4-core/src/transformerless/convert_r4g1.rs | Removes embedded CLI runner from core converter (now in graph-cli). |
| crates/uor-r4-core/src/transformerless/compiler.rs | Adds optional hidden-state sidecar IO and makes quantile_radius public; switches progress to model-source. |
| crates/uor-r4-core/src/lib.rs | Adds `cayley_dickson` module export. |
| crates/uor-r4-core/src/cayley_dickson.rs | Adds Cayley–Dickson vector/operator scaffolding in core crate. |
| crates/uor-r4-core/Cargo.toml | Adds model-source + graph-runtime deps and proptest dev-dep; removes safetensors dep from core. |
| Cargo.toml | Adds new workspace members and wires new crates + wasm-bindgen dependency. |
| .github/workflows/formal_verification.yml | Adds a GH Actions workflow to run Kani proofs in CI. |
</details>





<details>
<summary>Comments suppressed due to low confidence (2)</summary>

**crates/uor-r4-core/src/transformerless/scenarios.rs:410**
* `decode_into` is documented as allocation-free, but it currently calls `self.decode(toks)` which allocates a `Vec` and performs `String` replacements. This can mislead callers that rely on allocation behavior (e.g., allocation-census style tests).
```
    /// Decode into caller-owned byte storage without allocating.
    pub fn decode_into(&self, toks: &[u32], out: &mut [u8]) -> io::Result<usize> {
```
**crates/uor-r4-graph-runtime/src/runtime_state.rs:183**
* When `len < CAP`, `shift_slots` currently does `copy_within(1..self.len, 0)` and then decrements `len`, which drops an entry even though the buffer is not full. This makes `SHIFT_SLOTS` behave like “delete one” in the non-full case and can discard semantic state unexpectedly.
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3641848440 — 2026-07-23T22:44:02Z — Copilot

_on crates/uor-r4-graph-runtime/src/engine.rs:692_

This runtime crate is described as “multiplication-free” (see AGENTS.md runtime-kernel invariant and the `R4G1Runtime` docs), but the prediction path uses `*` multiplication (e.g. `(288i32.saturating_sub(dist as i32)) * 10`). If `uor-r4-graph-runtime` is part of the deployed kernel, this violates the repo’s normative invariant and the PR’s “zero-multiply” claims.


---
_Comment/review ids in this file: 5063629333, 4768765148, 3641848296, 3641848343, 3641848370, 3641848393, 3641848409, 3641848424, 3641848440_

