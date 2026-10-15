# #161 — Runtime operation, allocation, and CPU portability certificates

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:19:44Z
- **Updated:** 2026-07-25T00:00:59Z
- **Closed:** 2026-07-25T00:00:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/161
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #157 (contract), #158 (scoring semantics), #159 (kernels), #160 (CI enforcement evidence).
Extends: #20 (certificate schema with CIDs), #27 (bytes-read/cache-miss/branch-miss counters), #14 (per-token op instrumentation).
Source: plan §9.10, §9.17, §9.4; `crates/uor-r4-graph-certify/src/{certificate.rs,performance_certificate.rs}`; R4G1.md §5 (CERT section); direction PDF §13.

> Scope expanded (2026-07-24 update) to own allocation evidence (allocation/deallocation counts, zero-allocation witnesses) and CPU portability records (feature requirements, portable scalar fallback, artifact compatibility, memory bandwidth/cache metrics) in addition to operation accounting.

## Motivation

Operation accounting today is fragmented and partly honor-system: `OpKernel` (`crates/uor-r4-core/src/transformerless/runtime.rs:30-89`) counts only adds/xors/shifts/compares/table-reads/candidate-scans (no popcount, no bytes-read); the #27 performance counters are **estimates over caller-supplied `bytes_read`** (`performance_certificate.rs:94-117`), not measurements; and `ProtocolAttestation.{zero_allocation_verified, no_multiply_verified}` (`certificate.rs:25-31`) are plain booleans asserted by the caller — nothing wires census, scan, audit, or allocator evidence into certificate construction. There is no artifact-level declaration of per-step operation bounds, no allocation/deallocation accounting, and no CPU portability record. This issue unifies declared bounds, measured counters, and enforcement evidence into a versioned operation, allocation, and CPU portability certificate.

## Scope

- **Certificate fields (per step/token unless noted):**
  - XOR operations; AND/OR operations; shifts/rotates; popcounts; additions/subtractions; comparisons;
  - routing probes; graph nodes visited; transition entries read; emission entries read; bytes read;
  - maximum active-frontier width; maximum planning-frontier width; maximum plan depth;
  - candidate count; fallback count; **allocation count; deallocation count**;
  - **declared-zero fields:** multiply, divide, floating-point arithmetic, FMA, dot products, matrix/tensor operations, GPU dispatch, GPU memory transfer, and dynamic allocation during repeated inference — each recorded as a declared bound of zero with its evidence link.
