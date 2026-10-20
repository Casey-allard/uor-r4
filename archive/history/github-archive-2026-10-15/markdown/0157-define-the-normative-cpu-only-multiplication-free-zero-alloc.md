# #157 — Define the normative CPU-only, multiplication-free, zero-allocation inference contract

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:18:59Z
- **Updated:** 2026-07-24T21:06:36Z
- **Closed:** 2026-07-24T21:06:36Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/157
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: `docs/r4_graph_compiler_implementation_plan.md` §6 ("Runtime contract and R4G1 summary"), implementation-plan PDF §17 ("Allowed Runtime Operation Contract"), direction PDF §1 (guarantee inventory), `docs/transformerless/GLOSSARY.md` ("Normative kernel", "Step (R_G)"), `docs/transformerless/PROOF.md` (P1), `docs/transformerless/R4G1.md` §9.3, `docs/formal_vocabulary.md` §§1-2.

> Supersedes the original body of this issue (2026-07-24), which covered multiplication-freedom only. Scope expanded to own the full normative contract: multiplication-free, **CPU-only**, and **zero-heap-allocation** inference.

## Motivation

The multiplication-free rule exists today only as prose (plan §6, plan PDF §17, GLOSSARY "Normative kernel"), a one-file source scan (witness P-4, `crates/uor-r4-core/src/transformerless/mod.rs:79-132`), and the de-facto `OpKernel` method set (`crates/uor-r4-core/src/transformerless/runtime.rs:30-89`). Nothing normatively defines the **inference boundary**, the **execution platform** (no issue or doc mentions GPU/CPU targeting at all — the CPU-only stance is currently implicit), or the **zero-allocation lifecycle**. Concretely: the R4G1 graph path is unscanned and contains live multiplies reachable from scoring (`crates/uor-r4-graph-runtime/src/cayley_dickson.rs:17,27`); the legacy hot path contains `unreachable!` panics (`runtime.rs:580,621`) and heap-resident state (`Store = Vec<BTreeMap<...>>`, `runtime.rs:138`); and `uor-r4-graph-runtime` links `alloc` unconditionally. The dependent work — scoring semantics (#158), packed kernels (#159), CI enforcement (#160), certificates (#161) — needs one normative contract to build against.

## Normative definitions (to be added to `docs/formal_vocabulary.md` and GLOSSARY)

- **Inference boundary** — the exact set of runtime activities the contract governs (enumerated under Scope).
- **Normative execution target** — ordinary CPUs; the required platform for production inference.
- **Zero-allocation lifecycle** — the six-phase boundary below, after which every inference step performs zero heap operations.
- **Legal address generation** — the addressing-mode exception to the no-multiplication rule, defined precisely below.
- **Forbidden instruction classes / permitted operation classes** — the negative and positive lists below, versioned with this contract.

## Scope

### 1. Inference boundary (activities in contract)

- incremental context/signature update (`RuntimeState` token-state push, rolling-signature programs);
- semantic-region routing (ROUT/Boolean routing-program evaluation);
- candidate verification (masked-Hamming predicate evaluation);
- active-frontier updates (`ActiveFrontier` push/evict at bounded width `A`);
- transition scoring (E_f forward transition residuals);
- goal and constraint scoring (lowered planner scoring subset per #131);
- token candidate scoring and shortlist accumulation (EMIT/EXCT residual reads, top-K selection);
- fixed-width planning (bounded plan-frontier propagation within HEAD limits);
- decoding support used by the runtime (ScoreQ storage-descriptor decode, shift+add per R4G1.md §9.3).

### 2. Zero-allocation lifecycle boundary

1. Artifact bytes are loaded or memory-mapped (external I/O — outside the guarantee).
2. Artifact structure is validated (two-stage validation, R4G1.md §6).
3. A borrowed or statically owned graph view is created (`GraphView<'_>`).
4. Caller supplies all runtime state and output buffers, or the runtime creates them from fixed-size stack/static storage.
5. Inference begins.
6. Every inference step executes with **zero** allocation, reallocation, deallocation, allocator lock acquisition, and lazy heap initialization.

Whether *initialization* (phases 2-4) must also be allocation-free is decided explicitly in this issue: adopt the stronger requirement (validation and view construction are already allocation-free in `uor-r4-graph-format`/`GraphView`) and record any exception with its owner. The repeated-inference API (phase 6) is unconditionally zero-allocation.

### 3. Forbidden instruction classes (production inference call graph)

- scalar integer multiply; widening integer multiply; high-half integer multiply;
- scalar integer divide; scalar integer remainder where lowered through division;
- scalar floating-point arithmetic; SIMD/vector multiply; SIMD/vector divide;
- fused multiply-add or multiply-subtract; dot-product instructions; matrix instructions; tensor instructions;
- floating-point normalization; runtime coefficient scaling; runtime softmax; runtime cosine or dot-product similarity;
- conversion into floating-point arithmetic for subsequent computation.

Instruction families named where applicable — x86/x86_64: `mul`, `imul`, `div`, `idiv`, `mulss/mulsd/mulps/mulpd`, `divss/divsd/divps/divpd`, FMA families, vector dot-product families. AArch64: `mul`, `madd`, `msub`, `smull`, `umull`, integer divide, floating multiply/divide, `fmla`/`fmls`, dot-product and matrix-extension families. Plus architecture-specific equivalents on any additional target.

### 4. Legal address-generation exception

Permitted addressing includes normal ISA address-generation forms: base + constant offset; base + index; base + scaled index when encoded as part of a load/store addressing mode; pointer advancement through packed arrays. A scaled load/store address is **not** semantic multiplication provided: no multiply instruction is emitted; the scaling only locates fixed-size packed data; no computed numeric result is exposed to inference arithmetic. Multiplication by powers of two in *semantic* arithmetic is lowered to shifts **by design**, not left to compiler luck.

### 5. Permitted operation classes

XOR; AND; OR; NOT; shifts; rotates; popcount; bit extraction; bit deposit where supported and justified; leading/trailing-zero count; integer addition; integer subtraction; saturating or explicitly checked addition/subtraction; comparisons; equality tests; min/max; masks; conditional selection; fixed-capacity insertion or selection networks; bounded branches; branchless selection; table lookup; contiguous packed-array reads; validated indexed reads; CPU prefetch instructions if shown beneficial; legal CPU address-generation modes; fixed-width integer conversions that do not enter floating-point arithmetic. Every permitted operation has deterministic semantics across supported architectures or an explicitly defined architecture-neutral reference behavior.

### 6. CPU-only normative target

- The normative runtime targets ordinary CPUs and requires **no** GPU, accelerator, shader runtime, tensor runtime, or device driver. It must not require: CUDA, ROCm, Metal Performance Shaders, Metal compute kernels, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU shader compilation, GPU command queues, device memory, GPU synchronization, tensor accelerators, neural engines, vendor BLAS, GPU-resident artifacts, GPU runtime detection, or accelerator fallback logic.
- The normative artifact format is directly executable from CPU-addressable memory (already true for R4G1/`GraphView`; this issue makes it normative).
- Baseline implementations: portable scalar CPU path; x86_64 path; AArch64 path; optional CPU SIMD only when it obeys the same forbidden-instruction contract (XOR/AND/OR/shift/compare/integer-add/table-oriented forms only — never multiply, FMA, dot-product, matrix, or floating point).
- **GPU non-goal:** no GPU implementation is required or planned for the normative inference architecture. A future GPU backend may only exist as a separate, explicitly non-normative experiment that is not a dependency of the core runtime, does not influence artifact correctness, does not weaken CPU support, does not change deterministic semantics, is not required for acceptable operation, and never appears in the default feature set.

### 7. No-heap requirement and fixed-capacity memory model

The hot path must not use allocation-capable structures or APIs: `Vec`, `VecDeque`, `String`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `Box`, `Rc`, `Arc`, owned trait objects, allocating formatting, dynamically allocated error messages, dynamically growing logging buffers, heap-backed iterator collection, hidden `collect()` calls, allocating lazy global initialization, runtime graph construction, runtime deserialization into owned heap structures, or recursive structures requiring allocation.

Preferred: fixed-size arrays; const-generic bounded containers; borrowed slices; memory-mapped immutable artifact data; caller-provided scratch buffers; stack-resident runtime state; static storage where appropriate; packed offset-based graph views; fixed-capacity top-K structures; fixed-capacity active and planning frontiers; typed compact errors with no allocated payload.

### 8. Runtime API shape (ownership properties, adapted to repo conventions)

APIs shaped around caller-owned memory, in the style of the existing bounded `*_into` façade (`src/r4g1.rs:443-609`) and `generate_greedy_into` (`runtime.rs:628`). Representative direction, not a mandated signature:

```rust
pub struct RuntimeState<const ACTIVE: usize, const PLAN: usize> { /* fixed-capacity state only */ }
pub struct StepOutput<const K: usize> { /* fixed-capacity output only */ }

pub fn infer_step(
    graph: &GraphView<'_>,
    state: &mut RuntimeState<ACTIVE, PLAN>,
    token: u32,
    output: &mut StepOutput<K>,
) -> Result<(), RuntimeError>; // RuntimeError: typed, compact, non-allocating
```

### 9. Machine-readable representation and ownership

- A versioned contract module (suggested home: `crates/uor-r4-graph-format`, no_std, alloc-free) encoding the boundary activities, permitted/forbidden operation classes, and lifecycle phases — consumed by the source scan, the disassembly audit (#160), the certificate schema (#161), and the proof-status matrix (#132).
- Crate/module ownership map: each boundary activity assigned to `uor-r4-core::transformerless::runtime`, `uor-r4-graph-runtime::{engine,routing,runtime_state,status}`, or the `src/r4g1.rs` bounded façade (per `docs/transformerless/LOCAL_ONLY.md`).
- **Isolation goal:** the inference kernel compiles with `#![no_std]`, without `alloc`, with default features disabled, without GPU dependencies, and without floating-point dependencies wherever feasible (today `uor-r4-graph-runtime` links `alloc`; reducing that to a no-alloc configuration is part of this contract's adoption, with any residual `alloc` use explicitly justified and gated off the hot path).

### 10. Interactions

- Register "operation-set conformance", "CPU-only platform conformance", and "zero-allocation lifecycle" as named rows in the #135 invariant-ownership matrix and as obligation families in #132.
- CI enforcement expectations: extend the P-4-style source scan to every contract-owned module (or supersede it once #160's machine-code audit lands); record the contract version in artifacts and certificates.

## Explicit exclusions

Training; transformer teacher execution; offline behavioral probing; graph induction; clustering; compiler optimization; quantization; artifact generation; offline proof and certification tooling; test-only reference implementations unreachable from production inference. **These may use multiplication, floating point, tensors, GPUs, and heap allocation.** This work is not multiplication-free training or compilation.

## Non-goals

- Implementing the enforcement machinery (#160), scoring semantics (#158), kernels (#159), or certificates (#161).
- Any GPU backend (explicit non-goal; see §6).
- Restricting compiler, certifier, teacher, or tooling code in any way.
- Proving teacher equivalence or any behavioral claim — this issue is a Definition plus structural Guarantees only.

## Relevant code, docs, files

- `crates/uor-r4-core/src/transformerless/mod.rs:79-132` (P-4), `runtime.rs:30-89` (`OpKernel`), `runtime.rs:138` (heap `Store`), `runtime.rs:580,621` (`unreachable!`)
- `crates/uor-r4-graph-runtime/src/{engine.rs,routing.rs,runtime_state.rs,status.rs,cayley_dickson.rs,patch_chain.rs}`, `lib.rs:1-4` (`extern crate alloc`)
- `crates/uor-r4-graph-format` (`GraphView`, feature ladder `default = ["std"]`, `std = ["alloc"]`)
- workspace `Cargo.toml` (no GPU/tensor/BLAS dependencies exist today — the contract keeps it that way)
- plan §6, §7 (T2); R4G1.md §9.3; GLOSSARY.md; PROOF.md (P1); LOCAL_ONLY.md; formal_vocabulary.md §§1-2

## Dependencies

- Consumed by: #158 (scoring semantics), #159 (packed kernels), #160 (CI enforcement), #161 (certificates).
- Feeds invariant rows into #135 and obligation families into #132 (merged via PR #151).
- Vocabulary and claim language per #123 (`docs/formal_vocabulary.md`).

## Deliverables

1. Versioned normative contract document under `docs/` covering §§1-10 above, cross-linked from plan §6, GLOSSARY, and the monograph (#133) runtime-execution chapter.
2. Machine-readable contract module (no_std, alloc-free) with boundary activities, operation classes, lifecycle phases, and focused error types.
3. Vocabulary entries for the normative definitions above in `docs/formal_vocabulary.md` / GLOSSARY.
4. Crate/module ownership map and the initialization-strength decision for the zero-allocation lifecycle.
5. Extended source-level scan (or documented supersession plan) covering every contract-owned module, including `uor-r4-graph-runtime`.
6. Proof-matrix and invariant-ownership entries referencing this contract.

## Acceptance criteria

- The inference boundary is unambiguous: every production hot-path entry point is either listed as in-contract or explicitly excluded.
- The zero-allocation lifecycle is defined phase-by-phase; repeated inference is unconditionally zero-allocation; the initialization-strength decision is recorded.
- Hot-path public interfaces are integer-only; no floating-point types appear in production hot-path signatures.
- Allowed and forbidden operation lists match §§3-5 verbatim; the legal address-generation exception is stated exactly as in §4.
- CPU is the declared normative target; the GPU/accelerator non-goal list (§6) is stated verbatim, including "no GPU implementation is required or planned for the normative inference architecture."
- The contract is machine-readable and versioned; the version flows into artifacts and certificates.
- Every boundary activity maps to an owning crate/module; compilation and training are stated as unrestricted.
- The existing live violations are resolved or explicitly bounded: `cayley_dickson.rs` multiplies, legacy `unreachable!` panics, heap-resident `Store`, and `graph-runtime`'s `alloc` linkage — each either fixed, relocated outside the boundary, or recorded as a scheduled exception with an owner.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo test --workspace --offline` stay green; claim-wording gate passes on all new text.

## Formal claim classification

- The contract itself: **Definition** (formal_vocabulary.md §1).
- "The deployed hot path executes only permitted operations, on CPU, with zero heap allocation in repeated inference": **Guarantee**, status **Structural** only once #160's machine-code and allocator evidence exists; **Witnessed** (source scan + census) until then — the proof matrix must not record Structural prematurely.

## Test requirements

- **Unit:** contract module round-trip; boundary-activity registry covers every public hot-path entry point (test-enumerated).
- **Property:** none beyond existing suite.
- **Fuzz:** none (no parser at this layer).
- **BDD/Cucumber:** one scenario asserting contract document and module versions agree, per `features/suites/` conventions.
- **Benchmark:** none.

## Security considerations

Aligns with `docs/transformerless/THREAT_MODEL.md` (integer saturation/overflow): permitted overflow semantics are the declared saturating/checked forms (A4). CPU-only execution removes the GPU supply-chain and driver attack surface from the normative path entirely. Reduces the unaudited instruction surface of the runtime.

## no_std / no-alloc implications

The contract module builds under `no_std` without `alloc`, matching and extending the `uor-r4-graph-format` feature ladder. A `no_std` build with `alloc` still enabled is not evidence of zero allocation — the contract targets a no-`alloc` configuration for the production kernel wherever feasible (#160 owns the CI gate).

## CPU portability considerations

Permitted operations must have deterministic semantics on x86_64 and AArch64 (and the portable scalar fallback); any architecture-specific behavior (e.g. bit deposit, prefetch) gets an explicitly defined architecture-neutral reference behavior in the contract.

## Traceability

- Direction PDF §1 (guarantee inventory), §9 (invariants), §13 (structural proofs); plan §6, §7 (T2); plan PDF §17; formal_vocabulary.md §§1-2; monograph (#133) runtime-execution/resource-contract chapter; traceability matrix rows (#137).

## Related issues and PRs

#11, #13, #14, #17, #19, #20, #22, #25, #27, #28, #32, #130, #131, #132, #133, #135, #137, #158, #159, #160, #161; PRs #139, #149, #151, #154.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

