# #603 — trace: add deterministic versioned TeacherTrace profiles and content-addressed observation shards

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:00:12Z
- **Updated:** 2026-08-12T15:02:06Z
- **Closed:** 2026-08-12T15:02:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/603
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The repository already has the v2 observation foundation: content-addressed sample IDs, deterministic shard spill/resume, ordered merges, final hidden-state and top-k hooks, and v4 observation records. Open-weight target-operator fitting needs a bounded trace bundle that can preserve richer evidence without building a parallel observation pipeline.

# Current evidence

- Closed #59 and merged #2b1d060 provide `TeacherOracle::hidden_state()`/`top_k()`, shard manifests, crash-safe resume, and deterministic merge.
- `crates/uor-r4-graph-compiler/src/observation.rs:184-250` (`ObservationManifest`) records shard CIDs and optional input CID; it does not identify a trace profile, tokenizer CID, adapter CID, or operator/source configuration as a versioned bundle.
- `crates/uor-r4-graph-compiler/src/observation.rs` stores bounded token/top-8/probability rows; the oracle surface exposes final hidden state only, not per-layer hidden states, Q/K/V, or selected attention support.
- Merged PR #95 added a hidden-state lane and measured it negative for the existing cover compiler (2.8% vs 31.7% Gate C top-1), so this issue must preserve negative lanes rather than silently adopt them.
- Merged PRs #74/#94 established top-8 records and #307 established runtime teacher-parity metrics; neither defines a versioned multi-profile trace bundle.

# Problem

Trace richness, boundedness, profile identity, absence semantics, and source/provenance dependencies are not represented as one stable bundle. New source families would otherwise add ad hoc fields or rerun extraction with incompatible semantics.

# Goal

Extend the existing observation infrastructure with deterministic `TeacherTrace`-equivalent profiles and content-addressed observation shards suitable for compiler experiments and certification.

# Scope

- Define minimal, layer, attention-support, and full bounded profiles.
- Represent token/position, final hidden state, logits/top-k, and optional layer input/output or Q/K/V/support data; absence must remain absence, never zero-filled data.
- Bind source snapshot, corpus, tokenizer, adapter, trace-profile, and operator IDs/CIDs.
- Preserve deterministic content partitioning, canonical merge order, bounded output, crash-safe resume, and explicit profile/schema versions.
- Reuse `ObservationManifest`, `ObservationShardWriter`, `TeacherOracle`, and graph compiler observation types where possible.

# Non-goals

- Do not create a second observation pipeline or change the deployed runtime.
- Do not collect unbounded full-model traces by default.
- Do not claim that richer traces improve quality; profile adoption requires a separate measured fitting issue.

# Dependencies and blockers

- Depends on #597 (source snapshot identity), #599 (adapter conformance), and #602 (operator identity for operator-specific traces).
- The negative hidden-state result from #95 remains a recorded measurement, not a reason to delete the lane.
- #515 owns dormant feature/ledger handling; #531 owns corpus scale; #589 blocks registry/formal upstream integration.

# Proposed implementation shape

Add a versioned profile descriptor and bundle manifest around the existing shard writer/reader. Make each optional trace component length-delimited and profile-declared, preserve old observation ABI readers, and require deterministic canonical serialization before fitting consumes a bundle.

# Acceptance criteria

- [ ] Minimal and final-hidden/top-k profiles can be emitted, resumed, merged, and reloaded with stable bytes.
- [ ] Layer/attention fields are optional, bounded, profile-declared, and absent rather than zero-filled when unavailable.
- [ ] Bundle identity changes when any source/corpus/tokenizer/adapter/profile/operator dependency changes.
- [ ] Completion, shard, and merge metadata make partial/incomplete traces distinguishable from valid traces.
- [ ] A replay test proves shard completion order and worker count do not alter the merged bundle.
- [ ] The existing #59/#95 observation and hidden-state fixtures remain readable.

# Verification

- Unit tests for profile validation, canonical ordering, absent-field semantics, CID dependency checks, resume, and thread-count invariance.
- Deterministic double extraction on a small pinned fixture; report bytes and bounded peak memory.
- Run observation/graph compiler tests, full fmt/clippy/workspace tests, no-std graph-format checks, and the claim-wording checker.

# Compatibility and migration

