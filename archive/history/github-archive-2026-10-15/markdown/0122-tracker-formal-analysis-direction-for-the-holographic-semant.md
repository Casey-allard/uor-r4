# #122 — TRACKER: Formal analysis direction for the holographic semantic operating system

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:02:23Z
- **Updated:** 2026-07-25T13:45:55Z
- **Closed:** 2026-07-25T13:45:55Z
- **Labels:** graph-compiler
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/122
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Source: `docs/hologram_formal_analysis_direction.pdf`

This tracker turns the PDF's 17-section formal-analysis direction into an implementation and evidence program. The intended architecture is a compiler that transforms trained neural behavior into an immutable, bounded, transformerless semantic graph; runtime inference remains deterministic graph propagation over compressed semantic state.

The work keeps three claim classes separate:

- **Definitions:** architectural objects and mathematical state.
- **Objectives:** quantities optimized by the offline compiler.
- **Guarantees:** structural properties proven about the artifact and runtime.

## Complete PDF coverage

- **§1 Definitions/objectives/guarantees:** #123
- **§2 Semantic state manifold:** #124
- **§3 Graph dynamics:** #124, #131
- **§4 One graph/multiple edge algebras:** #125
- **§5 Mathematical hologram:** #126
- **§6 Predictive entropy:** #127
- **§7 Semantic compression:** #129, #136
- **§8 Information bottleneck:** #127
- **§9 Explicit graph invariants:** #135
- **§10 Thinking versus language:** #134
- **§11 Compiler as core contribution:** #128, #129, #130
- **§12 Future-state optimization replacing intent:** #131
- **§13 Structural proofs:** #132
- **§14 Overall direction/project identity:** #122, #133
- **§15 Project description:** #123, #133
- **§16 Monograph structure:** #133
- **§17 Immediate research sequence:** #122, #137

## Formal-direction issues

1. #123 - formal vocabulary and claim classes.
2. #124 - semantic state model and typed graph dynamics.
3. #125 - shared node space and edge algebras.
4. #126 - holographic encoding and reconstruction metrics.
5. #127 - predictive entropy and information-bottleneck objectives.
6. #128 - unsupervised interventions and counterfactual probes.
7. #129 - reference floating-point compiler and IR.
8. #130 - Boolean/integer/fixed-point lowering.
9. #131 - bounded future-state optimization and planning.
10. #132 - structural graph and planner proofs.
11. #133 - formal monograph and implementation specification.
12. #134 - semantic reasoning versus language emission.
13. #135 - graph invariant ownership and loader validation.
14. #136 - semantic compression and rate-distortion.
15. #137 - PDF-to-implementation traceability matrix.

Existing #11–#34 remain the lower-level R4G1, runtime, certificate, evaluation, threat-model, lifecycle, and proof infrastructure backlog. They are intentionally reused rather than duplicated.

## Dependency guidance

- Start with #123, #124, and #137.
- #125 depends on the R4G1 and transition work in #11 and #18.
- #126 and #128 define evaluation evidence for #127, #129, and #136.
- #127 and #136 define the compiler objective and measurement contracts.
- #129 implements the reference compiler; #130 lowers its output and extends #14, #15, and #19.
- #131 depends on #124 and #125; #134 defines its language boundary.
- #135 and #132 consume the bounded interfaces from #125, #130, and #131.
- #133 is maintained throughout and finalized after the vocabulary, compiler, and proof interfaces stabilize.

## Tracker definition of done

- Every PDF section maps to an issue, implementation/document location, and evidence artifact.
- Definitions, optimization objectives, empirical claims, and structural guarantees are labeled separately.
- The reference compiler is evaluated independently from Boolean lowering and deployed runtime execution.
- The runtime accepts only artifacts passing existing validation and proof gates.
- No semantic-equivalence or machine-verified claim is made without corresponding empirical or formal evidence.

## Comments (4)

### comment 5072832882 — 2026-07-24T17:49:56Z — auser

## Status update (2026-07-24)

### Landed
- **#123** formal vocabulary — merged via PR #139 (merge commit `624a1cf`). `docs/formal_vocabulary.md` v0.1.0 is now the normative claim-language spec (claim classes, statuses mapped to `ProofStatus`, core notation, term discipline), with `scripts/check_claim_wording.py` enforcing the wording rule in CI.
- **#124–#131** — closed (semantic state model, edge algebras, holographic encoding, predictive entropy/IB, counterfactual probes, reference compiler IR, Boolean lowering, future-state planning).

### In flight — all six reviewed, changes requested
All remaining formal-direction issues are assigned to @Casey-allard with open PRs. Every PR passes the four gates + κ-reproduction on a merge preview, but all six received **Request changes** reviews today (2026-07-24) for unmet acceptance criteria:

