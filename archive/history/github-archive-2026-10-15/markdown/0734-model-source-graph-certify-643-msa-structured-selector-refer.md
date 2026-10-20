# #734 — model-source/graph-certify/#643: MSA-structured selector (reference + witness)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T16:21:55Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-16T16:42:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/734
- **Merged:** 2026-08-16T16:42:48Z
- **Merge commit:** d039c9df732e5920d5f01c7653c1b4e1a2297cdf
- **Base / Head:** main / UOR-Foundation:issue-643-msa-selector
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Slice 1 of #643: registers a new attention-operator identity, `msa-structured-selector/1`, in the #602/#604 operator-identity registry, grounded in the Modular Structural Arithmetic (MSA) paper (private paper, cleared for publication by Casey 2026-08-16).

- MSA7's **11-Theorem**: role anchors for residues 2, 4, 8 mod 11 → Gen/Med/Man.
- Theorem M4's **11-Cascade Theorem**: the doubling-cascade orbit `(2,4,8,5,10,9,7,3,6,1)` has maximal period 10 in ℤ/11ℤ (Lagrange's theorem).

**Role-extension gap, resolved explicitly by Casey:** the MSA paper only proves role classification for 3 of the 11 residues mod 11 (2, 4, 8). Rather than silently inventing a rule for the remaining 8 nonzero residues, I raised this as a genuine open design question via `AskUserQuestion`. Casey selected **cascade-position mod 3** (2026-08-16): each nonzero residue's role rank is derived from its position in the doubling-cascade orbit, taken mod 3. This is documented in `msa_selector.rs`'s module doc and in the `role_extension_is_cascade_position_mod_3` test as **this operator's own design choice, explicitly not a proven MSA/paper theorem**.

## What's new

`crates/uor-r4-graph-certify/src/msa_selector.rs` (593 lines):
- `classify(candidate_id) -> MsaClassification`: query-independent `role_rank` (0=Gen, 1=Med, 2=Man, 3=Zero-sentinel) + `cascade_position`, both via integer-only table read/compare — no runtime modulo, P-4-legal.
- `MsaSelectorReference`: top-M selection sorted by `(role_rank, cascade_position, index)`, reusing `route_attention`'s (#604) exact saturating-scoreq-aggregation and lowest-index tie-break conventions — deliberately plug-compatible with a future shared A/B harness against `r4-route-attention/1`.
- `MsaSelectorWitness` + `replay_msa_selector_witness`: independent replay verification of a run's recorded selections/aggregate/census without re-running the operator. Returns `Option<String>` (R5 audit-limits sanctioned shape), matching the existing `ReleaseBundleManifest::validate` convention.

`crates/uor-r4-graph-certify/tests/msa_selector_643.rs` (17 tests, all passing): role-anchor/cascade-orbit fidelity to MSA7/Theorem M4, the mod-3 extension's own behavior, registry resolution and digest stability, reference-run determinism and top-M correctness, witness census/replay (tamper/reorder/version-mismatch rejection), and a malformed-instance refusal case. Carries a "Pre-registered exit rule (issue #643 — binding)" module doc per the #626 convention.

`crates/uor-r4-model-source/src/attention.rs`: new `MSA_STRUCTURED_SELECTOR_ID`/`_VERSION` constants, `msa_structured_selector_v1()` builder, and the corresponding `operator_spec()` registry match arm.

## Scope / deferred

This is slice 1: compiler-side reference semantics and a replayable witness only. NOT yet done — tracked as follow-ups on #643:
- Packed R4G1 lowering in `uor-r4-graph-runtime` (the #604 TARGET-operator template's second half).
- The actual A/B evaluation run against `r4-route-attention/1`.

## Test plan

- [x] `cargo fmt --check -p uor-r4-model-source -p uor-r4-graph-certify`
- [x] `cargo clippy -p uor-r4-model-source -p uor-r4-graph-certify --all-targets -- -D warnings`
- [x] `cargo test -p uor-r4-model-source -p uor-r4-graph-certify` — zero regressions across both crates' full existing suites, plus 17/17 new tests passing
- [x] `cargo run -p xtask -- audit-limits` (R5 gate) — clean

References #643.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

