# #769 — feat(#653): typed cost/scope/resource/witness schema for the #602-606 seam

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T18:28:33Z
- **Updated:** 2026-08-17T19:07:12Z
- **Closed:** 2026-08-17T18:50:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/769
- **Merged:** 2026-08-17T18:50:00Z
- **Merge commit:** 4fb8709320939724d090a143642aa18a09ad98f1
- **Base / Head:** main / UOR-Foundation:i653-route-cost
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Part of #653 phase-3: "Apply the typed plan/scope/resource/cost/witness
pattern to at least one active pipeline seam." This applies it to the
route-attention fit-ladder certification path -- the #602-606 seam
`docs/gnaf_integration_653.md` names as the target.

New module `crates/uor-r4-graph-certify/src/route_cost.rs` adopts the
*shape* of `proofs/wasm-gemm-gnaf/WasmGemmGnaf/Cost/` (static /
dynamic-sum / dynamic-max cost coordinates, a declared first-order
weight objective, a declared resource bound, a composed witness)
applied to the one quantity this seam already measures in closed form:
`RouteOpCensus`, #604's per-step operation count. It deliberately does
NOT port GNAF's `ProperObjective`/sublevel-finiteness machinery (no
Lean global-optimality theorem exists at this seam to discharge) --
see the module doc for the full "what's adopted vs. not" rationale.

### Identity discipline (#600 pattern)

`RouteCostObjective` and `RouteResourceBounds` each carry a pinned
`canonical_bytes()` line format, a `blake3:<hex>` declared-identity
digest, and a versioned registry that refuses an unknown `(id,
version)` by name rather than guessing -- the same pattern
`AttentionOperatorSpec`/`TargetOperatorCertificateSpec` already use.
The v1 resource bound is derived from the format's own
`ROUTE_MAX_CANDIDATES`/`ROUTE_MAX_TOP_M` caps and
`expected_route_census`'s closed form, scaled by a declared step
ceiling -- not an arbitrary number.

### Vocabulary: mirrored, not imported

`RouteCostWitness` reports GNAF SS12.4/SS12.5-shaped
`ExecutionStatus`/`OptimizationStatus`, matching `uor_r4_naf::claims`
variant-for-variant -- but mirrored locally rather than depending on
`uor-r4-naf`. `uor-r4-naf`'s own `Cargo.toml` declares it "never a
dependency of a shipped r4 crate," and `uor-r4-graph-certify` IS
transitively shipped (root `r4` binary -> `uor-r4-proof-model` ->
`uor-r4-graph-certify`). An earlier draft of this branch did add that
dependency edge; it was caught by tracing the actual Cargo dependency
graph and reverted before landing, in favor of the local mirror.

### Wiring

`route_fit_report.rs`'s `HeadEvidence` gains a `cost: RouteCostVector`
field, accumulated in `packed_selection_evidence` from the deployed
packed kernel's own per-step census (the same census the existing
closed-form/witness-replay checks already verify agreement against --
not a separate measurement). `aggregate_runtime` combines every head's
cost and attaches a `compute_route_cost_witness` result to a new
`RuntimeChecks.cost: Option<RouteCostWitness>` field.

The witness is **additive and informational only**:
`RuntimeChecks.cost` is `#[serde(default, skip_serializing_if =
"Option::is_none")]`, so it doesn't disturb any record built before
this field existed, and it deliberately does **not** gate
`target_operator_certificate::derive_overall_quality`. Folding cost
compliance into that derivation would be a quality-rule change, which
the certificate schema's own #600 discipline requires to be a new
registry version (`TARGET_OPERATOR_CERTIFICATE_VERSION` bump), never
an in-place edit of v1 -- that promotion is a deliberate,
separately-reviewed follow-up. `RuntimeBoundsRow.checks` already
embeds `RuntimeChecks` wholesale, so the new field threads through
into the certificate automatically; `target_operator_certificate.rs`
itself is untouched.

## Remaining #653 scope (not this PR)

Per `docs/gnaf_integration_653.md`'s integration matrix: the 4
Lean-toolchain `Tools/*.py` scripts (blocked on CI policy), the
`gen_conformance.py` xtask port (deferred behind #767 to avoid a
rebase conflict, now unblocked), Atlas incremental-rebuild discipline
in xtask, and the CI policy decision on Lean workflows. This PR does
not close #653.

## Test plan

- [x] `cargo test -p uor-r4-graph-certify --offline` -- 39/39 lib tests
      pass (7 new `route_cost` tests)
- [x] `cargo fmt --check` -- clean
- [x] `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` -- clean
- [x] `cargo test --workspace --offline` -- 1869 passed, 0 failed


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

