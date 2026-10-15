# #942 — runtime/S3 re-entry: carry a complete generated-prefix signature through normative R4G1 serving

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T20:32:07Z
- **Updated:** 2026-08-25T20:54:25Z
- **Closed:** 2026-08-25T20:53:50Z
- **Labels:** enhancement
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/942
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824

## Problem

The strict production generator in `src/tless_uor.rs` retains only the newest `WINDOW` tokens and calls `NormativeServingEngine::predict`. After the initial prompt is truncated, information outside that window cannot affect later token decisions. This is the exact representation boundary behind the S3 `LIMIT` verdict: the deployed artifact was memoryless beyond its suffix.

The normative runtime already supports a secondary session-signature lane, but the strict production loop does not derive or update one from its admitted token trajectory.

## Goal

Carry the complete admitted prompt plus committed generated prefix into every normative production decision through a deterministic, fixed-size incremental token-history signature.

## First implementation slice

- Refactor the existing token-history signature into a fixed-size incremental state whose output is byte-identical to `session_signature_from_tokens` for every prefix.
- Seed it once from the exact tokenized prompt.
- Call `NormativeServingEngine::predict_with_session_signature` on every production step.
- Update it only after a token is actually served and committed; EOS, abstention, and decline do not enter state.
- Retain the existing bounded `WINDOW` context lane alongside the full-trajectory lane.

## Acceptance

- Two trajectories with the same newest `WINDOW` tokens but different earlier tokens produce different session signatures.
- Incremental and batch signatures agree at every prefix.
- The production loop uses the rolling signature without per-step allocation.
- Existing old-artifact and fail-closed admission behavior is unchanged.
- Focused router and production-path tests pass; ordinary fast CI remains the merge gate.

## Claim boundary

This establishes deployed cross-step memory reachability. It does not claim coherent generation or clear the frozen #841 generation bar. Broader measurement is authorized only after the product-path canary works.

## Comments (1)

### comment 5416645454 — 2026-08-25T20:54:24Z — Casey-allard

Delivered through protected PR #943 and merged to main as b8f78786eb8f7697b4419742008375a58fe8767b.

Definition-of-done evidence:
- The router now maintains a fixed-size incremental signature over the complete admitted prompt plus every committed generated token; no prefix retention, rescanning, or per-step allocation is required.
- The normative production generator passes that full-prefix signature into every R4G1 serving decision while retaining the existing fixed runtime token window.
- A focused regression demonstrates that two trajectories with the same newest runtime window but different earlier prefixes remain distinguishable.
- Abstention, decline, and EOS do not mutate the trajectory state.

Focused local gates:
- cargo fmt --check
- python3 scripts/check_claim_wording.py
- cargo test -p uor-r4-router session_signature --offline
- cargo test --lib production_trajectory_keeps_prefix_memory_beyond_runtime_window --offline
- cargo test -p uor-r4-graph-runtime --test r4g1_runtime_test session_signature_enters_rout_fallback_as_secondary_probe --offline
- cargo test --lib complete_production_envelope_installs_and_corpus_tamper_is_atomic --offline
- cargo check -p uor-r4-wasm-router --all-targets --offline

Protected evidence:
- PR gate: https://github.com/UOR-Foundation/uor-r4/actions/runs/32896656968 — PASS in 6m17s
- Merge-group gate: https://github.com/UOR-Foundation/uor-r4/actions/runs/32897342159 — PASS in 6m01s

Claim boundary: this establishes cross-step full-prefix memory reachability in the deployed path. It does not establish coherent free-running generation or clear the frozen #841 acceptance bar; tracker #824 remains open for that measured result.


---
_Comment/review ids in this file: 5416645454_