- **#132 / PR #151** — proof-model expansion covers ~4 of 8+ obligation families; differential test is a constant-closure tautology; no CI wiring for status drift.
- **#133 / PR #152** — conflicts with main; 9 absolute `file://` links; 4/9 module links dead; all proof-status rows "Verified" without evidence artifacts; dependency graph + review checklist missing.
- **#134 / PR #153** — conflicts with main; bypasses the #124/#129/#131 machinery; certification dimensions hardcoded; packed/runtime API distinction absent.
- **#135 / PR #154** — conflicts with main; `validate_graph_structure` not wired into the stage-2 loader; 1 of 6 boundary fixtures; compiler-side failures and proof-matrix criteria unmet.
- **#136 / PR #155** — synthetic curve generator (no corpus, no teacher, no CIs); `optimal_tradeoff_depth`/`is_certified` hardcoded; report IDs not content-addressed.
- **#137 / PR #156** — conflicts with main; 8/15 evidence links dead; open issues #132–#136 marked `Verified`; PDF sections renumbered vs. the canonical 17-section mapping in `docs/hologram_formal_analysis_direction.md`; verifier certifies its own hardcoded copy.

### Tracker DoD notes
- Claim-class separation (definitions / objectives / guarantees / empirical) is now enforceable repo-wide via `docs/formal_vocabulary.md` + the CI wording gate — that tracker bullet has teeth as of #139.
- "No semantic-equivalence or machine-verified claim without evidence" is the exact failure mode in PRs #152/#155/#156 (hardcoded `is_certified`/`Verified`); the reviews call this out against the vocabulary spec.
- The §-coverage table in this tracker remains accurate; once #132–#137 land, the remaining DoD items are the traceability matrix (#137) and final monograph (#133).

### comment 5073103419 — 2026-07-24T18:21:18Z — auser

## Status update — multiplication-free inference chain (#157–#161)

Five new issues now track the strict multiplication-free inference program for the deployed hot path. Review of the existing backlog confirmed no overlap: #11–#34 cover R4G1/runtime/certificate infrastructure, #130 (closed) covers Boolean/fixed-point *lowering*, and plan §7 T2 already names "census + source scan + disassembly audit" — but the disassembly leg had no implementation and no tracking issue, and the operation contract itself existed only as prose (plan §6, plan PDF §17, GLOSSARY "Normative kernel") with the P-4 scan covering a single file.

### The chain (dependency order)

1. **#157** — normative multiplication-free inference contract: exact inference boundary, allowed/forbidden operation lists, machine-readable contract module, crate ownership, compiler-generated-arithmetic policy. **Definition + structural Guarantee.**
2. **#158** — fixed-point scoring semantics: residual taxonomy, accumulation order, saturation, sentinels, deterministic tie-breaking, overlap residualization. Depends on #157. **Structural Guarantee for arithmetic; Empirical for fidelity.**
3. **#159** — allocation-free packed inference kernels over immutable graph arrays (nine kernels; removes the recursion in `engine.rs:393`, the multiplies in `cayley_dickson.rs:17,27`, and the `unreachable!` panics in `runtime.rs:580,621`). Depends on #157, #158. **Structural Guarantee (Gate B).**
4. **#160** — CI disassembly audit of release machine code (x86_64 scalar/POPCNT/AVX2, aarch64 NEON, portable fallback; per-arch opcode policies; must-fail negative fixture). Depends on #157, #159. **Witnessed Guarantee; completes T2.**
5. **#161** — operation and memory-cost certificates: declared bounds vs. observed counts vs. disassembly-verified absence vs. empirical performance, kept as separate CID'd components; replaces honor-system attestation booleans. Depends on #157–#160. **Witnessed/Empirical.**

### Interactions with the open formal-direction issues

- **#132** gains obligation families from #157 (operation-set conformance) and #158 (safe fixed-point arithmetic); #160 supplies the evidence that upgrades the no-multiplication claim to Structural in the proof matrix.
- **#135** should add an "operation-set conformance" row to the invariant ownership matrix (#157 names it).
- **#133** (monograph) should cite #157 in the runtime-execution/resource-contract chapter and #161 in the certification chapter.
- **#137** (traceability matrix) needs rows for #157–#161 under PDF §1 (guarantee inventory) and §13 (structural proofs) once the matrix lands.

No implementation is claimed here — these issues define and sequence the work only.


### comment 5073195512 — 2026-07-24T18:30:51Z — auser

## Ownership matrix — normative inference chain #157–#161 (updated 2026-07-24)

The five issues now own the full normative contract: multiplication-free, **CPU-only**, and **zero-heap-allocation** inference. Bodies were expanded in place (no new issues created — no duplicates). Every requirement has exactly one owning issue:

| Requirement | Sole owner | Supporting issues |
|---|---|---|
| Inference boundary (activity list, crate ownership) | **#157** | — |
| No-multiplication contract (forbidden instruction classes, legal address generation) | **#157** | #160 (enforcement), #158 (scoring arithmetic) |
| CPU-only normative target; GPU/accelerator non-goals | **#157** | #160 (dependency gate), #161 (declared-zero GPU fields) |
| Zero-allocation lifecycle definition (six phases, init-strength decision) | **#157** | #159 (kernel code), #160 (failing-allocator gate) |
| Fixed-capacity memory model / caller-owned API shape | **#157** (definition) | #159 (implementation) |
| Fixed-point and residual scoring semantics | **#158** | #19, #130 (landed precursors) |
| Packed no-alloc CPU inference kernels (nine kernels) | **#159** | #17, #25, #28 (landed precursors) |
| Disassembly auditing (release machine code, arch matrix) | **#160** | — |
| Allocation auditing (failing allocator, no-`alloc` build gate) | **#160** | #13 (landed census) |
| GPU dependency / Cargo feature auditing | **#160** | — |
| Source/type-level enforcement beyond P-4 | **#160** | #157 (contract module it enforces) |
| Operation certification | **#161** | #14, #20, #27 (landed precursors) |
| Allocation / CPU portability certification | **#161** | #160 (evidence source) |

