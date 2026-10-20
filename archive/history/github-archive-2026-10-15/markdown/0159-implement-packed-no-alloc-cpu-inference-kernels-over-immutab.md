# #159 — Implement packed no-alloc CPU inference kernels over immutable graph arrays

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:19:18Z
- **Updated:** 2026-07-24T20:30:55Z
- **Closed:** 2026-07-24T20:23:33Z
- **Assignees:** afflom, Casey-allard, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/159
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #157 (normative inference contract) and #158 (fixed-point scoring semantics).
Extends: #17 (reference ActiveFrontier + packed edge ranges), #28 (fixed-capacity RuntimeState), #13 (allocation census), #130 (lowering).
Source: plan §6 (`step()` "synchronous, deterministic, allocation-free, recursion-free, lock-free"), plan PDF §17, `docs/transformerless/BASELINE.md` §3.3/§4 (M.V.G. Gate B), direction PDF §1.

> Scope expanded (2026-07-24 update) to the full packed, no-alloc, CPU-only kernel set: explicit allocation-capable API ban, caller-owned/const-generic buffers, no GPU path, and behavioral identity between scalar and permitted CPU-SIMD kernels. This issue owns the *code* being allocation-free; #160 owns the CI *gates proving* it (failing allocator, no-alloc build, disassembly).

## Motivation

The R4G1 graph runtime is close to the contract but not there: `collect_target_leaf_nodes` is **recursive** (`crates/uor-r4-graph-runtime/src/engine.rs:393-418`) against plan §6; `cayley_dickson.rs:17,27` multiply inside the scoring path; the legacy hot path has `unreachable!` panics (`crates/uor-r4-core/src/transformerless/runtime.rs:580,621`) and heap-resident state (`Store = Vec<BTreeMap<...>>`, `runtime.rs:138`); `PatchChain` holds `Vec<GraphView>` inside the runtime crate; and the allocation census (`crates/uor-r4-core/tests/allocation_census.rs`) asserts zero allocation only for the legacy predict path and `predict_token`. This issue implements the production hot path as a complete set of allocation-free, CPU-only packed kernels over immutable graph arrays.

## Scope

Implement or harden the following kernels over `GraphView`'s borrowed, validated, immutable packed arrays:

