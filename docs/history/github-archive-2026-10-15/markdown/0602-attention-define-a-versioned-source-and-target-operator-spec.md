# #602 — attention: define a versioned source and target operator specification

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:59:44Z
- **Updated:** 2026-08-12T13:17:14Z
- **Closed:** 2026-08-12T13:17:14Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/602
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The current source teacher has a boolean `r4_attention` switch, while the deployed R4G1 runtime has separate integer/table scoring semantics. The open-weight compiler needs an explicit operator boundary so source-model attention is not silently reinterpreted as a target runtime operator.

# Current evidence

- `crates/uor-r4-model-source/src/lib.rs:677-730` has standard and experimental source-teacher branches; both compute dot-product scores and apply `softmax_with_mode`.
- `crates/uor-r4-model-source/src/lib.rs:1047-1054`, `crates/uor-r4-api/src/compile.rs:131-153`, and `src/main.rs:216-218` expose the boolean/CLI switch rather than a versioned operator description.
- `docs/deferral_record_2026_08_05.md` calls the path “Spin(4)” and “softmax-free”, but #515’s current audit records that those claims do not match the control flow.
- `crates/uor-r4-graph-format/src/inference_contract.rs` and `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md` specify deployed operation constraints, but there is no corresponding explicit source/target attention specification.
- Dormant packed-routing surfaces are being preserved and gated under #515 (PRs #593/#594/#596); this issue must not reactivate them.

# Problem

There is no versioned, inspectable representation of Q/K/V projection, positional action, compatibility relation, selector/normalization, value aggregation, output projection, runtime state, permitted operations, and tie-breaking for source versus target attention.

# Goal

Define an `AttentionSpec`-equivalent repository-native specification that makes source attention and any R4-native target operator distinct, versioned, and auditable.

# Scope

- Inventory the current source-teacher attention operator truthfully, including softmax and remainder-dimension behavior.
- Define versioned fields for projections, positional action, relation, selector/normalization, aggregation, output projection, state, tie-breaking, and permitted operation class.
- Define how an operator ID/version is carried into compiler IR, artifacts, certificates, and reports.
- Provide a standard-attention reference surface used for controlled A/B measurements.
- Correct or link stale “Spin(4)”/“softmax-free” wording after the specification is accepted.

# Non-goals

- Do not implement or promote a new target operator here.
- Do not reuse unchanged Q/K/V weights under an incompatible equation.
- Do not change the deployed runtime contract or feature-gate behavior owned by #515.

# Dependencies and blockers

- Depends on #515’s claim/activation-gate conventions and on #599’s source conformance boundary.
- The target implementation/fitting work is a later follow-up; cross-repository operator integrations remain blocked by #589.

# Proposed implementation shape

Use the existing compiler/report/artifact versioning patterns and the current source/runtime type names where they fit. Keep the specification host-side and serializable; make absence explicit for operators that are not part of an artifact.

# Acceptance criteria

- [ ] A versioned specification distinguishes standard source attention, the current experimental source branch, and any future R4-native target operator.
- [ ] The specification records the current experimental branch’s actual formula and remainder policy without stale claims.
- [ ] Operator ID/version is present in compiler reports and certificates for any operator-specific experiment.
- [ ] A reference standard-attention path has deterministic tie-breaking and fixture coverage.
- [ ] No dormant operator is activated merely by defining the specification.

# Verification

- Serialization round-trip and unknown-version rejection tests.
- Formula/control-flow tests on divisible and non-divisible head widths.
- Documentation claim-wording check and review against `INFERENCE_OPERATION_CONTRACT.md`.
- `cargo fmt --check`, workspace clippy/tests, and graph-format no-std checks.

# Compatibility and migration

The specification is additive. Existing artifacts without an operator record retain a documented legacy interpretation; new operator-bearing artifacts/reports include the ID/version and are not treated as byte-compatible with prior eras.

# Claim status

Definition and implementation with proof obligations for deterministic serialization and explicit operator identity. It is not a model-quality or equivalence claim.


## Comments (3)

### comment 5266383812 — 2026-08-12T11:46:23Z — Casey-allard

## Plan (the #600/#601 versioned-record pattern, which this issue's own implementation-shape section prescribes)

1. **Truthful inventory first**: read the standard + experimental source-teacher branches (model-source lib.rs:677-730) and record the experimental branch's ACTUAL formula — including softmax_with_mode use and remainder-dimension behavior — as the spec's ground truth (no stale 'Spin(4)'/'softmax-free' wording; #515's audit already recorded the mismatch).
2. **Typed `AttentionOperatorSpec`** (host-side, serializable, versioned): projections (Q/K/V/output), positional action, compatibility relation, selector/normalization, value aggregation, runtime state, tie-breaking, permitted operation class — three registered operators: `standard-source-attention/1`, `experimental-source-branch/1` (its real formula), and the explicit absence marker for artifacts carrying no operator. Registry refuses unknown (id,version) via the sanctioned error surface (R5).
3. **Identity carriage**: operator ID/version into compiler reports + observation manifests on the #597/#600/#601 seams (serde(default), legacy bytes unchanged when unset); the boolean `r4_attention` switch maps to the versioned IDs at the boundary without changing CLI behavior.
4. **Reference standard-attention surface** with deterministic tie-breaking + fixture coverage (divisible and non-divisible head widths) for controlled A/B use.
5. **Docs**: correct/link the stale deferral-record wording to the accepted spec; review against INFERENCE_OPERATION_CONTRACT.md. **No dormant operator activation** (#515's gates untouched — PRs #593/#594/#596 surfaces not reactivated); no new target operator implemented.

Six gates + full suite before the PR; record on close.

### comment 5266835244 — 2026-08-12T12:31:14Z — Casey-allard

## Delivery record (PR #640 in queue)

Acceptance: **(1)** Versioned spec distinguishes `standard-source-attention/1`, `experimental-r4-source-attention/1`, and explicit absence (None = documented legacy interpretation); future target operators get new registry entries. **(2)** The experimental branch's ACTUAL formula recorded from code, not lore: chunked-4-wide dot product; **selector/normalization is the same softmax as standard** (max-subtract, exp, sum-normalize — not a bypass); **remainder policy: trailing H mod 4 q/k dims are truncated from scores** (perturbation-tested at H=6), while scale still divides by √(full H) and values aggregate over all H. Stale 'Spin(4)'/'softmax-free' wording corrected — README's false '~25% faster by bypassing softmax exponents' claim removed, CLI help + doc comments fixed, dated correction appended to the deferral record with original text retained as history. **(3)** Operator ID/version carried in ObservationManifest (auto-populated from the oracle) and CoverReport (`--attention-operator` on the #600 seam; api/server bind naturally); certify-side reports have no oracle seam — recorded as an honesty item consistent with #597/#600/#601 rather than plumbing invented. **(4)** Standard + experimental per-head attention factored into free deterministic functions with documented tie-breaking, arithmetic untouched (bit-exact `forward_batch_matches_serial_forward` green), fixtures for divisible + non-divisible head widths. **(5)** No dormant operator activated — #515 gates and #593/#594/#596 surfaces untouched; the boolean switch still only selects. Six gates green (127 suites). Closes on #640 merge; #603 (TeacherTrace profiles + content-addressed shards) next.

### comment 5267334928 — 2026-08-12T13:17:14Z — Casey-allard

PR #640 merged — delivered per the record (comment 5266835244). Closing; #603 begins now on the fully-landed identity stack (#597/#600/#601/#602).


---
_Comment/review ids in this file: 5266383812, 5266835244, 5267334928_

