# #604 — attention: specify and implement R4RouteAttentionV1 reference semantics and packed runtime lowering

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:00:49Z
- **Updated:** 2026-08-12T17:14:08Z
- **Closed:** 2026-08-12T17:14:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/604
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The source-teacher `r4_attention` path is not an R4-native operator: it is chunked scalar dot-product attention followed by softmax. The deployed R4G1 runtime does have bounded Boolean/ROUT and integer/table routing primitives, but the dormant packed-kernel surface is not a compiled teacher-replacement operator.

# Current evidence

- #515 and PRs #593/#594/#596 preserve and gate dormant mechanisms; `packed-routing-dormant` remains explicitly unactivated.
- `crates/uor-r4-graph-runtime/src/packed_kernels.rs:90-175` documents placeholder ROUT/transition kernels, including fixed return values and TODOs; this is not evidence of a working target attention operator.
- `crates/uor-r4-graph-runtime/src/routing.rs` and `crates/uor-r4-graph-format/src/rout.rs` provide an existing bounded ROUT bytecode/reference surface that should be reused where applicable.
- `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md` and `crates/uor-r4-graph-format/src/inference_contract.rs` constrain deployed operations to integer/bitwise/compare/popcount/table-read/bounded-state behavior.
- #602 will define the source/target operator identity; #603 will provide the trace container used for later fitting.

# Problem

There is no first genuinely R4-native attention/relation operator with independently stated reference semantics, packed lowering, bounded operation accounting, and replayable witness. Calling the current `r4_attention` flag such an operator would be inaccurate.

# Goal

After the dormant-mechanism gate is cleared, specify and implement a versioned `R4RouteAttentionV1` reference operator and its packed runtime lowering without weakening the deployed contract.

# Scope

- Define fixed-width route codes, masked XOR/popcount relation, bounded top-M selection, deterministic ties, integer/table contribution aggregation, fixed-point residuals, state bounds, and witness inputs/outputs.
- Implement compiler-side/reference semantics first, then a packed R4G1 lowering using borrowed bytes and caller-owned bounded state.
- Add operation, byte-read, candidate-bound, float/multiply/divide, allocation, and witness-replay instrumentation.
- Keep source-teacher standard attention and target routing semantics as separate operators; fitting is a follow-up issue.

# Non-goals

- Do not reuse unchanged Q/K/V weights under the route equation.
- Do not enable `packed-routing-dormant` or change the serving default as part of the initial implementation.
- Do not claim quality improvement without the fitting/certification issues.
- Do not couple to `uor-matmul`, UOR-NAF, kappa-registry, or formal-proof repositories before #589 is unblocked.

# Dependencies and blockers

- **BLOCKED BY #515** until feature/ledger/activation-gate conventions and the relevant dormant route surfaces are settled.
- Depends on #602 for operator identity and #603 for trace/provenance inputs.
- #531 owns corpus-scale evidence; #589 blocks upstream integrations.

# Proposed implementation shape

Use the existing ROUT format/validation and graph-runtime routing primitives as the wire/runtime substrate. Keep a scalar reference implementation in compiler/certifier code, define a packed lowering only after reference semantics and bounds are testable, and preserve the current R4G1 serving path until an explicit activation gate clears.

# Acceptance criteria

- [ ] `R4RouteAttentionV1` has a versioned reference specification and deterministic tie-breaking.
- [ ] The reference and packed paths agree on a pinned synthetic fixture and replayable witness.
- [ ] Packed execution is bounded, allocation-free in steady state, and passes zero-float/zero-multiply/divide source/census gates.
- [ ] Candidate count, operations, bytes read, and state capacity are explicit and enforced.
- [ ] Existing runtime/artifact fixtures remain unchanged while the operator is dormant.
- [ ] A predeclared activation record identifies the quality/runtime verdict required before promotion.

# Verification

- Unit/property tests for masked relation, top-M selection, ties, saturation, and bounds.
- Reference-vs-packed differential tests and independent witness replay.
- Runtime op census, allocation census, no-std check, deterministic artifact rebuild, and `cargo fmt`/clippy/workspace tests.
- No quality claim is accepted without a pinned source/corpus/operator report.

# Compatibility and migration

Emit the operator only in a new operator/artefact version or optional section. Historical artifacts and the current default runtime remain valid. Activation must be a separate reviewed change tied to #515’s ledger gate.

# Claim status

Definition, implementation, empirical criterion, and proof obligations. It is not a proof of behavioral equivalence or a measured quality result.

# Experimental run contract

metric to move: teacher-forced top-1/top-k agreement and bits/token while preserving the runtime operation contract.
current baseline: no R4-native target-operator baseline; the current `r4_attention` source branch is not a valid target baseline and no number is invented.
reachability ceiling: awaiting #603 traces and a reference-operator fixture.
cheap instrument and required verdict: synthetic reference-vs-packed replay plus short pinned teacher-forced sample; first verdict must prove semantic/operation/witness correctness before quality interpretation.
exit rule: stop if the packed path violates any runtime bound, fails witness replay, or cannot be distinguished from a declared null under the predeclared test.
if positive: hand the operator and traces to the fitting issue; keep dormant until the activation gate clears.
if negative: retain the operator/reference and negative report; do not promote or delete it.
cost estimate: reference semantics + packed lowering + certification harness; potentially one medium engineering tranche.
pinned source CID: supplied by #597/#599.
pinned corpus CID: supplied by #531’s eventual saturation-selected result; absent for the synthetic preflight.
target operator ID: `R4RouteAttentionV1`.


