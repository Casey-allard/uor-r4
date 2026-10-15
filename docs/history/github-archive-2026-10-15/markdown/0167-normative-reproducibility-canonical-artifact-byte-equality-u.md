# #167 — Normative reproducibility: canonical artifact byte equality under compiler parallelism

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:43:28Z
- **Updated:** 2026-07-25T02:15:24Z
- **Closed:** 2026-07-25T02:15:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/167
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor), #166 (stage DAG).
Extends: #21 (deterministic artifact rebuild test in CI).
Source: R4G1.md §7 (D2: "identical graph content + format version ⇒ identical bytes", Gate E), plan §4.1, AGENTS.md determinism invariant ("identical pinned inputs ⇒ identical artifact bytes; no HashMap-iteration-order, clock, or RNG dependence in compiler outputs").

## Motivation

#21 landed a sequential deterministic-rebuild test, but nothing makes byte reproducibility normative **under parallelism**. Before #170–#173 add concurrency, the invariant and its enforcement harness must exist, or nondeterminism will be discovered after it is baked into stage designs. This issue owns the invariant and the harness; the stage issues owe it compliance.

## Scope

Make this invariant normative:

> Parallel execution may change compilation time, but must not change the canonical graph artifact produced from the same pinned inputs, compiler version, configuration, and target-independent compilation mode.

Required mechanisms:

- stable input ordering and stable shard numbering (content-addressed or positional, never scheduling-derived);
- stable candidate ordering and deterministic tie-breaking at every selection point;
- deterministic reductions only; stable sorting before any canonical output;
- deterministic deduplication; canonical node/edge ordering, IDs, and offsets; canonical certificate ordering; canonical bytes and content hashes;
- **floating-point policy:** no order-sensitive floating-point parallel reductions for canonical output — either fixed-order summation, integer/fixed-point accumulation for canonical quantities, or documented exact-reassociation-safe patterns; compiler float use stays allowed but canonical results must not depend on summation order;
- **random-seed policy:** pinned seeds recorded in the compile report; deterministic clustering initialization independent of thread scheduling;
- **thread-count invariance:** topology, IDs, offsets, hashes, and certificates identical at any thread count.

Required tests:

- sequential vs. parallel artifact **byte equality** (extends the #21 rebuild gate);
- thread-count sweep: 1, 2, 4, and available logical CPU count;
- repeated builds (double-compile determinism, per AGENTS.md drift-investigation procedure);
- randomized scheduler stress where feasible (varying batch sizes/chunkings to perturb scheduling);
- artifact hash equality and certificate equality across all of the above.

Forbidden patterns (normative, restated from the chain rules): no canonical IDs during parallel discovery; no completion order as semantic order; no appending into shared global graph structures; no unordered map iteration feeding canonical output; no order-sensitive FP parallel reductions; no first-finished-candidate-wins; thread count and Rayon scheduling must never influence topology or artifact bytes.

## Non-goals

- Implementing stage parallelism (#170–#173) or the executor (#165).
- The κ-reproduction baseline (the AGENTS.md Gate E macOS pin) — this issue covers artifact reproducibility, not teacher logprob pinning.
- Proving anything about teacher fidelity (empirical, separate).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler`, `crates/uor-r4-graph-certify`, deterministic-rebuild CI job (`.github/workflows/ci.yml`, extending #21/#58)
- R4G1.md §7, plan §4.1, `docs/formal_vocabulary.md`

## Dependencies

- #165, #166.
- Enforced upon: #170, #171, #172, #173 (each must pass this harness); reported by #175 (scaling claims are only meaningful against byte-identical outputs).

## Deliverables

1. Normative reproducibility section (docs) stating the invariant, the FP and seed policies, and the forbidden patterns.
2. The byte-equality harness: pinned fixtures × {sequential, parallel × thread counts} × repetitions, comparing bytes, hashes, and certificates.
3. CI wiring (extends the existing deterministic-rebuild gate).

## Acceptance criteria

- The invariant sentence above appears verbatim in the normative docs.
- Byte-identical artifacts across sequential/parallel, thread-count sweep (1/2/4/N), and repeated builds on pinned fixtures.
- Certificate bytes identical across all runs.
- The harness fails on a seeded nondeterminism fixture (negative test: e.g. an intentionally order-sensitive reduction) — a gate that cannot fail is not evidence.
- Four gates green; claim-wording check passes.

## Formal claim classification

- Canonicalization and deterministic output under parallelism: **Guarantee**, status **Structural** (harness-backed; Kani/property where bounded per #132).
- Speedup and scaling: **Empirical Criterion** (owned by #175).

## Test requirements

- **Unit:** tie-breaking and dedup order stability.
- **Property:** canonical form invariant under input-permutation-with-same-content and under chunking variation.
- **Reproducibility:** the full harness above (core deliverable).
- **BDD/Cucumber:** one scenario pinning sequential≡parallel byte equality on a small fixture, per `features/suites/` conventions.
- **Benchmark:** none (that is #175).

## Memory considerations

The harness runs within #169's budget model; memory-constrained configurations are part of the sweep matrix.

## Determinism considerations

This issue *is* the determinism consideration for the chain; every other issue defers to it.

## CPU portability and no-GPU statement

Reproducibility is CPU-native and GPU-free: no CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, tensor accelerators, neural engines, vendor BLAS, or GPU-specific formats anywhere in the harness or the compiled pipeline. Cross-target reproducibility (x86_64 vs AArch64 artifact equality) is recorded as an explicit goal or an explicitly documented exception (target-dependent teacher logprobs are already known to differ per the Gate E macOS pin — that exception is teacher-side, not compiler-canonicalization-side).

## Traceability

- R4G1.md §7 (D2, Gate E); plan §4.1; AGENTS.md determinism invariant; direction PDF §13; monograph (#133); traceability matrix (#137); proof-model entry via #132.

## Related issues and PRs

#21, #58, #59, #122, #129, #132, #157, #165, #166, #168, #169, #170, #171, #172, #173, #174, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

