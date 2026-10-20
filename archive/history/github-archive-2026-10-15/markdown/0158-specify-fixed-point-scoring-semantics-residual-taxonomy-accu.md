# #158 — Specify fixed-point scoring semantics: residual taxonomy, accumulation order, saturation, and tie-breaking

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:19:07Z
- **Updated:** 2026-07-24T20:26:02Z
- **Closed:** 2026-07-24T20:23:33Z
- **Assignees:** afflom, Casey-allard, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/158
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #157 (normative CPU-only, multiplication-free, zero-allocation inference contract).
Extends: #19 (ScoreQ fixed-point residuals), #130 (Boolean/fixed-point lowering), #131 (bounded planning); relates to the scoring-redesign chain #64, #68, #69, #70.
Source: `docs/transformerless/R4G1.md` §9.3 (ScoreQ two-tier design), plan §6, direction PDF §§7, 12, `docs/formal_vocabulary.md` §§1-2.

> Scope confirmed and tightened (2026-07-24 update): this issue remains the sole owner of runtime scoring semantics; wording aligned with the expanded contract in #157 (CPU-only, zero-allocation) and the CI enforcement in #160.

## Motivation

`ScoreQ` exists as a wire type (`crates/uor-r4-graph-format/src/types.rs:43`, i32 Q16.16, saturating add/sub) and #130 lowered residuals into fixed-width tables, but no single normative specification defines **runtime scoring semantics**: the residual taxonomy, accumulation order, saturation and sentinel behavior, deterministic tie-breaking, and overlap residualization. Today these are scattered across `score.rs`, `engine.rs`, the #64 sibling-stacking redesign, and plan prose — differential testing and the #160 machine-code audit need one frozen semantic contract to test against.

## Scope

Define the complete runtime scoring semantics as a versioned normative specification:

