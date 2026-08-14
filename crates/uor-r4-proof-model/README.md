# uor-r4-proof-model

Executable proof specification and formal-verification harness for the R⁴
holographic graph compiler.

This crate turns the proof obligations in
`docs/r4_holographic_graph_compiler_implementation_plan.pdf` (§25) into
runnable specifications and property checks. It is deliberately decoupled from
the production runtime: the executable spec lives here so later Kani/Verus/Lean
tranches can target the same definitions without the runtime inheriting a proof
tool dependency (plan §4, Phase 10).

## Modules

| Module | Obligation it operationalizes |
|---|---|
| `allocation_proof` | Theorem 1 (Allocation Freedom): counting-allocator harness (`verify_zero_allocation`) used by runtime/test crates to assert allocation-free sections |
| `deterministic_topk_proof` | Deterministic top-K: canonical candidate ordering (highest score, then lowest token id) and its verifier (PDF §23) |
| `range_bounds_proof` | Theorem 4/9 bounded ranges: section-relative packed-range boundary checks |
| `theorem7_proof` | Theorem 7 (Forward/Reverse Index Consistency): reverse entries resolve to canonical edges whose target matches the containing range's node |
| `proof_matrix` | The proof-status matrix: every theorem/assumption mapped to a `ProofStatus` with implementation links — the Gate F record (CI-checked "no 'machine-verified' wording without a checked proof artifact", see `docs/formal_vocabulary.md` §2.1) |

Claim classes, claim statuses, and the mapping from `ProofStatus` to document-level
claim language are defined in `docs/formal_vocabulary.md` (issue #123), which is
normative for how this crate's statuses are cited in documents.

## Usage

Property tests and production test-suites call the `verify_*` functions
directly (see `tests/proof_model_tests.rs`); the proof matrix is the
machine-readable registry of what is proven, assumed, and still open.

The default `full-model` feature enables that complete executable-spec surface.
Kani runs with `--no-default-features`, which retains only the harnesses over
`uor-r4-graph-format` and `uor-r4-graph-runtime`. This keeps native teacher and
compiler dependencies outside the bounded-model-checking graph; Kani never
suppresses or skips their assembly.

## Conventions

- Mirrors the runtime's integer-only semantics — proofs are stated over the
  normative scalar kernel, not floating-point compiler internals.
- Findings are reported as `Result<_, String>` with the violated invariant in
  the message; no panics in verification paths.
- The default executable specification depends on core, compiler, certifier,
  format, runtime, and serde surfaces. The isolated Kani graph depends only on
  format and runtime.