## Comments (3)

### comment 5268592808 — 2026-08-12T15:03:09Z — Casey-allard

## Plan (blocker cleared: #515 is CLOSED — its ledger/activation-gate conventions are settled and already exercised by tropical-route-dormant et al.)

1. **Reference semantics first** (compiler/certifier-side, floats-legal zone but this operator needs none): versioned `R4RouteAttentionV1` spec — fixed-width route codes, masked XOR/popcount relation on the existing ROUT substrate (graph-format rout.rs + graph-runtime routing.rs reused, placeholder packed_kernels.rs NOT treated as evidence), bounded top-M selection with deterministic lowest-index ties, integer/table contribution aggregation, fixed-point (ScoreQ) residuals, explicit state bounds, witness inputs/outputs. Registered in #602's operator registry as the first TARGET operator entry.
2. **Packed R4G1 lowering second**: borrowed bytes, caller-owned bounded state, allocation-free steady state; emitted ONLY in an optional section/new operator version — historical artifacts byte-identical; serving default untouched; `packed-routing-dormant` NOT enabled.
3. **Instrumentation + censuses**: op/byte-read/candidate-bound counters, zero-float/zero-multiply/divide source scan (P-4 discipline — runtime code in scanned source), allocation census, independent witness replay.
4. **Differential + property tests**: reference-vs-packed agreement on a pinned synthetic fixture; masked relation, top-M, ties, saturation, bounds properties; deterministic rebuild.
5. **Predeclared activation record**: ledger dormant lane (#515 pattern) stating the quality/runtime verdict required before promotion; the experimental-run contract from the issue body carried verbatim into the record. Fitting = #605, quality claims = none here.

Discovered-work note: an **MSA-structured selector** (Modular Structural Arithmetic role-classes + cascade orbits — Casey's early-r3/r4 mechanism, papers on file) is deliberately NOT folded into V1 (the issue prescribes the XOR/popcount relation); it will be filed as a follow-up candidate operator to be A/B'd under the same registry + pre-registration discipline. Six gates + census suites before the PR.

### comment 5269561736 — 2026-08-12T16:30:05Z — Casey-allard

## Delivery record (PR #642 in queue)

Acceptance: **(1)** `R4RouteAttentionV1` versioned reference spec with deterministic lowest-index ties — 288-bit route codes (width justified from the signature substrate), masked XOR + table-popcount relation, bounded top-M (M≤8, N≤64), ScoreQ saturating aggregation in pinned selection order. **(2)** Reference and packed paths agree bit-for-bit on the pinned synthetic fixture; witness (`uor-r4-route-attention-witness/1`) independently replayable — the replayer verifies digests, per-slot distances, strict selection order, completeness-by-optimality, aggregate refold, and the closed-form census WITHOUT running the operator; 12 tamper classes rejected by name. **(3)** Packed execution: borrowed bytes, caller-owned bounded state, allocation-free steady state (census-asserted ×8 steps = 0 allocations), and the lowering lives IN a P-4-scanned file added to the scan list — zero float/multiply/divide by machine check, not assertion. **(4)** Candidate count, ops, bytes read, and state capacity are explicit closed forms of (N, M), enforced with sanctioned-error refusal. **(5)** Nothing enters R4G1 artifacts (separate canonical RAT1 objects; optional-section mechanism documented for activation); historical fixtures byte-identical, deterministic rebuild --include-ignored green; `packed-routing-dormant` NOT enabled; serving default untouched. **(6)** Predeclared activation record = ledger `r4-route-attention-dormant` (first verdict = correctness — established here; quality only via the fitting issue). Registered as `r4-route-attention/1` in the #602 registry (no QKV reuse). Honest note: instances are NOT wired through GraphView/ROUT sections — that surface belongs to packed-routing-dormant, which must stay unactivated; carriage decision documented. Discovered-work filed: https://github.com/UOR-Foundation/uor-r4/issues/643 (MSA-structured selector as a separate candidate operator). Seven gates green (128 suites). Closes on #642 merge; #605 (fitting) next.

### comment 5270028651 — 2026-08-12T17:14:07Z — Casey-allard

Closing: PR #642 merged (2026-08-12T16:49:57Z). Delivery record: https://github.com/UOR-Foundation/uor-r4/issues/604#issuecomment-5269561736 — R4RouteAttentionV1 shipped dormant (ledger lane r4-route-attention-dormant): RAT1 wire format + parse/build/census in graph-format, packed zero-mult/float step in graph-runtime (P-4 scan extended), reference + witness (12 tamper classes) in graph-certify, 0-allocation packed steps in the census. Instances deliberately NOT wired through GraphView/ROUT — that surface belongs to packed-routing-dormant. MSA-structured selector follow-up filed as #643 (unassigned, pending publishability word).


---
_Comment/review ids in this file: 5268592808, 5269561736, 5270028651_