Add a new trace schema/profile version without changing existing v3/v4 observation records or historical artifacts. New bundles must list their source/adapter/profile CIDs; old manifests remain readable as legacy profiles with missing optional fields.

# Claim status

Definition and implementation with empirical determinism/boundedness criteria. It is an evidence-container capability, not a quality guarantee.


## Comments (4)

### comment 5260314483 — 2026-08-12T00:05:18Z — Casey-allard

Ecosystem-rebase input (#589/#620): shard/bundle CID binding decision recorded on #620 — canonical-JSON manifests addressed via `uor_addr::json::address_blake3` (already a dependency; same 71-byte blake3 κ label format as the rest of the repo, and byte-compatible with kappa-registry's KappaLabel per #624).

### comment 5267339201 — 2026-08-12T13:17:38Z — Casey-allard

## Plan (on the now-complete identity stack: #597 source manifest, #599 conformance, #600 geometry, #601 tokenizer, #602 operator — binding = referencing landed fields, no new identity invented)

1. **Versioned trace-profile declaration** (`uor-r4-teacher-trace/1`): four bounded profiles — `minimal` (token/position, final hidden, logits/top-k — today's surface), `layer` (+ declared layer inputs/outputs), `attention-support` (+ Q/K/V + selected attention support, bounded), `full` (bounded union) — as a typed schema-versioned record with explicit capture bounds. **Absence stays absence**: optional lanes are Option/None, never zero-filled (the issue's hard rule).
2. **Extend, never fork**: profile identity + the five upstream IDs (source-manifest κ, corpus, tokenizer adapter, geometry, attention operator) bound as one versioned bundle record on ObservationManifest (serde(default), the established seam) — reusing ObservationManifest/ObservationShardWriter/TeacherOracle; deterministic content partitioning, canonical merge order, and crash-safe resume preserved and re-asserted by tests.
3. **#95's negative hidden-state lane preserved as a recorded measurement** (2.8% vs 31.7% Gate C top-1) — the layer profile exists without any claim that richer traces improve quality; adoption of any profile for fitting is a separate measured issue (the issue's own non-goal).
4. Bounded-by-default: no unbounded full-model traces; capture bounds declared in the profile record and enforced.

Six gates + full suite before the PR; record on close.

### comment 5268120722 — 2026-08-12T14:24:24Z — Casey-allard

## Delivery record (PR #641 in queue)

Scope delivered on the existing pipeline — no fork: **(1)** four bounded profiles under `uor-r4-teacher-trace/1` (minimal = today's bytes exactly and never recorded; layer incl. the #95 final-hidden lane; attention-support; full), typed + digest-bearing, registry refusing unknown/unbounded declarations. **(2)** Richer lanes as `.trace` sidecars copying the `.prob` pattern (the v4 record is a fixed 88-byte layout — in-record extension would be a byte-format change; sidecar keeps primary shards era-stable): per-shard κ registration, canonical merge, torn-row checks, crash-safe resume tested, pinned row width. **(3)** Absence stays absence: optional lanes are None; unfilled support slots carry an explicit 0xFFFFFFFF marker — never zero-fill (tested: minimal-traced ≡ plain, byte-identical). **(4)** One stable bundle: `identity_bundle_digest()` over all six identity components (#597 κ, input CID, #600 geometry, #601 tokenizer, #602 operator, trace profile) with explicit absent/present markers — absent ≠ empty ≠ zero, order-independent (tested). **(5)** Capture taps #599's forward_capturing and #602's factored per-head weights, bounded to declared indices; traced step bit-identical to plain step. **(6)** #95's negative result (2.8% vs 31.7% Gate C top-1) cited in the profile docs + MODEL_LIFECYCLE as the recorded reason richer profiles are opt-in; adopting ANY profile for fitting requires a separate measured issue. Honest bounds: from-text batched drivers stay minimal-only (BatchedTeacher has no capture surface — recorded follow-up); the 2.8% figure is attributed to #95's record, not independently re-derivable from repo artifacts; no real-checkpoint traced run (fixture absent from env). Six gates green (127 suites). Closes on #641 merge; #604 next.

### comment 5268579788 — 2026-08-12T15:02:05Z — Casey-allard

PR #641 merged — delivered per the record (comment 5268120722). Closing; #604 next (blocker check in progress).


---
_Comment/review ids in this file: 5260314483, 5267339201, 5268120722, 5268579788_