1. **Incremental context/signature update** — fixed-capacity `RuntimeState` push (`copy_within` pattern) and rolling-signature program evaluation.
2. **Boolean / polynomial routing-program evaluation** — ROUT bytecode within HEAD `D` step bound.
3. **Masked-Hamming verification** — signature/mask/popcount predicate against region radii (#15).
4. **Bounded active-frontier update** — `ActiveFrontier` push/evict at width `A` with canonical ordering (#17).
5. **Semantic transition propagation** — packed E_f edge-range walks with reverse indexes (#18).
6. **Token shortlist accumulation** — caller-owned fixed buffers, capacity `K`, no full-vocabulary score array.
7. **Fixed-capacity top-K** — canonical tie-breaking per #158.
8. **Bounded plan-frontier propagation** — the fixed-capacity lowered subset of #131 within HEAD limits.
9. **Novelty/backoff behavior** — `ResolutionStatus`-driven widen/EXCT/abstain per #25/D4, with bounded fallback counts.

### Constraints (enforced in review and by #157/#160 tooling)

- **Memory:** fixed-size arrays or caller-owned buffers only; borrowed packed graph slices; stack-resident runtime state; no heap allocation, reallocation, or deallocation in the hot path; no allocation-capable structures or APIs (`Vec`, `VecDeque`, `String`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `Box`, `Rc`, `Arc`, owned trait objects, allocating formatting/errors, hidden `collect()`, allocating lazy init).
- **Platform:** CPU-only; no GPU path, accelerator dependency, or device abstraction of any kind (per #157 §6).
- **Arithmetic:** no multiplication, no division, no floating point; scoring per #158 semantics.
- **Control:** no recursive graph traversal (replace `collect_target_leaf_nodes` with an explicit fixed-capacity stack or bounded iteration); no virtual dispatch; no runtime graph construction; no runtime deserialization into owned heap structures; no full-vocabulary score allocation.
- **Capacities:** fixed capacities declared by the artifact (HEAD bounded-work constants `W, A, C, E, K, D`) or the runtime contract; per-step work within `O(D + A·C·W + A·E·K)`.
- **Boundaries:** packed immutable arrays with two-stage-validated ranges only; no unchecked indexing at untrusted artifact/runtime boundaries; typed compact non-allocating errors; **no panics** on malformed artifacts or exhausted capacities (remove the `unreachable!` sites).
- **API shape:** caller-owned memory in the style of the existing bounded `*_into` façade (`src/r4g1.rs`) — const-generic fixed-capacity state and output types per the representative shape in #157 §8, adapted to repo conventions.
- **Portability:** scalar (normative) and any optional CPU-SIMD kernels are behaviorally identical, differential-tested per GLOSSARY's Normative-kernel rule; SIMD only for permitted operations (XOR/AND/OR/shift/compare/integer-add/table-oriented), never multiply/FMA/dot-product/matrix/float.
- **no_std/no-alloc:** kernels in `uor-r4-graph-runtime` remain `#![no_std]`; hot-path code does not touch `alloc` (supports #157's no-`alloc` isolation goal and #160's no-alloc build gate).

### Out-of-boundary legacy handling

The legacy TLA/TLS path's heap-resident `Store` (`runtime.rs:138`) is either migrated behind the fixed-capacity boundary or formally excluded from the contract boundary per #157's ownership map — the decision is recorded, not left implicit.

## Non-goals

- SIMD/vectorized acceleration itself (optional later work; must pass the same differential and disassembly gates).
- Any GPU backend (non-goal per #157 §6).
- Changing artifact layout or HEAD constants (owned by the R4G1 RFC).
- Compiler-side kernel generation beyond emitting the packed arrays these kernels read.
- The CI enforcement machinery itself (#160) and certificate schema (#161).
- The wasm dashboard/router path (`crates/uor-r4-router` stays out of the graph migration path per AGENTS.md).

## Relevant code, docs, files

- `crates/uor-r4-graph-runtime/src/{engine.rs:35,63,393,421,780, routing.rs, runtime_state.rs, status.rs, cayley_dickson.rs, patch_chain.rs}`
- `crates/uor-r4-core/src/transformerless/runtime.rs:138,512-628`, `tests/allocation_census.rs`
- `crates/uor-r4-graph-format/src/view.rs` (`GraphView`), `src/stage2.rs`
- `src/r4g1.rs` (`*_into` bounded APIs), `docs/transformerless/LOCAL_ONLY.md`, plan §6, R4G1.md §4/§6

## Dependencies

- #157 — defines the permitted operation set, CPU-only target, and fixed-capacity memory model these kernels use.
- #158 — defines accumulation/saturation/tie-breaking the kernels implement.
- Consumed by: #160 (audits these kernels' release machine code, allocation behavior, and dependency graph) and #161 (counts these kernels' operations).
- Proof obligations consumed by #132; invariant rows by #135.

## Deliverables

1. The nine kernels above, integrated behind the existing bounded `*_into` runtime APIs, with focused non-allocating error enums.
2. Elimination of the recursion, multiplication, panic, and heap-state sites listed in Motivation (or their documented relocation outside the inference boundary, recorded per #157).
3. Extension of `allocation_census.rs` (or a sibling census) asserting zero allocation across **all** kernels, not only the predict path (the failing-allocator gate itself is #160's).
4. Property and differential test suites; BDD scenarios; benchmark coverage.

## Acceptance criteria

- Zero-allocation instrumentation asserts 0 allocations and 0 deallocations per step across every kernel in steady state (extending the Gate B census beyond `predict`/`predict_token`); covers success, novelty, fallback, contradiction, and error paths over multiple steps.
- No allocation-capable type or API appears in hot-path kernel signatures or state; hot-path errors are typed compact values.
- Bounded execution: worst-case per-step work derives from manifest-declared HEAD limits; adversarial fixtures (frontier exhaustion, candidate explosion per THREAT_MODEL.md) terminate at the declared bounds.
- Property tests cover frontier eviction ordering, shortlist accumulation, top-K canonical order, and transition propagation against the reference evaluator.
- Fuzzing covers artifact/range validation paths the kernels consume (extending the `uor-r4-graph-format` fuzz targets; malformed artifacts yield typed errors, never panics).
- Scalar and permitted CPU-SIMD kernels produce byte-identical outputs on the differential suite on x86_64 and AArch64.
- Benchmark coverage: per-token latency and bytes-read for each kernel on pinned fixtures in release mode, wired into existing benchmark reporting (not wall-clock asserts in unit tests).
- BDD/Cucumber scenarios for the deployed-path behaviors per repo convention (`features/suites/r4g1_runtime.feature` style).
- Four gates green (`fmt`, `clippy -D warnings`, `cargo test --workspace --offline`, no_std ladder); κ-reproduction unaffected or re-pinned per AGENTS.md procedure.

## Formal claim classification

- "Hot path performs zero heap allocation in steady state" and "execution is bounded by manifest-declared limits": **Guarantee**, status **Structural** (census + property tests; Kani where bounded per #132).
- Latency/bytes-read numbers: **Empirical Criterion**, status **Empirical**.

## Test requirements

- **Unit:** each kernel at empty/maximum capacity, saturation boundaries, and every typed-error path.
- **Property:** as listed in acceptance criteria.
- **Fuzz:** malformed-artifact and range-validation robustness; integer boundary inputs.
- **BDD/Cucumber:** deployed-path scenarios for routing, frontier, shortlist, and ResolutionStatus fallback.
- **Benchmark:** pinned-fixture per-kernel latency/bytes-read, release mode.

## Security considerations

Removes panic-based failure modes reachable from adversarial artifacts (THREAT_MODEL.md: tampered artifact bytes, frontier exhaustion, candidate explosion). All artifact-derived indexing remains validated; no new unsafe code (`#![forbid(unsafe_code)]` preserved). No allocator attack surface in repeated inference.

## no_std / no-alloc implications

Kernels in `uor-r4-graph-runtime` remain `#![no_std]`-compatible and do not use `alloc` on the hot path; the `uor-r4-graph-format` no_std ladder stays green; residual `alloc` use in the crate (e.g. `PatchChain`) is gated off the hot path with the justification recorded in #157's ownership map.

## CPU portability considerations

Scalar kernels are the architecture-neutral reference; CPU-SIMD variants (x86_64 POPCNT/AVX2 logical forms, AArch64 NEON logical forms) are optional, permitted-ops-only, and proven behaviorally identical. No kernel may require a specific microarchitecture for correctness.

## Traceability

- Direction PDF §1 (allocation-freedom guarantee), §9 (bounded frontier invariant), §13; plan §6, §9.3, §9.7, §9.18; BASELINE.md §4 (Gate B); monograph (#133) runtime chapter; traceability matrix (#137).

## Related issues and PRs

#13, #15, #17, #18, #22, #25, #28, #32, #78, #79, #130, #131, #132, #135, #137, #157, #158, #160, #161; PRs #50, #51, #92, #93.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

