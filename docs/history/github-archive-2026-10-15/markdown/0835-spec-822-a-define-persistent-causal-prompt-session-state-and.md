# #835 — spec/#822-A: define persistent causal prompt/session state and its compiler-runtime boundary

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:35:23Z
- **Updated:** 2026-08-20T16:58:13Z
- **Closed:** 2026-08-20T16:56:45Z
- **Labels:** enhancement
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/835
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #822

## Context

#784 showed that distinct full-depth codes can still converge on the same marginal continuation. Existing suffix/NGRAM and ExactContext evidence does not provide a persistent whole-prompt state across generated tokens.

## Problem

“Bidirectional anchors” is not yet a causal, bounded, packable semantics. Without a reference state transition, candidate/support effects, capacity/eviction rules, and witness fields, experiments cannot distinguish prompt information from suffix leakage or decoder noise.

## Goal

Specify a minimal persistent prompt/session state with independent reference semantics and an explicit offline-learning versus deployed-integer boundary.

## Scope

- [ ] Define bounded lanes for local suffix, prompt segment, role/turn, entity/topic, session history, and explicit constraints; justify each lane by a measurable intervention.
- [ ] Define the initial fold over the complete supplied prompt and the per-generated-token transition without reading future completion tokens.
- [ ] Define how state changes candidate support, evidence weighting, and score contributions independently of decoding.
- [ ] Separate compiler-learned quantities from deployed fixed-point/table quantities and enumerate every allowed hot-path operation.
- [ ] Specify capacity, deterministic eviction/decay, overflow/decline, saturation, and tie-breaking semantics.
- [ ] Specify state reset/continuation rules for native requests, OpenAI-compatible chat, streaming, and supported WASM hosts.
- [ ] Define a simple executable reference model and witness fields sufficient to replay every state transition and contribution.
- [ ] Record R4G1 optional-section/versioning choices, byte layout constraints, limits, and old-artifact behavior.
- [ ] Project artifact bytes, table reads, operations/token, and caller-owned state before fitting implementation-specific arms.

## Non-goals

- Access future target/completion tokens at inference.
- Treat random sampling, repetition penalties, or prompt-template injection as persistent semantic state.
- Commit to W(3,3) or any geometry before the downstream bake-off.

## Dependencies and entry gate

Requires #831 normative scorer ownership, #832 causal suite/report conventions, and the admitted #833 baseline before the spec is frozen. Drafting may proceed in parallel.

## Acceptance criteria

- [ ] The reference semantics determine the same state and score contributions from the same prompt, artifact, and prior state.
- [ ] Every lane has a causal benchmark intervention and a bounded deployed representation.
- [ ] Absent sections exactly preserve historical behavior; invalid capacities/indices fail with typed errors.
- [ ] The specification includes byte/operation/state budgets and explicit witness replay semantics.
- [ ] No deployed transition uses multiply, divide, floating point, heap allocation, clock, network, or RNG.

## Verification

- [ ] Reference-model unit/property tests for determinism, capacity edges, saturation, reset, and tie breaks.
- [ ] Causality tests: prompt meaning interventions change the intended state; paraphrases remain within the frozen stability criterion.
- [ ] Prompt-shuffle and constant-state planted negatives prove the benchmark detects non-causal implementations.
- [ ] Format/reference examples round-trip and old artifacts remain behaviorally identical when the section is absent.
- [ ] Claim-wording and conformance marker tests cover every normative addition.

## Compatibility and migration

Prefer a versioned optional R4G1 section and caller-owned state extension. Old artifacts and clients without the section retain exact behavior. Any request-state ABI change needs versioned negotiation and a migration/recompile note.

## Repository conformance

- **Execution scope:** Reference specification and proposed offline compiler plus bounded deployed R4Engine state semantics. Evidence outside this scope is not credited as serving.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes, bounded state/work, typed errors, `no_std`, and witness obligations.
- Normative equations/statements follow `docs/formal_vocabulary.md`; empirical criteria include distribution, protocol, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-01, RF-21, RF-22, RF-27, RF-28.. Extend existing capability suites unless the implementation creates a genuinely new capability.
- Built capability order is `model/ids.toml` → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Dormant/unselected arms remain ledgered with activation or retirement gates. Generated conformance is never hand-edited.
- Claim-changing measurements append a per-issue record and reconcile all current summary assertions.

## Claim status and next action

Completion freezes an experimentable contract; it does not establish that persistent state improves prompt causality.

## Comments (1)

### comment 5359084558 — 2026-08-20T16:58:12Z — Casey-allard

## Definition of Done — #835 (spec/#822-A) COMPLETED

Merged via PR #872, merge commit `8785aedd717e8746f501bb265799efc886b61446` on `main` (merge_group full ladder: success, single clean pass).

**Delivered**
- `docs/prompt_state_spec_835.md` — the frozen S1 experimentable contract: six bounded lanes each justified by a causal intervention; a whole-prompt initial fold Φ₀ and a causal per-token transition T_ps that reads no future token; a decode-independent `ScoreQ` contribution lowering over the existing residual taxonomy; capacity/eviction/decay/saturation/tie-break with typed errors; an optional versioned R4G1 `PSTATE` section whose absence preserves the baseline exactly; byte/table-read/operation/state budgets; the replayable witness schema; and the frozen `s1-causal-prompt-pairs` benchmark (primary metric `causal-influence-delta`) with prompt-swap, suffix-only, and shuffled/constant-state falsifiers.
- `crates/uor-r4-api/tests/prompt_state_spec_835.rs` — the executable reference model and machine-checked evidence (11 tests): determinism, bounded fixed-capacity state, causal (no-future-token) transition, decode-independent contributions, capacity/eviction, canonical tie-break, saturation, reset/continuation, witness replay, typed errors, and planted negatives (prompt-swap / suffix-only / constant-state) with an anti-vacuity guard proving the benchmark separates a causal from a non-causal arm. Binds the frozen #832 control vocabulary (`ControlKind`, `MetricStatus`, `is_degenerate_control`).
- `docs/RESEARCH.md` — reconciled with the S1-A note.

**Acceptance criteria** — all five met: reference determinism (same prompt/artifact/prior state ⇒ same state + contributions); every lane has a causal benchmark intervention and a bounded deployed representation; absent-section baseline preservation + typed errors on invalid capacities/indices; byte/operation/state budgets + explicit witness-replay semantics; and the deployed no-multiply/divide/float/heap/clock/RNG contract enumerated (the deployed-path allocation/P-4 proof is discharged by #836).

**Verification** — `cargo fmt --check`; `cargo clippy -p uor-r4-api --all-targets --all-features -D warnings`; `cargo test -p uor-r4-api --test prompt_state_spec_835` (11/11); `python3 scripts/check_claim_wording.py`; `cargo run -p xtask -- validate` (R1 check-model 29 ids, R4 audit-deferral, R5 audit-limits, gnaf firewall/root/scan/conformance). The merge queue re-verified the full ladder.

**Scope / conformance** — reference-only / off-serving-path, mapping to and extending the evidence of existing RF-01/21/22/27/28. No new `model/ids.toml` capability row and no `CONFORMANCE.md` regeneration (the #832 precedent for a spec/infrastructure leaf); no artifact-byte or runtime-behavior change.

**Claim status / next action** — this freezes an experimentable contract; it does **not** establish that persistent state improves prompt causality. Next: **#834** fits prompt-conditioned evidence arms against these causal and equal-budget controls; the deployed lowering and certification are **#836**.



---
_Comment/review ids in this file: 5359084558_

