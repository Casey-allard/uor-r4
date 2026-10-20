# #160 — Machine-code, allocator, and dependency CI enforcement for the inference contract

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:19:28Z
- **Updated:** 2026-07-24T22:30:01Z
- **Closed:** 2026-07-24T22:30:01Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/160
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #157 (defines the forbidden set, CPU-only target, and zero-allocation lifecycle) and #159 (defines the audited kernel targets).
Implements: the "disassembly audit" leg of proof obligation T2 (plan §7: "census + source scan + disassembly audit") plus the allocator, no-alloc-build, and dependency/feature gates the expanded contract requires.
Source: plan §7 (T2), `docs/transformerless/PROOF.md` (P1), `docs/transformerless/BASELINE.md` §3.3, direction PDF §13.

> Scope expanded (2026-07-24 update) from disassembly-only to the full CI enforcement layer: machine-code audit, allocator-failure testing, no-`alloc` build gate, and GPU/dependency/feature auditing, with negative fixtures for multiplication, floating point, **and** allocation. This issue owns the enforcement *machinery*; #159 owns making the kernel *code* clean.

## Motivation

All current no-multiplication evidence is source-level: the P-4 scan covers one file (`crates/uor-r4-core/src/transformerless/mod.rs:79-132`), and the `OpKernel` interface simply lacks a multiply method. Neither sees what LLVM emits: indexing, strides, iterators, bounds checks, `memcpy` expansions, and library helpers can lower to `imul` or worse in optimized builds. Likewise, the zero-allocation evidence is a single counting-allocator test over two paths, and there is **no** gate anywhere for GPU/tensor dependencies, accidental `alloc` linkage, or Cargo feature unification pulling forbidden dependencies into the normative runtime. PROOF.md P1 and BASELINE.md §3.3 currently claim "0 multiplies (machine-checked source scan)" — a source scan cannot substantiate that claim for release machine code.

## Scope — four enforcement layers

### Layer 1: Source and type-level enforcement (CI gates)

