# External research intake for UOR-R4

Audited 2026-09-03T03:16:18.572373+00:00.

Read-only research triage; source snapshots and reports only in /Users/casey.allard/.local/share/uor-r4/knowledge/audits/2026-09-03/research. No installs, config changes, source changes, tests, experiments, issues or external messages.

Complete Git tree inventories for four named repositories; representative source/document review. Not an exhaustive mathematical or code audit.

## Recommended inclusion

| Repository | Pinned head | License | GitHub size | Inventory | Recommendation |
|---|---|---|---:|---:|---|
| [markrnd87-cmd/NEMESIS-Theory](https://github.com/markrnd87-cmd/NEMESIS-Theory) | `0d106967843c2c96477cf3e57aeff213e7db1c97` | None detected | 52805 KiB | 205 files | LINK_AND_CHRONICLE_ONLY_PENDING_LICENSE_AND_CLAIM_AUDIT |
| [Graph-and-Geometric-Learning/helm](https://github.com/Graph-and-Geometric-Learning/helm) | `7501deca8f413848bfef804be64ce874b72a3cd7` | MIT | 21054 KiB | 10510 files | PINNED_REFERENCE_ADAPTER_CANDIDATE |
| [unicornd47-afk/GoldSnnail](https://github.com/unicornd47-afk/GoldSnnail) | `e8e0f303aa956759343cc14177068dba9ba027bd` | MIT | 88446 KiB | 301 files | SELECTIVE_DESIGN_REFERENCE_NO_ATTENTION_PORT |
| [wilcompute/W33-Theory](https://github.com/wilcompute/W33-Theory) | `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d` | MIT | 361043 KiB | 29843 files | PINNED_CLAIM_SCOPED_REFERENCE_AND_FINITE_OBJECT_CANDIDATES |

GitHub size is the repository metadata field, not a measured working-tree download size. All four public repositories were accessible; no tree response was truncated.

## markrnd87-cmd/NEMESIS-Theory

Document archive; mathematical proposals and historical collaboration context; no standalone executable source or formal proof project in complete file inventory.

GitHub license metadata null; complete tree has no LICENSE/COPYING file. No reuse grant established by this audit.

### Source observations

- 205 files: 165 PDF, 36 DOCX, one README, one TXT, two extensionless files. Five selected PDFs downloaded and text-extracted; two first pages visually inspected.
- Canonical Geometry and Gauge Structure is labeled From Gemini A.I. / Research Core, dated October 30, 2025, addressed to Alex Flom; its E8 quotient is explicitly proposed and its own next steps include identifying the action. Preserve this authorship/proposal provenance.
- The structure-carrying-substrates report names state-space identity, transition fidelity, and primitive interpretation as desired criteria. It does not supply a compiled implementation, benchmark provenance, or machine-checked certificate for its O(d), zero-error, or energy claims.
- The kernel-test PDF contains illustrative Rust snippets, including is_void_quarantined that returns true after a placeholder comment and a carry helper that increments an adjacent digit without proving arbitrary carry propagation. These do not establish the advertised semantic or complexity guarantees.
- The README describes n=8 in Z/(2^n)Z as a 256-dimensional space. An 8-bit word has 256 possible values; scalar cardinality and vector-space dimension need separate typed definitions before reuse.
- Useful hypotheses to chronicle: explicit finite word-state algebra, carrier/interpretation separation, and preservation obligations. Physical, cosmological, ancient-language and higher-dimensional claims supply no measured language-generation evidence in the inspected artifacts.

### Reuse boundary

Create a citation/claim map with repository SHA, document page, attributed author, proposition, assumptions and required falsifier. Extract a fresh typed specification of word-state algebra and carrying law for review; do not vendor archive or copy pseudo-validation into the runtime.

SOURCE_INSPECTION_ONLY; no experiments or proof checking; whole document archive was inventoried but only named representative documents read.

### Pinned source entry points

- [README.md](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/README.md)
- [Canonical Geometry and Gauge Structure.pdf](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Canonical%20Geometry%20and%20Gauge%20Structure.pdf)
- [Technical Report_ Integration of Hypercomplex Geometries as UOR Structure Carrying Substrates.pdf](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Technical%20Report_%20Integration%20of%20Hypercomplex%20Geometries%20as%20UOR%20Structure%20Carrying%20Substrates.pdf)
- [Blueprint for NEMESIS-UOR-Fork Repository.pdf](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Blueprint%20for%20NEMESIS-UOR-Fork%20Repository.pdf)
- [Topological Computation and the Reality Engine_UOR Kernel Test Suite & Continuous Integration Pipeline_.pdf](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Topological%20Computation%20and%20the%20Reality%20Engine_UOR%20Kernel%20Test%20Suite%20%26%20Continuous%20Integration%20Pipeline_.pdf)
- [Formalizing Cayley-Dickson Topology.pdf](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Formalizing%20Cayley-Dickson%20Topology.pdf)

## Graph-and-Geometric-Learning/helm

Executable PyTorch hyperbolic decoder/model code, training and evaluation scripts, paper and checkpoint links; relevant causal-language-model reference.

MIT LICENSE present. Repository includes vendored HyperCore and lm-evaluation-harness; retain upstream notices and inspect per-component provenance for any extraction.

### Source observations

- 10,510 tracked files, largely the vendored evaluation harness (8,239 YAML files). Ingest relevant modules and citations rather than treating every vendored task as new product work.
- helm_d.py explicitly constructs a causal mask and outputs vocabulary logits through a learned mapping. It composes learned Q/K/V projections, Lorentz attention, normalization, residuals and feed-forward layers.
- lorentz_former_conv.py full_attention calculates Lorentz similarity, masked softmax, and a Lorentz centroid. This is ordinary learned causal attention in hyperbolic geometry, with multiplication/floats; it is not a table-only runtime.
- hmla.py has actual low-rank Q/K/V projection code, but the inspected KV cache buffers and assignments are commented out. The README cache-memory claim cannot be assumed to describe a working incremental-cache path at this commit.
- setup_training.sh installs torch 2.6.0 CUDA 12.4 wheels and graph extensions; train_helm_d.sh launches four GPUs with bf16. No turnkey CPU build/performance qualification was established.
- Paper arXiv:2505.24722 v2 (November 6, 2025) and a 100M checkpoint link to Zenodo record 18729608 are present. The 1B link in README is empty. Zenodo browser access returned an internal error, so checkpoint availability/content was not verified.
- No formal-proof source files appear in the full tree. Published experimental claims are external reported results, not reproduced in this audit.

### Reuse boundary

Keep HELM-D as a pinned architectural reference and, only for a named future comparison, wrap its minimal decoder with common token IDs, masked-prefix inputs and logits/NLL outputs. Avoid wholesale train-stack/vendored-harness adoption. Consider HMLA only after a separate working-cache contract and CPU measurement. Preserve existing UOR learned reader/core during the current exposure diagnostic.

SOURCE_INSPECTION_ONLY; no install, training, checkpoint loading, inference or CPU benchmarking.

### Pinned source entry points

- [README.md](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/README.md)
- [LICENSE](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/LICENSE)
- [helm/modules/helm_d.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/modules/helm_d.py)
- [helm/modules/hmla.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/modules/hmla.py)
- [helm/modules/mice.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/modules/mice.py)
- [helm/hypercore/nn/attention/lorentz_former_conv.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/hypercore/nn/attention/lorentz_former_conv.py)
- [helm/hypercore/manifolds/lorentzian.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/hypercore/manifolds/lorentzian.py)
- [helm/hypercore/nn/linear/lorentz_linear.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/helm/hypercore/nn/linear/lorentz_linear.py)
- [lm-evaluation-harness/lm_eval/models/helm.py](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/lm-evaluation-harness/lm_eval/models/helm.py)
- [setup_training.sh](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/setup_training.sh)
- [example/train_helm_d.sh](https://github.com/Graph-and-Geometric-Learning/helm/blob/7501deca8f413848bfef804be64ce874b72a3cd7/example/train_helm_d.sh)

Paper: [https://arxiv.org/abs/2505.24722v2](https://arxiv.org/abs/2505.24722v2); checkpoint link observed, contents unverified: [https://zenodo.org/records/18729608](https://zenodo.org/records/18729608).

## unicornd47-afk/GoldSnnail

Rust experimental SNN/geometry/ARC system with runnable-looking sources and tests; layout/performance reference, not demonstrated general language model.

MIT LICENSE present, copyright 2026 unicornd47-afk; LICENSE.md is empty but LICENSE contains the grant. Cargo package says MIT.

### Source observations

- GitHub dominant language says JavaScript, but complete tree includes 181 Rust files among 301. Cargo.toml is edition 2024/rust-version 1.85, while README still says edition 2021/Rust 1.70+.
- StateArena uses structure-of-arrays and indexed buffers; quaternion helper functions offer straightforward stack/scalar Hamilton-product references. AVX2 uses unsafe x86_64 intrinsics, floating multiplication/FMA and some Vec allocation; it is not portable integer-kernel code.
- src/attention.rs uses score ||q * conjugate(k)||. For standard Hamilton quaternions, norm multiplicativity reduces this to ||q|| ||k||, so unit keys all receive equal scores and orientation cannot influence attention. This is a mathematical source-level derivation, not a measured test.
- The same attention implementation allocates score and weight Vecs per query even in forward_in_place and has no causal mask argument. The in-place label does not establish allocation-free attention.
- poincare.rs exp_map_origin computes tanh(v)/|v| * tanh(|v|), which does not simplify to the stated tanh(v) for nonzero v. project_radius also changes interior values and maps NaN/Inf to zero. Do not copy these semantics as exact geometric identities or fail-closed validation.
- Chat smoke tests register a five-word lexicon, test spike round-trip, buffer serialization and a short transition sequence. They do not establish coherent open-ended generation.
- README reports ARC training 16/400 and SHD 43.4% contrastive versus 42.7% rate baseline and 46.5% softmax MLP. Phase 2 reports 1/400 ARC evaluation (written 0.2%; exact fraction 0.25%) and a no-go. These are reported metrics on different tasks; no UOR transfer or matched performance result follows.
- Committed benchmark JSON gives short-response grammatical_rate 0.3333 and avalanche rate 0.0500; another validation report gives criticality is_critical=0. Do not combine selected favorable rows into a model-readiness claim.

### Reuse boundary

Mine StateArena and buffer ownership/layout patterns only if UOR profiling names a memory/dispatch cost. Compare isolated scalar primitives against a mathematical oracle before extraction, with source/license attribution. No Tauri route, SNN replacement or attention port is justified by this audit.

SOURCE_INSPECTION_ONLY; no cargo build/test, datasets, inference, timing or benchmark reproduction.

### Pinned source entry points

- [README.md](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/README.md)
- [LICENSE](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/LICENSE)
- [Cargo.toml](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/Cargo.toml)
- [src/substrate/mod.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/substrate/mod.rs)
- [src/substrate/avx2.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/substrate/avx2.rs)
- [src/swarm/qlif.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/swarm/qlif.rs)
- [src/geometry/mod.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/geometry/mod.rs)
- [src/geometry/poincare.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/geometry/poincare.rs)
- [src/geometry/quaternion.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/geometry/quaternion.rs)
- [src/attention.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/attention.rs)
- [src/semantics/token_engine.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/src/semantics/token_engine.rs)
- [tests/test_chat.rs](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/tests/test_chat.rs)
- [docs/src/development/benchmark_results.json](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/docs/src/development/benchmark_results.json)
- [docs/src/development/validation_report.json](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/docs/src/development/validation_report.json)
- [docs/src/development/PHASE_2_STATUS.md](https://github.com/unicornd47-afk/GoldSnnail/blob/e8e0f303aa956759343cc14177068dba9ba027bd/docs/src/development/PHASE_2_STATUS.md)

## wilcompute/W33-Theory

Large mixed finite-geometry research archive with exact Python/GAP witnesses, a Lean package, persistent-state reference runtime, conjectures, historical overclaims and correction ledgers.

MIT LICENSE present. Treat external datasets/papers and individual imported artifacts separately before vendoring.

### Source observations

- 29,843 tracked files, 13,862 Python files, 7,799 JSON files, 60 .lean files; tree API not truncated. Full deep proof audit was not attempted.
- The chamber GAP witness reconstructs projective points/lines from F3^4, constructs panel matrices and checks exact polynomial/rank identities. This is a genuine finite-object computation rather than only a list of supplied constants. It was inspected but not executed.
- The Python microVM runtime constructs the same finite geometry and a SHA256-addressed persistent state DAG with addressed send/run, path-copy checkpoints and immutable siblings. It explicitly disclaims Linux/KVM/security-boundary status. Uniform 40-ary sharing represents billions of logical leaves with seven blobs; this is representation compression, not billions of independent active computations or measured throughput.
- formal/ pins Lean and mathlib v4.32.0-rc1. All 60 Lean files were downloaded and lexically scanned; no sorry/admit/axiom occurrence was found, but this does not check elaboration, theorem scope or transitive assumptions.
- Pass806 explicitly formalizes only the diagonal cyclic-group arithmetic half of the two-branch gluing story, leaving the Smith-form reduction outside that module.
- Pass828 assigns coalescence_rank_3 := 10 then proves equality to 10, and uses externally supplied determinant constants. These theorems do not derive ranks/determinants from the graph. Three valuation checks use native_decide.
- Pass1091 contains real generic matrix implications and finite map involution checks, but certificate SHA256 strings are constants, not kernel validation of external tensor files. Pass1106 explicitly says large class actions remain external executable certificates.
- Latest observed Lean CI run 33681447009 is on earlier commit 614c2570563ed123753e2bc06c9d41b28f8f9b7b: kernel-build succeeded; independent-leanchecker failed because runner shutdown produced exit 143 before completion. This is not a proof counterexample, and is not an all-green independent check for audited default head.
- Root verify_w33_full.py supplies many constants and includes test("Both sectors KS non-colorable", True). Its ALL W33 PREDICTIONS VERIFIED banner is not an independent validation of all claims.
- Current correction ledger marks CE2 global closure OPEN, real K3 object not loaded, family-flag alignment REFUTED, and transport/cocycle identification OPEN/conditional. Any ingestion must pair old positive claims with their current correction owners.

### Reuse boundary

Curate one claim-index record per finite object, linking constructor, exact witness, result artifact, theorem statement and correction status. Possible later references: projective canonicalization, exact chamber operators, and immutable DAG path-copy lifecycle. Require an explicit typed map from each F3/projective carrier into a UOR R4/Spin/H4 carrier before claiming shared geometry or learned-language benefit. Do not import the whole archive or use physical/continuum claims as implementation guarantees.

SOURCE_INSPECTION_AND_REMOTE_CI_OBSERVATION_ONLY; no GAP/Python/Lean run or performance measurement.

### Pinned source entry points

- [README.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/README.md)
- [LICENSE](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/LICENSE)
- [RESULTS_VOCABULARY.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/RESULTS_VOCABULARY.md)
- [analysis/W33_CLAIM_STATUS_LEDGER.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/W33_CLAIM_STATUS_LEDGER.md)
- [analysis/PASS6233_6240_scaffold_claim_tier_repair.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/PASS6233_6240_scaffold_claim_tier_repair.md)
- [formal/README.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/README.md)
- [formal/lakefile.toml](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/lakefile.toml)
- [formal/lean-toolchain](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/lean-toolchain)
- [formal/W33.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33.lean)
- [formal/W33/Pass806TwoBranchGluing.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33/Pass806TwoBranchGluing.lean)
- [formal/W33/Pass828CoalescenceArithmetic.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33/Pass828CoalescenceArithmetic.lean)
- [formal/W33/Pass1091FrameOrbitalIntertwiner.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33/Pass1091FrameOrbitalIntertwiner.lean)
- [formal/W33/Pass1106CliffordFirewallCarrier.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33/Pass1106CliffordFirewallCarrier.lean)
- [analysis/w33_pass4324_4327_chamber_hecke_hashimoto.g](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_pass4324_4327_chamber_hecke_hashimoto.g)
- [analysis/BT4324_BT4334_CHAMBER_HECKE_AND_AUDITED_CORRECTIONS.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/BT4324_BT4334_CHAMBER_HECKE_AND_AUDITED_CORRECTIONS.md)
- [analysis/w33_fractal_microvm_runtime.py](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py)
- [analysis/w33_fractal_microvm_routing.g](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_routing.g)
- [docs/W33_FRACTAL_MICROVM_RUNTIME.md](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/docs/W33_FRACTAL_MICROVM_RUNTIME.md)
- [tests/test_w33_fractal_microvm_runtime.py](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/tests/test_w33_fractal_microvm_runtime.py)
- [verify_w33_full.py](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/verify_w33_full.py)

## Lean tooling and proof sequence

Latest Lean stable release observed v4.33.1; mathlib current master requires v4.34.0-rc2; W33 formal package pins v4.32.0-rc1. Pin a matching Lean/mathlib pair; do not mix independently latest revisions.

oOo0oOo/lean-lsp-mcp is an independent MIT research-tool integration, not the Lean kernel itself. It provides goals, diagnostics, hover, local search and optional external theorem search. Prefer a project-scoped local stdio instance after a small Lean project builds; remote search sends queries externally. No configuration or execution performed.

- [leanprover/lean4](https://github.com/leanprover/lean4) at `19c79593c47bb8dd3371327c08fc80775d8488af` (Apache-2.0; 8960 GitHub stars observed).
- [oOo0oOo/lean-lsp-mcp](https://github.com/oOo0oOo/lean-lsp-mcp) at `bb176c58a4f895061561685318e92b8db446f1b5` (MIT; 488 GitHub stars observed).
- [leanprover-community/mathlib4](https://github.com/leanprover-community/mathlib4) at `c4a007f4389f67c1f224c4b87984a3470754b53f` (Apache-2.0; 3989 GitHub stars observed).

The following proof order is proposed; no proof implementation or checker run occurred.

1. Define the exact word semantics: width w, BitVec w/Fin(2^w), wrapping arithmetic, complement, shifts/rotates and their bounds. Prove the representation link to ZMod(2^w). Cardinality is not dimension.
2. Prove complement and successor identities, e.g. -bnot(x)=x+1 modulo 2^w, involutions and only the actual used bit-operation laws. Record overflow, shift-width and signedness conventions.
3. Define a finite frame registry with exact matrices or linear isometries. Prove orthonormality/invertibility and registry lookup/admission; keep labels separate from frame values and distinguish exact algebraic coefficients from floating approximations.
4. For E_s(v)=B_s v, T_ds=B_d B_s^T and D_d=B_d^T, prove D_d(T_ds(E_s(v)))=v; composition T_ed T_ds=T_es, identity/inverse, norm and inner-product preservation under explicit orthonormality assumptions.
5. Prove transported weighted pooling commutes with common-frame decoding. With fixed nonnegative weights, prove D=norm(sum_i a_i delta_i) <= A=sum_i a_i norm(delta_i), changed-frame mass 0<=M<=1, and zero-mass/zero-displacement cases. This is directly relevant to the next exposure/cancellation diagnostic.
6. Prove causal-prefix masking and exact-real score/softmax invariance only for the specified unchanged Q/K/V/transport operator order. Carry the consequences to deterministic logits under explicitly identical downstream maps; do not infer empirical language quality.
7. Specify floating-point refinement separately: finite inputs, rounding modes, accumulation order, orthogonality defect and error propagation. Formal real identities alone do not certify current f64/f32 tolerances. Keep measured bounds labeled empirical until refinement is proved.
8. Connect the theorem definitions to concrete serialized tables and Rust functions through a checkable manifest/decoder and bounded implementation obligations. A hash string or a constant equality alone does not prove the bytes implement the model. Audit imported theorem axioms and reject placeholders.

A completed formal layer would establish its stated algebraic and implementation properties. Learned language behavior, transfer, useful long-context state, and CPU performance still require separately declared empirical evidence.

Primary tooling sources:

- https://lean-lang.org/doc/reference/latest/
- https://lean-lang.org/doc/reference/latest/Basic-Types/Bitvectors/
- https://github.com/leanprover/lean4
- https://github.com/leanprover-community/mathlib4
- https://leanprover-community.github.io/mathlib4_docs/Mathlib/Algebra/Quaternion.html
- https://leanprover-community.github.io/mathlib4_docs/Mathlib/LinearAlgebra/Matrix/Orthogonal.html
- https://github.com/oOo0oOo/lean-lsp-mcp

## Recommended chronicle record

For each imported idea retain: repository and exact commit, file and page/line, author attribution as printed, source license, typed mathematical object and conversion map, claim tier, theorem/experiment dependencies, reproduction status, known correction owner, intended product decision, and chosen disposition. For NEMESIS retain links and short attributed notes until license is resolved. For executable candidates retain a minimal dependency slice, upstream notice and adapter boundary instead of importing whole archives.

Maintain distinct evidence states: SOURCE_INSPECTED, REPORTED_EXTERNAL, RUNNABLE_SOURCE_UNTESTED, FORMAL_STATEMENT_INSPECTED, LOCALLY_REPRODUCED, and IMPLEMENTATION_REFINED. This audit establishes only the first four where described.

## Files produced

- `research-triage.json`: machine-readable findings and suggested proof sequence.
- Four `*.inventory.json` files: full pinned Git tree metadata.
- Three Lean/tool `*.metadata.json` files: pinned upstream metadata.
- `snapshot-manifest.json`: local snapshot file paths, sizes and SHA256 identities.
- `snapshots/`: selected source files, all W33 Lean files, representative PDFs and text extracts.
- `w33-lean-workflow-observation.json`: three latest observed external formal workflow states.