- **Evidentiary classes, kept separate in the schema:**
  1. **compiler-declared operation bounds** (per-artifact, emitted at compile time, stored alongside the CERT section per R4G1.md §5);
  2. **runtime-observed debug/benchmark counters** (measured on pinned fixtures in release mode);
  3. **disassembly-verified instruction absence** (audit report CID from #160);
  4. **allocation-instrumentation evidence** (failing-allocator run reference from #160);
  5. **CPU feature requirements** (minimum ISA per build tier; portable scalar fallback availability);
  6. **empirical latency, memory-bandwidth, and cache metrics** (hardware metadata per #27, honestly labeled measurement vs. estimate).
- **CPU portability record:** per artifact and runtime build — required CPU features (none beyond baseline for the portable scalar path; POPCNT/AVX2/NEON only as optional accelerations), confirmation that the portable scalar fallback executes the same artifact, and artifact compatibility across supported targets (same bytes, same semantics, per R4G1.md §7 determinism).
- **Artifact-level representation:** the compiler emits the declared-bound certificate content-addressed; the runtime/certifier attaches observed counts and evidence CIDs; `ProtocolAttestation` booleans are replaced by (or derived from) links to actual evidence artifacts, and `verify_attestation` validates the links.
- **Hot-path cost:** benchmark/debug counters are feature-gated or witness-only (the existing `OpKernel` pattern) and must not burden the production hot path — zero overhead when disabled, asserted by the #160 allocator and disassembly gates.
- **CI regression checks:** measured counters and declared bounds compared against pinned baselines with explicit thresholds (following the Gate C trend-alarm precedent in `scripts/check_gate_c_regression.py`).
- **Claim-wording discipline:** documentation and certificate text must state that operation, allocation, and portability certificates bound runtime cost and platform behavior only; they do **not** by themselves prove teacher equivalence — teacher fidelity remains an empirical certificate (per `docs/formal_vocabulary.md` §2.1 and `EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`).

## Non-goals

- Hardware performance-counter (PMU) integration — estimates vs. measurements must be honestly labeled, but PMU work is out of scope.
- Changing the Gate C quality metrics or teacher-fidelity certification.
- Extending `OpKernel`'s semantics beyond counting (it remains witness tooling).
- New artifact sections (CERT references the existing schema; format changes go through the R4G1 RFC).
- Certifying any GPU execution (GPU is a non-goal per #157 §6; the certificate records GPU dispatch/transfer as declared-zero fields).

## Relevant code, docs, files

- `crates/uor-r4-graph-certify/src/{certificate.rs:25-134, performance_certificate.rs:25-117}`, `tests/performance_certificate_test.rs`
- `crates/uor-r4-core/src/transformerless/runtime.rs:30-93` (`OpKernel`), `crates/uor-r4-core/tests/allocation_census.rs`
- `scripts/check_gate_c_regression.py`, `scripts/gate_c_trend.sh`, `.github/workflows/ci.yml`
- R4G1.md §5 (CERT), §7 (reproducibility); plan §9.10/§9.17/§9.4; `docs/transformerless/PROOF.md` (P1); `docs/formal_vocabulary.md`

## Dependencies

- #157 defines the operation classes, CPU-only target, and lifecycle phases; #158 defines what counts as a scoring operation and supplies the quantization-loss certificate fields; #159 is what is counted; #160 supplies disassembly, allocator, and dependency-audit evidence.
- Feeds the proof-status matrix (#132) and traceability matrix (#137).

## Deliverables

1. Versioned certificate schema extension (serde, CID-addressed per #20 conventions) with the fields and six evidentiary classes above.
2. Measured (not estimated) bytes-read and per-class op counters, plus allocation/deallocation counters, behind a witness/debug feature, wired into benchmark reporting.
3. Compiler emission of declared per-artifact operation bounds and CPU feature requirements.
4. Replacement of honor-system attestation booleans with evidence links; `verify_attestation` validates the links.
5. CPU portability records per artifact/build tier, including portable-scalar-fallback confirmation.
6. CI regression checks against pinned counter baselines.

## Acceptance criteria

- The certificate distinguishes compiler-declared bounds, runtime-observed counters, disassembly-verified absence, allocation-instrumentation evidence, CPU feature requirements, and empirical latency/cache metrics as separate, individually-CID'd components.
- All fields listed under Scope are present, including allocation and deallocation counts; every declared-zero field (multiply, divide, floating point, FMA, dot products, matrix/tensor ops, GPU dispatch, GPU memory transfer, dynamic allocation during repeated inference) carries its evidence link.
- Counters are measurably zero-overhead when disabled (release hot path unchanged; #160's disassembly audit still passes on the production build).
- The portability record shows the same artifact executing under the portable scalar fallback and any accelerated CPU tier with identical outputs.
- CI fails on counter regressions beyond declared thresholds and on attestation entries lacking evidence links.
- Documentation states explicitly that operation/allocation/portability certificates do not prove teacher equivalence.
- Four gates green; claim-wording gate passes on all new text.

## Formal claim classification

- Declared bounds and declared-zero fields: **Guarantee**, status **Witnessed** (census/audit/allocator evidence) with upgrade to **Structural** tracked in #132.
- Observed counts, latency, bandwidth, and cache figures: **Empirical Criterion**, status **Empirical** (pinned fixtures, hardware metadata, confidence where applicable).
- Teacher fidelity: remains a separate **Empirical Criterion** — never inferred from these certificates.

## Test requirements

- **Unit:** schema round-trip, CID stability, attestation-link validation, counter enable/disable behavior, portability-record construction.
- **Property:** counter invariants (e.g. nodes visited ≤ declared frontier bound × steps; allocations = 0 in repeated inference) on randomized fixture graphs.
- **Fuzz:** none new (certificate parsing reuses the CBOR/schema validation path).
- **BDD/Cucumber:** scenario asserting a certificate carries evidence links for every declared-zero field, per `features/suites/` conventions.
- **Benchmark:** pinned-fixture runs producing the observed-count and latency/cache components in release mode, per CPU tier.

## Security considerations

Removes trust-on-assertion attestations from the certificate path (tampered-artifact threat row): a certificate claiming zero multiplication or zero allocation must cite the audit that demonstrated it. Counter regression alarms surface compiler/toolchain drift early. Declared-zero GPU fields make any future accelerator experiment visibly non-normative.

## no_std / no-alloc implications

Certificate schema types remain serde-only and allocation-bounded; counter instrumentation stays out of `no_std` hot-path crates (feature-gated witness code only).

## CPU portability considerations

This issue is the record of portability: per-tier CPU feature requirements, scalar-fallback equivalence, and cross-target artifact compatibility are first-class certificate content, not prose.

## Traceability

- Plan §9.10, §9.17, §9.4; R4G1.md §5, §7; direction PDF §13; PROOF.md (P1, P5); monograph (#133) certification chapter; traceability matrix (#137).

## Related issues and PRs

#14, #20, #22, #27, #32, #58, #77, #132, #133, #137, #157, #158, #159, #160; PRs #55, #86.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