### Dependency order (unchanged)

#157 → #158 → #159 → #160 → #161

### Newly surfaced requirements now tracked

- **CPU-only normative execution** was completely untracked (zero issue/doc hits for GPU/CUDA/Metal/OpenCL/accelerator/CPU targeting). Workspace `Cargo.toml` is currently clean of GPU/tensor/BLAS dependencies; #157 §6 makes the stance normative and #160 Layer 4 gates it.
- **No-`alloc` build configuration**: `uor-r4-graph-runtime` links `alloc` unconditionally today; #157 §9 sets the isolation goal and #160 Layer 3 adds the CI gate (a `no_std` build with `alloc` enabled is explicitly not zero-allocation evidence).
- **Allocator-failure testing across all ResolutionStatus paths** (#160 Layer 3, extending the #13 census beyond two paths).
- **Legal address-generation exception** and the expanded forbidden instruction families (widening/high-half multiply, `madd`/`msub`, dot-product and matrix extensions) are now spelled out in #157 §§3-4.

### Interactions with open formal-direction issues

- #133 (monograph): runtime-execution chapter cites #157; certification chapter cites #161.
- #135 (invariant ownership): gains three rows — operation-set conformance, CPU-only platform conformance, zero-allocation lifecycle.
- #137 (traceability matrix): rows for #157–#161 under PDF §1 and §13 (see comment on #137).
- #132 (merged via PR #151): obligation families for operation-set conformance and safe fixed-point arithmetic now have their normative sources in #157/#158.

No implementation is claimed — these issues define and sequence the work only.


### comment 5073320073 — 2026-07-24T18:45:00Z — auser

## Status update — deterministic compiler parallelism chain (#165–#175)

Eleven new issues track deterministic multicore CPU parallelism for the **offline compiler** (Rayon-style CPU concurrency; the compiler may use multiplication, floats, heap, and threads — the #157 contract governs production inference only). Review confirmed no prior ownership: zero issue hits for rayon/executor/thread-count/oversubscription; `rayon` is declared-but-unused in `uor-r4-graph-compiler` (1.8) and `uor-r4-graph-certify` (1.10) with version drift; precursors are #21 (sequential rebuild test), #59 (content-addressed shards), #129 (ordered reductions), and the existing sequential `--memory-budget` in `induction.rs`.

### Ownership matrix (each concern has exactly one owner)

| Concern | Owner |
|---|---|
| Executor abstraction (sequential + Rayon) | **#165** |
| Stage classification / parallelization DAG | **#166** |
| Reproducibility invariant + byte-equality harness | **#167** |
| Thread-pool / `--jobs` / oversubscription policy | **#168** |
| Memory budget + backpressure | **#169** |
| Observation/trace/evaluation shard parallelism | **#170** |
| Clustering / region induction / graph discovery | **#171** |
| Routing-program / XOR-polynomial synthesis | **#172** |
| Emission / residual / certificate compilation | **#173** |
| CPU-only compiler dependency + feature audit | **#174** |
| Scaling benchmarks + certificates | **#175** |

### Dependency order

#165 (executor) → #166 (stage DAG) → #167 (reproducibility invariant) → #168 (jobs) + #169 (memory) → #170, #171, #172, #173 (parallel stages; #171→#172→#173 ordering) → #174 (dependency audit) → #175 (benchmarks measure everything above).

### Notes

- The chain-wide determinism rules are normative in #167 and restated per issue: no canonical IDs during parallel discovery, no completion-order semantics, no shared mutable accumulation, no unordered-map iteration into canonical output, no order-sensitive FP reductions, thread count never influences topology or bytes; canonicalization precedes IDs/offsets/hashing/signing.
- No-GPU is restated in every issue: the compiler must not require CUDA/ROCm/Metal/OpenCL/Vulkan/WebGPU/DirectML/SYCL/GPU runtimes/tensor accelerators/vendor BLAS; external teacher backends may be GPU but correctness and supported operation must not depend on them (#170 boundary, #174 audit).
- Relationship to the inference chain: #157's CPU-only stance is the policy source; #160 owns the runtime-side dependency audit, #174 owns the compiler-side twin — distinct dependency graphs, explicitly non-overlapping.
- #162–#164 (inference contract/scoring/kernel children, created concurrently by another workflow) are unrelated to this chain; no interaction required.

No implementation is claimed — these issues define and sequence the work only.



---
_Comment/review ids in this file: 5072832882, 5073103419, 5073195512, 5073320073_