- Integer-only hot-path APIs: no floating-point types in production inference interfaces (compile-checked, e.g. via the contract registry or a `static_assertions`-style test).
- No allocator-owning containers in hot-path state types (source scan over contract-owned modules per #157's ownership map, superseding the one-file P-4 scan).
- Explicit fixed capacities, explicit overflow semantics, no hidden dynamic dispatch (`dyn` ban in hot path), no recursive traversal (source checks).

### Layer 2: Disassembly audit of optimized release machine code

- **Architecture matrix (initial):** x86_64 portable scalar; x86_64 with permitted CPU SIMD/POPCNT; AArch64 portable scalar; AArch64 with permitted NEON operations.
- **Audited symbols:** a declared, stable set of representative kernel symbols plus end-to-end inference entry points (`R4G1Runtime::step`/`predict_token`, legacy `predict`/`generate_greedy_into`, bounded `*_into` façade functions). Symbol retention for audit is a deliverable (export list / `#[no_mangle]` shims / documented strip policy). Callees reachable from audited entry points are examined where tooling permits.
- **Forbidden classes (per #157 §3):** integer multiply (including widening and high-half forms), integer divide/remainder, scalar and vector floating-point arithmetic, FMA/multiply-subtract, dot-product, matrix, and tensor instructions, and FP-conversion instructions used to enter floating-point computation — matched per-arch (`imul/mul/div/idiv`, `mulps/mulpd/mulss/mulsd`, `divps/...`, FMA, `pmaddwd`/VNNI on x86_64; `mul/madd/msub/smull/umull/sdiv/udiv`, FP mul/div, `fmla/fmls`, dot/matrix extensions on AArch64; equivalents on future targets).
- **Compiler-generated sources** the audit must catch regardless of origin: indexing, strides, slice iterators, bounds checks, intrinsics, and library helpers lowering to forbidden arithmetic.
- **Legal address generation:** per-arch policy files whitelist address-generation forms (`lea`, base+offset, base+scaled-index addressing modes, shift-based indexing) with documented rationale, keeping false positives at zero — a scaled load/store address is never flagged as semantic multiplication (per #157 §4).
- **CI artifacts:** each run uploads the audit report (tool, pinned version, binary hash, per-symbol verdicts, matched instructions) as a build artifact; #161 consumes the report CID.

### Layer 3: Allocator-failure testing and no-`alloc` build gate

- **Failing/counting test allocator:** records allocation calls; panics or fails on any allocation or deallocation after inference initialization; proves repeated inference completes with zero allocation and deallocation calls; covers success, novelty, fallback, contradiction, and error paths; tests multiple tokens/steps, not one trivial call (extends `crates/uor-r4-core/tests/allocation_census.rs` from assertion-on-two-paths to the full lifecycle).
- **No-`alloc` build configuration:** a CI configuration builds the production inference kernel with `#![no_std]`, **without `alloc`**, and with default features disabled, wherever feasible (today `uor-r4-graph-runtime` links `alloc` unconditionally — this issue adds the gate as the target configuration lands per #157's isolation goal). Passing `no_std` with `alloc` still enabled is explicitly **not** accepted as zero-allocation evidence.

### Layer 4: Dependency and feature auditing

- CI verifies the normative runtime dependency graph contains no required: CUDA, ROCm, Metal (Performance Shaders or compute kernels), OpenCL, WebGPU, Vulkan compute, DirectML, oneAPI/SYCL, tensor libraries, BLAS libraries, GPU runtime, device abstraction, or allocator-dependent graph representation. (Current state is clean — workspace `Cargo.toml` has no such dependencies; this gate keeps it that way.)
- Checks cover: default Cargo features; target-specific dependencies; optional features accidentally enabled by workspace builds (feature unification); transitive GPU or tensor dependencies; transitive use of `alloc` in the intended no-alloc configuration.
- Mechanism options (choose in implementation): `cargo deny` bans, a `cargo tree`-based script, or lockfile diff gates — pinned and versioned in-repo.

### Negative fixtures (must fail)

CI includes fixtures that intentionally emit (a) multiplication, (b) floating-point arithmetic, and (c) heap allocation behind test-only features, and proves each corresponding gate flags them. A gate that cannot fail is not evidence.

## Non-goals

- Auditing compiler, certifier, teacher, or test-only code (unrestricted per #157).
- Forbidding CPU SIMD or specific microarchitectural features outright (POPCNT/AVX2/NEON logical forms are permitted — only the forbidden instruction classes are policed).
- Formal verification of the binary (Kani/source-level territory, #132); these are empirical-absence and build-configuration checks.
- Replacing the op census (it remains a complementary T2 leg consumed by #161).
- Any GPU backend validation (GPU is a non-goal per #157 §6; Layer 4 exists to keep it out).

## Relevant code, docs, files

- `crates/uor-r4-core/src/transformerless/{mod.rs:79-132, runtime.rs}`, `crates/uor-r4-core/tests/allocation_census.rs`, `crates/uor-r4-proof-model/src/allocation_proof.rs` (thread-local counting allocator)
- `crates/uor-r4-graph-runtime/src/` (`lib.rs:1-4` `extern crate alloc`), `src/r4g1.rs`
- workspace and crate `Cargo.toml` files; `Cargo.lock`
- `.github/workflows/ci.yml` (gates job, no_std ladder, fuzz smoke), `Justfile`
- `docs/transformerless/PROOF.md` (P1), `docs/transformerless/BASELINE.md` §3.3, plan §7 (T2)

## Dependencies

- #157 — supplies the machine-readable forbidden-op classes, the audited boundary, the CPU-only dependency policy, and the zero-allocation lifecycle phases.
- #159 — supplies the finalized kernel symbol set and alloc-free code.
- Consumed by: #161 — audit report CIDs, allocator-run evidence, and dependency-audit results become certificate evidence; #132 records the claim-status upgrade to **Structural**.

## Deliverables

1. Disassembly audit tool (repo-owned, pinned `llvm-objdump`/`objdump` toolchain) driven by the #157 machine-readable contract and per-arch opcode policy files.
2. Symbol-retention mechanism and documented strip/link policy for audited release builds.
3. Failing/counting allocator harness covering the full inference lifecycle and all ResolutionStatus paths.
4. No-`alloc` build configuration for the production kernel plus its CI gate.
5. Dependency/feature audit (GPU/tensor/BLAS denylist, feature-unification check, transitive-`alloc` check) in CI.
6. Negative fixtures for multiplication, floating point, and allocation, each with a must-fail CI check.
7. CI jobs (release mode, per available tier; cross builds where the runner matrix allows) uploading reports as artifacts.
8. Wording updates to PROOF.md P1 / BASELINE.md §3.3 so claims cite machine-code and allocator evidence (subject to the claim-wording gate).

## Acceptance criteria

- The disassembly audit inspects **optimized release machine code** (`--release`), not source, MIR, LLVM IR, or debug builds; source review alone is never accepted as evidence.
- Every declared audited symbol on every available tier reports zero forbidden instructions, or CI fails; reachable callees are examined where tooling permits.
- The failing-allocator suite proves zero allocation and zero deallocation across success, novelty, fallback, contradiction, and error paths over multiple steps.
- The no-`alloc` build configuration compiles the production kernel without the `alloc` crate (where feasible per #157's decision), or the residual `alloc` dependence is explicitly justified, gated off the hot path, and tracked.
- The dependency/feature audit fails CI on any GPU, tensor, BLAS, or device-abstraction dependency — direct, optional-enabled, or transitive — in the normative runtime graph.
- Each negative fixture (multiply, float, allocation) fails its gate in CI.
- Legal address-generation instructions never trip the audit; policy files record the rationale per allowed mnemonic.
- Audit reports are uploaded as CI artifacts and content-addressed so #161 certificates can cite them.
- Adding a forbidden class or platform rule to the #157 contract bumps the audit policies in the same PR (drift check).
- Four gates green; new CI jobs integrate without breaking the existing job set.

## Formal claim classification

- "No forbidden instruction appears in audited release inference machine code" and "repeated inference performs zero heap operations": **Guarantee**, status **Witnessed** (per-build machine-code and allocator witnesses; upgrade path to **Structural** recorded in #132's matrix per `docs/formal_vocabulary.md` §2).
- Absence on non-audited builds/tiers: explicitly **Unproven**, never implied.

## Test requirements

- **Unit:** policy-file parsing; symbol discovery; verdict logic on canned disassembly snippets (positive and negative); dependency-audit rule evaluation on fixture lockfiles.
- **Property:** none required.
- **Fuzz:** none (audit inputs are pinned release binaries and lockfiles).
- **BDD/Cucumber:** none required (CI gates, not runtime behavior).
- **Benchmark:** none; audit builds must not alter the production build profile used by benchmarks (separate audit build if symbol retention changes codegen — documented either way).

## Security considerations

Closes the supply-chain/toolchain gap in the no-multiplication and no-allocation claims: a compiler upgrade, target-feature change, dependency bump, or feature unification that introduces forbidden instructions, GPU dependencies, or hidden allocation now fails CI instead of silently invalidating P1/Gate B. Audit tooling pins its disassembler and treats unexpected mnemonics as failures, not passes.

## no_std / no-alloc implications

Core of this issue: Layer 3 adds the no-`alloc` build gate; the audited kernels remain `no_std`-compatible. Audit tooling itself is std-only CI tooling.

## CPU portability considerations

The architecture matrix (x86_64 scalar + POPCNT/SIMD, AArch64 scalar + NEON, portable fallback) is the portability evidence for #157's CPU-only target; per-arch opcode policies keep semantics equivalent across tiers.

## Traceability

- Plan §7 (T2), plan PDF §17; direction PDF §13; PROOF.md P1; BASELINE.md §3.3; monograph (#133) structural-proofs chapter; traceability matrix (#137).

## Related issues and PRs

#13, #14, #20, #22, #27, #32, #58, #132, #133, #137, #157, #158, #159, #161.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

