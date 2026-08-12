# Inference Operation Contract (Normative)

- **Version:** 0.1.0
- **Status:** Normative for deployed inference hot paths
- **Role:** **Definition** (per `docs/formal_vocabulary.md` §1)
- **Machine-readable binding:** `uor-r4-graph-format::inference_contract`
- **Source anchors:** direction PDF §1 guarantee inventory; plan §6 / implementation-plan PDF §17; `docs/transformerless/GLOSSARY.md` ("Normative kernel", "Step (R_G)"); `docs/transformerless/R4G1.md` §9.3; `docs/transformerless/PROOF.md` P1; proof model issue #132.

This document defines the deployed inference operation-set contract and its runtime boundary.

## 1) Inference boundary (in scope)

The contract governs these production runtime activities:

1. Incremental context/signature update (`RuntimeState` token-state push, rolling-signature programs).
2. Semantic-region routing (ROUT / Boolean routing-program evaluation).
3. Candidate verification (masked-Hamming predicate evaluation).
4. Active-frontier updates (bounded-width push/evict).
5. Transition scoring (`E_f` forward transition residuals).
6. Goal and constraint scoring (lowered planner scoring subset).
7. Token candidate scoring and shortlist accumulation (EMIT/EXCT residual reads, top-K selection).
8. Fixed-width planning (bounded plan-frontier propagation within HEAD limits).
9. Runtime ScoreQ decode support used by inference (`StorageDescriptor` shift+add decode semantics per `R4G1.md` §9.3).

## 2) Allowed operations (positive list)

Allowed in contract-bound runtime execution:

- word XOR/AND/OR/NOT;
- shifts and rotates;
- popcount;
- integer addition/subtraction (including declared saturating and checked forms);
- integer comparisons and min/max;
- fixed-capacity selection/top-K;
- bounded branches and branchless selection;
- contiguous and indexed table reads;
- compiler-generated constant-offset addressing, provided audited machine code introduces no forbidden arithmetic.

Normative semantics are architecture-independent scalar safe Rust semantics. Optimized kernels are replaceable only if equivalence-tested.

## 3) Forbidden operations (negative list)

Forbidden in contract-bound runtime execution:

- scalar integer multiplication;
- SIMD/vector multiplication;
- floating-point arithmetic;
- division and remainder;
- fused multiply-add;
- dot-product instructions (`PMADDWD`, `UDOT`, `SDOT`, and equivalents);
- dense tensor operations and matrix multiplication;
- runtime normalization that requires multiplication or division;
- dynamic heap allocation.

## 4) Explicit exclusions (out of scope)

This contract does **not** restrict:

- training;
- transformer teacher execution;
- compiler optimization;
- clustering;
- graph induction;
- quantization;
- artifact generation;
- offline certification;
- test-only reference implementations outside the production inference path.

Compilation and training are explicitly unrestricted by this contract.

Note (non-normative cross-reference, #602): the source attention operator
specifications in `uor-r4-model-source::attention`
(`standard-source-attention/1`, `experimental-r4-source-attention/1`) are
host-side provenance records of teacher execution — an activity this
section already excludes. They are distinct from this deployed contract,
describe floating-point source computation the deployed hot path never
performs, and change none of the operation lists or obligations above.

Note (non-normative cross-reference, #604): `r4-route-attention/1`
(`R4RouteAttentionV1`, documented in `docs/MODEL_LIFECYCLE.md`) is a
DORMANT target operator whose packed lowering
(`uor-r4-graph-runtime::route_attention`, P-4-scanned) uses only
operation classes §2 already allows and none §3 forbids; its scalar
reference implementation is a test-only reference outside the production
inference path (excluded above), and while dormant
(`r4-route-attention-dormant` in `model/ledger.toml`) it participates in
no contract-bound serving activity. It changes none of the operation
lists or obligations above.

## 5) Activity ownership map

Each in-scope activity has a crate/module owner:

| Boundary activity | Owning module |
|---|---|
| Incremental context/signature update | `uor-r4-core::transformerless::runtime` |
| Semantic-region routing | `uor-r4-graph-runtime::routing` |
| Candidate verification | `uor-r4-graph-runtime::engine` |
| Active-frontier updates | `uor-r4-core::transformerless::reference_state` |
| Transition scoring | `uor-r4-graph-runtime::engine` |
| Goal and constraint scoring | `uor-r4-graph-runtime::engine` |
| Token candidate scoring + shortlist | `uor-r4-graph-runtime::engine` |
| Fixed-width planning | `uor-r4-graph-runtime::engine` |
| Runtime decode support (`*_into`) | `uor-r4-wasm-router::r4g1::{encode_into,decode_into,generate_into}` |

Machine-readable source of truth: `uor-r4-graph-format::ACTIVITY_OWNERS`.

## 6) Compiler-generated multiplication policy

For audited kernels in the boundary:

- indexing, strides, iterators, and address calculations must lower to shift/add or constant-offset addressing;
- source review is not evidence by itself;
- disassembly audit is the enforcement backstop (issue #160);
- until disassembly audit lands, source-scan witnesses remain **Witnessed** evidence, not **Structural**.

## 7) CI and evidence policy

- Source-scan coverage must include every contract-owned runtime module.
- Source-level witness may be superseded by machine-code audit once the release audit path exists.
- Contract version must remain synchronized between this document and the machine-readable module.
- Contract version is recorded by `INFERENCE_OPERATION_CONTRACT_VERSION` and consumed by proof/invariant ownership records.

## 8) Claim classification

- Contract text and operation lists: **Definition**.
- Claim "deployed hot path executes only contract-allowed operations": **Guarantee**.
  - Status now: **Witnessed** (source scan + allocation census).
  - Status target with disassembly audit: **Structural**.

## Changelog

- **0.1.0** (2026-07-24) — Initial normative contract definition and machine-readable binding.