- **Compiler-emitted fully quantized residual scores only:** all scores entering the runtime are pre-quantized residuals emitted by the compiler (B(v) root priors, child corrections, interaction residuals per #19); the runtime performs **integer add/subtract/compare only** — no coefficient multiplication, no division, no softmax, no runtime rescaling, no floating point, no runtime cosine or dot-product similarity, no floating-point normalization.
- **Explicit score width and signedness:** the canonical semantic score is `ScoreQ` (signed i32, Q16.16, declared in HEAD per R4G1.md §9.3); per-table storage stays behind the two-tier `StorageDescriptor` `{width, shift, zero_point}` whose decode is shift+add.
- **Quantization metadata:** HEAD declares the global score format; per-section descriptors declare storage parameters; both validated at stage 1/2.
- **Deterministic accumulation order:** a fixed, documented accumulation order for every scoring path (routing, transition, emission, goal/constraint), so integer results are reproducible bit-for-bit.
- **Explicit overflow/saturation semantics:** declared saturating add/sub (current `ScoreQ` behavior) versus checked forms where required; boundary behavior at `i32::MIN/MAX` specified, matching threat-model row A4.
- **Sentinel values:** explicit specification of sentinel encodings (e.g. "no score", "saturated-low/high") and their ordering relative to real scores.
- **Deterministic tie-breaking:** canonical order (score descending, `TokenId` ascending) per `deterministic_topk_proof`, extended to every selection point (frontier eviction, shortlist, plan ranking).
- **Architecture-stable ordering:** scalar and CPU-SIMD kernels produce identical total orders; no reassociation is permitted where it could change integer results.
- **Residual taxonomy:** goal rewards, constraint penalties, uncertainty penalties, semantic-transition residuals, and token-emission residuals are distinct typed contribution kinds with declared composition rules.
- **Overlap residualization:** the no-double-counting rule (per #64/#69 interaction residuals and #125 contribution IDs) specified as part of score composition: each evidence contribution is counted exactly once.
- **Reference versus optimized kernel equivalence:** the scalar reference accumulator is normative; optimized kernels are equivalence-tested against it on exhaustive small domains and randomized differential suites (per #130's approach).
- **Quantization error:** measured during compilation and recorded in an **empirical quantization-loss certificate** (divergence/tolerance per `D(·,·) ≤ ε` in the vocabulary), never computed at runtime.

## Non-goals

- Changing the wire format of `ScoreQ` or `StorageDescriptor` (owned by R4G1.md §9.3; changes go through the RFC).
- Redesigning the scoring formula or quality chain (owned by the #64-#80 line of work); this issue freezes the *semantics* of whatever the compiler emits.
- Compiler-side float mathematics (unrestricted per #157).
- Proving teacher fidelity (empirical certification, separate gate).

## Relevant code, docs, files

- `crates/uor-r4-graph-format/src/types.rs:43-118` (`ScoreQ`), `src/records.rs` (storage-descriptor wire form)
- `crates/uor-r4-core/src/transformerless/score_q.rs` (`StorageDescriptor` decode form — the live duplication AGENTS.md warns about)
- `crates/uor-r4-graph-runtime/src/engine.rs` (accumulation sites), `crates/uor-r4-graph-certify/src/score.rs`, `score_runtime.rs`
- `crates/uor-r4-proof-model/src/deterministic_topk_proof.rs`, `kani_proofs.rs` (Kani-1 ScoreQ saturating-add; PR #151 machinery)
- `docs/transformerless/R4G1.md` §9.3, `docs/transformerless/THREAT_MODEL.md` (A4), plan §6/§9.9

## Dependencies

- #157 — scoring semantics may only use contract-permitted operations and the fixed-capacity memory model.
- Consumed by: #159 (kernels implement these semantics), #161 (quantization-loss certificate fields), #160 (audits the accumulator's machine code); proof obligations for safe fixed-point arithmetic in #132; monograph (#133) lowering chapter.

## Deliverables

1. Versioned normative scoring-semantics document under `docs/` (per formal-vocabulary labeling rules).
2. Typed contribution kinds and a canonical accumulator in the runtime crates implementing the specified order, saturation, sentinels, and tie-breaking — alloc-free, no_std-compatible.
3. Differential equivalence tests between the reference and any optimized accumulator.
4. Compile-report fields recording quantization error per residual kind, consumed by the certifier (#161).

## Acceptance criteria

- Every runtime scoring path accumulates compiler-emitted integer residuals with add/subtract/compare only: no runtime coefficient multiplication, division, softmax, rescaling, float normalization, or dot-product/cosine similarity — enforced by the #157 contract scan and #160's disassembly audit.
- Score width/signedness, accumulation order, saturation behavior, sentinel semantics, and tie-breaking are written down, versioned, and match the code.
- Identical total order of candidates on x86_64 and AArch64 (scalar and permitted SIMD builds) for the pinned fixture suite.
- Overlap residualization: property tests show no evidence contribution is counted twice for overlapping memberships.
- Quantization error per residual kind appears in compile reports with CIDs as an empirical quantization-loss certificate; the runtime performs no error computation.
- No floating-point types in production scoring interfaces; accumulator state is fixed-capacity and alloc-free.
- Four gates green (`fmt`, `clippy -D warnings`, `cargo test --workspace --offline`, no_std ladder); claim-wording gate passes.

## Formal claim classification

- Score arithmetic semantics (width, order, saturation, tie-breaking, no-double-counting): **Guarantee**, status **Structural** (Kani/property-test backed; extends Kani-1).
- Teacher-fidelity and quantization-loss results: **Empirical Criterion**, status **Empirical** — certified with CIDs and confidence intervals, never worded as equivalence (per the CI wording gate).

## Test requirements

- **Unit:** boundary values for saturation, sentinels, tie-breaking; descriptor decode at width/shift extremes.
- **Property:** associativity-safe accumulation under the declared order; no-double-counting over randomized overlapping memberships; reference-vs-optimized accumulator equivalence.
- **Fuzz:** storage-descriptor decode and accumulator inputs near `i32` extremes (extends the integer-boundary fuzzing obligation in THREAT_MODEL.md §5).
- **BDD/Cucumber:** scenario pinning canonical tie-breaking and saturation behavior, per `features/suites/` conventions.
- **Benchmark:** none new; existing Gate C/trend tooling consumes the frozen semantics.

## Security considerations

Directly addresses the integer-saturation threat row (crafted scores hitting i32 extremes): specified saturation and sentinel ordering remove undefined adversarial orderings. No panics on adversarial scores; typed compact errors only.

## no_std / no-alloc implications

The canonical accumulator and contribution types are no_std and alloc-free, matching `uor-r4-graph-runtime`'s `#![no_std]` stance and #157's no-`alloc` isolation goal.

## CPU portability considerations

Accumulator semantics are architecture-neutral: identical results on x86_64, AArch64, and the portable scalar fallback; any CPU-SIMD accumulation path is proven order-identical to the scalar reference.

## Traceability

- Direction PDF §7 (semantic compression: residuals), §12 (future-state scoring), §13; plan §6, §9.9; R4G1.md §9.3; `docs/formal_vocabulary.md` §§1-2; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#14, #15, #19, #20, #25, #32, #64, #68, #69, #70, #125, #130, #131, #132, #133, #137, #157, #159, #160, #161; PRs #88, #92, #98, #99, #149, #151.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

