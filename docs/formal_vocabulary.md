# Formal Vocabulary, Notation, and Claim Classes

- **Version:** 0.1.15
- **Status:** Normative for all new specification, plan, proof-model, and certificate text.
- **Source:** `docs/hologram_formal_analysis_direction.pdf` §§1, 7, 13; tracker
  [#122](https://github.com/UOR-Foundation/uor-r4/issues/122); issue
  [#123](https://github.com/UOR-Foundation/uor-r4/issues/123).
- **Related:** terminology for existing graph-compiler concepts lives in
  `docs/transformerless/GLOSSARY.md`; this document governs *claim language* and
  mathematical notation. Where the two overlap, this document wins for claim
  classification and the glossary wins for structural terms. The route-scope
  decision is [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md),
  and experiment policy is
  [Geometric Intelligence Evaluation](geometric_intelligence_evaluation.md).

This spec separates the statement classes that the research notes previously mixed —
architectural definitions, compiler optimization objectives, empirical certification
claims, and structural runtime guarantees — and fixes the symbols that recur across
the compiler, artifact, certificate, and proof model.

## 1. Claim classes (role of a statement)

Every equation and every normative mathematical statement in repository documents
MUST carry exactly one of these role labels, written inline in bold
(`**Definition**`, `**Objective**`, `**Guarantee**`, `**Assumption**`,
`**Empirical Criterion**`):

| Label | Meaning | Where it may appear |
|---|---|---|
| **Definition** | Introduces an architectural object or mathematical term. True by convention; never "proven". | Anywhere |
| **Objective** | A quantity the offline compiler attempts to optimize. Never a runtime invariant, never a theorem. | Compiler-side docs/code only |
| **Guarantee** | A structural property of the compiled artifact or runtime. Requires a claim status from §2 and a proof artifact, witness, test, or explicit `Unproven` record. | Artifact spec, runtime docs, proof model |
| **Assumption** | A condition a proof or certificate requires but the implementation does not itself establish. | Proofs, certificates |
| **Empirical Criterion** | A measured property with a declared distribution, protocol, sample count, and uncertainty. Never stated as a proof. | Certificates, evaluation docs |

Design aspirations are not theorems: an optimization target (`arg max …`) is an
**Objective**, and a measured approximation (`P_θ(a|c) ≈ P_θ(a|Z(c))`) is an
**Empirical Criterion** until the protocol behind it is declared.

## 2. Claim status (evidence behind a statement)

Every **Guarantee** and every **Empirical Criterion** MUST also carry one of these
statuses. The statuses are the machine-readable vocabulary of the proof-status
matrix (`crates/uor-r4-proof-model/src/proof_matrix.rs`); the mapping is normative.

| Status | Meaning | `ProofStatus` mapping |
|---|---|---|
| **Structural** | Established by construction or by a named machine-checked test, fuzz target, or formal proof artifact that has actually run for the declared evidence. | `Verified` |
| **Witnessed** | Established per execution by a bounded, replayable witness an independent verifier replays without the teacher. | `Verified` (witness path) |
| **Empirical** | Measured on a pinned corpus with protocol, confidence intervals, and provenance identifiers, per `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`. | `DifferentialPass` |
| **Assumed** | Required by a proof or certificate but not established by the implementation. | recorded as an assumption entry |
| **Unproven** | Asserted as a goal but currently without evidence. | `Unverified` (blocks `verify_all`) |

`ExecutableSpec` in the proof matrix means the obligation has a runnable
specification but no activated binding check yet; documents must label such claims
**Unproven** (or **Assumed** where applicable), never **Structural**.

### 2.1 Wording rule (automated check, dormant by default)

The prohibited phrases are **"machine-verified"**, **"machine verified"**, **"exact equivalence"**, **"exact teacher equivalence"**, and **"provably equivalent"**. They are prohibited in
normative documents (`docs/**`, crate READMEs) unless the same line links a proof
artifact or certificate, or the phrase is explicitly disavowed on that line.
`scripts/check_claim_wording.py` enforces this when explicitly activated (see §6). The project does not
claim exact teacher equivalence (disallowed per this rule), does not claim
human-level reasoning, and does not treat plausible language output as evidence of
coherent internal state transitions.

## 3. Core notation

Symbols are listed with their claim role and their concrete Rust binding. Entries
marked *compiler/reference-only* have no deployed-runtime representation yet; the
runtime path for them lands in the issues noted.

| Symbol | Role | Meaning | Rust / artifact binding |
|---|---|---|---|
| `x_t^ℓ` | **Definition** | Hidden/residual state at token position `t` and decoder layer `ℓ`. | Active geometric-decoder/source-control state; exact binding lands under #950/#952. |
| `q_t^ℓ, k_i^ℓ, v_i^ℓ` | **Definition** | Learned geometric query, causal prefix key, and value contribution. These predictive roles are distinct from immutable R4G1 route codes, addresses, and CIDs. | The #950/#951 G0/G1 mixer is a retained negative comparator. The current compiler-side reference is `DirectCausalGeometricAttentionR4V1` under ADR-0005; its paired-H4/E8 hierarchy binding remains unimplemented. |
| `p_t, e_t, N_t` | **Definition** | Registered prime route atom; canonical unordered semiprime transition expert `e_t = p_(t-1)*p_t`; and bounded ordered n-let. If adjacent atoms repeat, `e_t = p_t^2` is retained. The square-free case is the subset `p_(t-1) != p_t`. `N_t` retains order and a multiplicity-preserving sorted factor multiset rather than relying on an overflowing numeric product. | Schema-2 source-free manifest records under #958. This binding establishes representation and deterministic recall substrate only. |
| `Gamma = (gamma_0,...,gamma_(m-1))`, `theta_j(p)`, `delta_theta_j` | **Definition** | `Gamma` is a finite ordered, revisioned zeta-ordinate grid. `theta_j(p) = wrap(gamma_j*log(p))`; `delta_theta_j` is the declared local phase difference. | Compiler/reference math and bounded compiled signatures under #958. Critical-line use is an **Assumption**, not a proof of RH; no serving or capability claim follows. |
| `s_t in S3 subset R4`, `h(s_t) in S2 subset R3`, `tau_t in S1` | **Definition** | `s_t` is the full normalized local spin state, `h` is its many-to-one Hopf observation, and `tau_t` is retained quantized fiber/transport phase (“torsion”). `h(s_t)` alone is not a rebuild identity. | Quantized source-free route records and experimental bounded lookup under #958; no physical or quantum interpretation is claimed. |
| `theta=atan2(sin(theta),cos(theta))`, `a(theta)=sin(theta)^2`, `chi(theta)=sign(sin(theta))`, `pol(theta)=sign(cos(theta))` | **Definition** | S3/R3 trigonometric transition contract. `(sin=+/-1,cos=0)` has activation `1`; `(sin=0,cos=+1)` is continuous-null activation `0`. Chirality and, where needed, cosine polarity preserve orientation/antipodes. `tan(theta)` is a local chart only. | At a tangent pole or declared null boundary, the architecture switches angle/cotangent chart and records a signed quarter-turn phase plus torsion shift instead of dividing by zero or terminating. Semantic value is **Unproven**. |
| `m_E=sqrt(2)`, `m_C=2i`, `m_R in [0,2]` | **Definition** | Typed least-cost adapter markers: Euclidean orthogonal-unit chord, complex/discrete antipodal displacement, and declared normalized Riemannian/chord score interval. They are not literal equalities of Euclidean, complex, discrete, and Riemannian domains. | A target-specific cost profile binds chart, units, orientation, quantization, error bounds, canonical tie-break, and conversion witness before choosing the cheapest faithful operation. |
| `r_t = a_t + b_t*phi in Z[phi]` | **Definition** | Direction-preserving golden radial shell. `phi:(a,b)->(b,a+b)` and `phi^-1:(a,b)->(b-a,a)`. Repeated steps have Fibonacci recurrence, which does not establish semantic value. | Exact coefficient updates and manifest chart profile under #958; product use remains separately qualified. |
| `I1, I2, IS` | **Definition** | Bounded causal indexes keyed by last route, ordered previous-plus-last route, and ordered sentence-route identity. | Incremental identity, exact rows, and an off-serving geometric-attention experiment exist under #958. Exact hits are recall; attention and product claims require the evaluation policy. |
| `e^(i*pi)+pi^0 =_bridge 0^0`, `b_0` | **Definition** | Typed zero/identity transition. `ContinuousNull` preserves the complex cancellation as `0`; `DiscreteEmptyProduct` phase-shifts/retypes the seam into the discrete identity `1`. `=_bridge` is a domain-transition operator, not ordinary numerical equality. The selected tag is part of route identity and deliberately exposes the `-1 / 0 / +1` landmarks without assigning two simultaneous values in one untyped calculation. | `ZeroPowerBridge` and schema-2 manifest binding under #958 select the boundary value; the full phase-shift transition remains an architectural contract for the route hierarchy. |
| `C_lex` | **Definition** | Deterministic lexical codec from pinned input bytes/tokenization to registered route atoms and back to output bytes where decoding is defined. Normalization, unknown-unit behavior, tokenizer CID, registry, and codec version are part of its identity. | Product binding remains **Unproven** until a provider-free serving path exercises the same codec end to end. |
| `B_ico : Lambda_E8 ~=_Z I -> (x,x') in R4 ⊕ R4`, `Phi_E8 = H4 ⊕ φH4` | **Assumption** | **Project shorthand:** `E8 = H4 × H4`. The load-bearing implementation contract realizes that conceptual identity through the chosen icosian presentation: the E8 lattice and icosian ring are identified as `Z`-modules, one state is represented by golden/Galois-coupled R4 points, and the declared 600-cell folding is `H4 ⊕ φH4`. This is a typed construction, not a claim that `R4 = E8`. | Basis, glue/parity rule, conjugation, scale, orientation, root order, and inverse witness must be kappa-bound. Architectural load-bearing is assumed; held-out advantage remains unproven. |
| `kappa(X)` | **Definition** | Canonical content identity of the schema/provenance-bound byte envelope `X`. Equality identifies equal canonical envelopes under the named digest contract; digest distance has no routing meaning. | Manifest/artifact CIDs. Kappa is integrity and provenance, not a semantic code. |
| `R_t^q`, `q in {local,sentence,paragraph,conversation,global}` | **Definition** | Identity-scoped, causal route accumulator at hierarchy scope `q`. A parent scope commits to ordered child identities and bounded geometric summaries; it does not rescan all descendant tokens at query time. | ADR-0004 contract; scopes beyond the current local/sentence substrate are **Unproven**. |
| `T_t^q = (v_session, w_window, E_proj, F_shared, c_res, alpha_Hopf, B_ico)` | **Definition** | Bounded transported trajectory/harmonic summary for scope `q`: session hypersphere vector, winding/window state, projection energy, shared-prime factors, cosine resonance, accumulated Hopf phase, and paired-H4/E8 coordinate. It is updated incrementally from the full transported path, not reconstructed from only the last route. | Ancestor routing evidence motivates these fields; current held-out locality when exact hierarchy keys miss is an **Empirical Criterion**, status **Unproven** until reproduced. |
| `W_cov` | **Definition** | Bounded coverage witness recording codec/address coverage, hierarchy rows read and hit/miss, candidates before/after admission, controls, selection or abstention, and artifact identities. | Coverage establishes reachability only; capability status comes from a separate empirical record. |
| `A_recall`, `A_geo` | **Definition** | `A_recall` retrieves a stored continuation by exact/backoff route identity. `A_geo` selects or orders admitted causal support using declared geometric terms and must be load-bearing against matched controls on anti-recall inputs. | Exact-row recall is implemented substrate. Geometric-attention advantage is an **Empirical Criterion**, currently not implied by implementation presence. |
| `P_(i -> t)` | **Definition** | Artifact-declared frame connection that moves a causal key or value from its local R4/S3 tangent frame into the current query frame before comparison or aggregation. | The bounded ADR-0005 scaffold uses an orthogonal `H4FrameConnection`. It is not called Levi-Civita or shortest-geodesic parallel transport without a separate proof. |
| `B_c(g)`, `theta_x^r` | **Definition** | `ConnectionGaugeCovarianceV4` represents each predictive role by the same three local coefficients `theta` in a declared tangent frame `B_c(g)` and transports them by `B_c(d) transpose(B_c(s))`. | V3 rejected the mixed-gauge ambient-R4 projection combination, not the exact H4 group law. V4 Phase I passed construction covariance, all 120 frames/14,400 ordered pairs, and finite-difference gradients. Its held-out population, labels, and verdict remain `NOT_RUN`. |
| `alpha_(t,i)`, `R_t` | **Definition** | Compiler-side direct-attention reference: `alpha_(t,i)` is the stable causal-softmax normalization of `<q_t, P_(i -> t) k_i>/sqrt(d)` for `i <= t`, and `R_t = sum_i alpha_(t,i) P_(i -> t) v_i`. | Implemented only as the bounded offline `DirectCausalGeometricAttentionR4V1` reference under #973. Softmax and the all-prefix scan are not deployed-runtime mechanisms. |
| `A_M`, `K_hat`, `N_t`, `Z_t` | **Objective** | A finite fiber-aware resonance amplitude, its nonnegative normalized kernel `K_hat = floor + abs(A_M)^2`, and the recurrent transported value-numerator and normalization-denominator mode sums that replace dense all-pairs weighting. | ADR-0005 and the #973 multi-resonance reuse audit define the next contract. The normalized sieve, recurrent factorization, and H4/Q29/integer lowering are `NOT_IMPLEMENTED` until separately measured. |
| `I_geo` | **Definition** | Causal inference step mapping observed prefix plus bounded hierarchy state to next-token scores/selection and updated state, followed by lexical decoding. | A support trace alone is not inference or coherent generation. |
| `Corr(D)`, `Abst(D)` | **Empirical Criterion** | Correctness and abstention on declared distribution `D`: correctness uses an independent oracle/constraint/source and is reported both conditional on answered cases and over all cases; abstention is a typed outcome, never silently scored correct. | Protocol and denominators are required by `geometric_intelligence_evaluation.md`. |
| `Reason(D)` | **Empirical Criterion** | On anti-recall tasks from `D`, typed intermediate route transitions preserve constraints, compare alternatives or counterfactuals, and reach an independently checkable conclusion. | Fluent text, recalled answers, teacher agreement, or non-zero geometric activity is insufficient evidence. |
| `Serve_pf` | **Definition** | Provider-free serving: the evaluated process performs no runtime call to Ollama, a cloud model, teacher endpoint, or other generative provider; all required artifacts and decoding are local and pinned. | Does not imply transformerless, geometry-only, multiplication-free, correct, or production-ready. |
| `M_R4^ℓ` | **Definition** | Declared bounded causal geometric mixing operator at layer `ℓ`; its metric, chart, support rule, transport, and aggregation must be frozen by the owning issue. | G0/G1 and ADR-0003 are historical comparators. ADR-0005 is the current direct-reference, resonance-replacement, and recurrent-lowering contract; a deployed realization is `NOT_YET_IMPLEMENTED`. |
| `d_R4^ℓ` | **Definition** | Declared compatibility or least-energy score over factor overlap, prime-gap phase delta, R4/S3 transport, Hopf observation, and torsion. | Normative G1R objective under #958; the score and its selected-support trace are `NOT_YET_IMPLEMENTED`. The learned `d_R4^ℓ(q,k)` from #950/#951 remains a negative comparator. |
| `N_t^ℓ` | **Definition** | Bounded causal neighborhood of prior route and memory states selected before value aggregation. Future positions are excluded. | Exact-row reachability exists in the source-free substrate; the intervention-qualified geometric neighborhood and deployed trace/census are `NOT_YET_IMPLEMENTED`. |
| `P_mem^ℓ` | **Definition** | Deterministic, tokenizer-CID-bound projection from an identity-scoped memory span into ordered prime-route and spin/torsion state. | G0/G1 key/value adapter is historical; the factorable memory projection and product integration are `NOT_YET_IMPLEMENTED` under #958/#953. |
| `Δ_geo-null` | **Empirical Criterion** | Difference on the predeclared held-out metric between real geometry/memory and its equal-budget disabled or permuted control. A nonzero support trace alone is reachability, not advantage. | Bounded one-layer report under #951. |
| `S` | **Definition** | Semantic state space; every observation projects into a state `s ∈ S`. Regions are subsets `R_i ⊆ S`; beliefs are predicates over `S`; goals are desired subsets of `S`. | *Compiler/reference-only abstraction* (issue #124). The deployed runtime carries a fixed-capacity approximation: the runtime state (frontier, rolling context code, token shortlist) in `crates/uor-r4-core`. |
| `G = (V, E)` | **Definition** | Compiled semantic graph: `V` packed semantic states, `E` typed transitions. | `crates/uor-r4-graph-format` `GraphView` over the R4G1 NODE/EDGE sections (`docs/transformerless/R4G1.md`). |
| `H(x)` | **Definition** | Holographic encoding of observation `x`: a family of overlapping projections `{h_0, …, h_k}` with partial recoverability, distributed evidence, progressive fidelity. The deployed path today is still the single compiled Boolean semantic code; the certifier additionally defines and measures overlapping projection families for issue #126. | Runtime binding: sign-bit signature path in `crates/uor-r4-core` (see "Semantic code H(x)" in the glossary). Measurement contract binding: `crates/uor-r4-graph-certify/src/holographic_encoding.rs` and its deterministic fixture tests. |
| `T : S × A → S` | **Definition** | Typed graph dynamics: transition function over states and actions/semantic operators `A`. | *Compiler/reference-only* (issue #124). Deployed precursor: forward transition edges `E_f` / R4G1 ROUT section. |
| `R` | **Definition** | Reconstruction / behavioral-recovery operator; `R(H(x)) ≈ x` read behaviorally as the divergence condition below. | *Compiler/certifier-only*; exercised through the fidelity-certification harness (`score.rs`, Gate C). |
| `C : Θ → G` | **Definition** | The compiler as a map from teacher parameter space `Θ` to the space of compiled artifacts. Compilation is lossy semantic compression: parameters → behavioral probing → latent graph induction → Boolean synthesis → packed immutable artifact. | `crates/uor-r4-core` compiler pipeline; graph generalization in `crates/uor-r4-graph-compiler`. |
| `P_θ(·‖c)` | **Definition** | Teacher distribution over next tokens for context `c`, pinned HF revision, deterministic mode. | `TeacherOracle` next-token surface (`crates/uor-r4-core`). |
| `P_G(·‖H(x))` | **Definition** | Runtime/graph distribution produced by the compiled artifact. | Graph scorer (`crates/uor-r4-core` `score.rs`; R4G1 adapter `src/r4g1.rs`). |
| `D(·, ·)`, `ε` | **Definition** | Declared divergence measure and empirical tolerance for the behavioral reconstruction condition (below). | `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`. |

Behavioral reconstruction condition (the testable form of `R(H(x)) ≈ x`):

> **Empirical Criterion.** `D(P_θ(· | x), P_G(· | H(x))) ≤ ε` for a declared
> divergence `D` and tolerance `ε`, measured on a pinned held-out distribution with
> confidence intervals. Status: **Empirical**; never a structural claim.

## 4. Objectives versus runtime invariants

These are **Objectives** — quantities the offline compiler optimizes. They must
never be stated as runtime properties:

| Quantity | Role | Binding |
|---|---|---|
| `J = L_teacher + λ·C_runtime + μ·C_artifact` | **Objective** | Compiler cost model: teacher behavioral loss, inference cost, artifact size/complexity (issue #129). |
| `min_Z I(Z;X) − β·I(Z;Y_future)` | **Objective** | Information-bottleneck compression target: discard surface detail, keep future-relevant information (issue #127). |
| `H(A \| R)`, `H(S_future \| R)` | **Objective** | Predictive-entropy criteria for splitting, merging, or removing regions (issue #127). |
| `π* = arg max_π [ V(G \| T(B,π)) − P(C,π) − R(U,π) ]` | **Objective** | Plan-ranking target for bounded future-state optimization (issue #131). Not a theorem. |

These are **Guarantees** — structural properties of artifact and runtime, each with
a proof-model entry. Their current statuses are the proof-matrix records; a document
citing one MUST cite the same status:

| Guarantee | Status | Evidence |
|---|---|---|
| Allocation freedom on the prediction hot path | **Structural** | `allocation_proof` counting-allocator harness; `allocation_census.rs` (proof matrix: PDF §16) |
| Bounded packed ranges | **Structural** | `range_bounds_proof` (Theorem 8) |
| Deterministic top-K (canonical tie-break) | **Structural** | `deterministic_topk_proof` (PDF §23) |
| Forward/reverse index consistency | **Structural** | `theorem7_proof` (Theorem 7) |
| Score arithmetic safety (no overflow/panic) | **Structural** | Kani-1 harness (`kani_proofs.rs`) |
| Fixed-capacity container invariants | **Structural** | Kani-2 harness (`kani_proofs.rs`) |
| Inference operation-set conformance | **Witnessed** (Structural after machine-code audit) | `INFERENCE_OPERATION_CONTRACT.md` + P-4 source scans (`transformerless/mod.rs`) |
| Termination, bounded frontier width, valid references, canonical serialization, provenance completeness | per proof matrix | R4G1 two-stage validation + proof-model entries; anything lacking an executed evidence artifact is **Unproven** |

## 5. Term discipline (overloaded words)

| Avoid (unqualified) | Use instead | Rule |
|---|---|---|
| "intent" | **Future-state optimization**: belief = estimated current state, goal = desired future-state subset `G ⊆ S`, constraint = forbidden subset `F ⊆ S`, action = transition operator, plan = bounded trajectory `π = (a_0, …, a_n)` with `T^π(s_0) ∈ G` and `T^π_i(s_0) ∉ F` for all intermediate `i` (**Definition**, PDF §12). | Unqualified "intent" is informal prose; it must not appear in a labeled statement. |
| "semantic atom" | **Semantic region** — multiresolution, overlapping, proof-addressable; explicitly not an atom (glossary). | "Semantic atom" is prohibited in normative text. |
| "equivalence" | Qualified forms only: **byte reproducibility** (identical pinned inputs ⇒ identical artifact bytes) or **behavioral equivalence** (an **Empirical Criterion**, valid only on the declared distribution, per the equivalence protocol). | Unqualified "equivalence" is informal; "exact equivalence" hits the §2.1 wording rule. |
| "reasoning" | Precise mechanisms: **typed state transitions**, **graph navigation**, **bounded planning** (trajectory evaluation over `T`). | Bare "reasoning" is informal; plausible language output is never evidence of it (§2.1). |
| "transformerless" | **Zero source-attention calls and no dense full-prefix Q·K matrix/softmax kernel** on the exact promoted decoder path; the replacement uses bounded geometric support shown load-bearing by disabled/permuted interventions. | Does not imply multiplication-free, integer-only, allocation-free, coherent, or production-ready. Name retained source components and the checkpoint/scope. |
| "geometric intelligence" | Name the measured mechanism: **geometric support change**, **memory-causal logit effect**, **student-prefix rollout**, or another declared empirical result. | Geometry being present or visually structured is not evidence of intelligence. |

Existing documents predate this convention. Their already-qualified uses
("behavioral equivalence", "not semantic atoms", "equivalence-tested") conform;
unqualified uses are hereby marked informal and are migrated opportunistically, not
by wholesale rewrite.

## 6. Enforcement

- `scripts/check_claim_wording.py` scans `docs/**/*.md` and crate `README.md` files
  and fails on §2.1 violations. Run locally: `python3 scripts/check_claim_wording.py`.
- Automatic CI is dormant. A named product/release decision may activate the
  same script locally or through the manual workflow in `.github/workflows/ci.yml`.
- The proof-status matrix (`proof_matrix.rs`) is the machine-readable registry for
  §2 statuses; `verify_all` fails on any `Unverified` entry.

## Changelog

- **0.1.15** (2026-08-26) — Defined the geometric-intelligence route hierarchy
  and its claim boundaries: typed zero/identity and trigonometric chart bridges,
  lexical/prime/semiprime/n-let identities, zeta/Hopf/torsion/golden state,
  conceptual `E8 = H4 × H4` realized by the witnessed icosian
  `H4 ⊕ φH4` construction, hierarchical kappa and trajectory summaries,
  attention versus recall, inference, correctness, reasoning, and provider-free
  serving.
- **0.1.14** (2026-08-20) — Added the issue-#830 register execution-scope, serving-reachability, and empirical-verdict vocabulary: a per-row execution `scope` (`reference-only` / `offline-compiler` / `certifier-instrument` / `dormant-portable-runtime` / `normative-runtime` / `deployed-production`) and a serving `reachability` (`deployed-serving` / `off-serving-path` / `dormant-gated`), plus a three-value empirical status (`PASS` / `FAIL` / `UNAVAILABLE`) kept as a separate axis from the harness-built `build` level — an absent fixture is `UNAVAILABLE`, never `PASS` (`crates/repo-model/src/registry.rs`, `crates/repo-model/src/empirical.rs`; rendered into `CONFORMANCE.md`).
- **0.1.13** (2026-07-25) — Added issue-#175 compiler parallelism benchmarks and scaling certificate definitions (`Compiler Parallelism Scaling Report`, `Multicore Thread Sweep Matrix`, `Stage Scaling Classification Taxonomy`, `Byte-Equality Scaling Premise` in `docs/compiler_scaling_certificate.md` and `uor-r4-graph-certify::compiler_scaling`).
- **0.1.12** (2026-07-25) — Added issue-#174 CPU-only compiler dependency and feature audit definitions (`Compiler Dependency Denylist Gate`, `Default Feature Unification Rule`, `Teacher-Backend Isolation Invariant`, `CPU-Only Runner Compliance` in `docs/compiler_dependency_audit.md` and `uor-r4-graph-compiler::dependency_audit`).
- **0.1.11** (2026-07-24) — Added issue-#170 parallel observation, trace, and evaluation processing definitions (`Content-Addressed Shard Partitioning`, `Ordered Deterministic Reduction`, `Shard ID Determinism`, `Teacher-Backend Boundary` in `docs/parallel_observation_shards.md` and `uor-r4-graph-compiler::observation_shards`).
- **0.1.10** (2026-07-24) — Added issue-#169 compiler memory-budget and backpressure model definitions (`Concurrency-Aware Memory Formula`, `Per-Stage Memory Estimate`, `In-Flight Backpressure Limiter`, `Constrained-Memory Determinism` in `docs/compiler_memory_budget.md` and `uor-r4-graph-compiler::memory_budget`).
- **0.1.9** (2026-07-24) — Added issue-#168 compiler thread-pool and jobs configuration definitions (`Compiler Concurrency Control`, `Jobs Precedence Resolution`, `Dedicated Thread-Pool Ownership`, `Oversubscription Policy` in `docs/compiler_concurrency_config.md` and `uor-r4-graph-compiler::jobs_config`).
- **0.1.8** (2026-07-24) — Added issue-#167 normative reproducibility & canonical byte-equality definitions (`Normative Reproducibility Invariant`, `Parallel Reproducibility Harness`, `Thread-Count Invariance`, `Deterministic Reduction Policy` in `docs/reproducibility.md` and `uor-r4-graph-compiler::reproducibility`).
- **0.1.7** (2026-07-24) — Added issue-#166 compiler stage ownership and parallelization DAG definitions (`Compiler Stage DAG`, `Parallel-Safe Stage`, `Deterministic Merge Stage`, `Bounded Parallel Stage`, `Sequential Canonical Finalization Spine` in `docs/compiler_stage_dag.md` and `uor-r4-graph-compiler::stage_dag`).
- **0.1.6** (2026-07-24) — Added issue-#165 deterministic compiler executor definitions (`Compiler Executor Abstraction`, `Sequential Reference Executor`, `Rayon Parallel Executor` in `uor-r4-graph-compiler::executor`).
- **0.1.5** (2026-07-24) — Added issue-#161 runtime operation, allocation, and CPU portability certificate definitions (`Runtime Performance Certificate`, `Evidentiary Class Schema`, `Declared-Zero Evidence Link`, `CPU Portability Record` in `uor-r4-graph-certify::performance_certificate`).
- **0.1.4** (2026-07-24) — Added issue-#160 machine-code, allocator, and dependency CI audit definitions (`Machine-Code Disassembly Audit`, `Counting Allocator Witness`, `Dependency Denylist Gate` in `uor-r4-proof-model::inference_audit`).
- **0.1.3** (2026-07-24) — Added the issue-#158 normative scoring semantics definitions (`Fixed-Point Scoring Semantics`, `Residual Taxonomy`, `Overlap Residualization`, `Deterministic Tie-Breaking` in `docs/scoring_semantics.md` and `uor-r4-graph-format::scoring_semantics`).
- **0.1.2** (2026-07-24) — Added the issue-#157 normative inference contract definitions (`Normative Inference Contract`, `Permitted Operation Class`, `Zero-Allocation Steady State`, `CPU-Only Target Contract` in `docs/inference_contract.md` and `uor-r4-graph-format::inference_contract`).
- **0.1.1** (2026-07-24) — Added the issue-#126 measurement-contract binding for
  `H(x)` (projection family schema, ablation semantics, and deterministic partial
  recovery/progressive-fidelity fixture in `uor-r4-graph-certify`).
- **0.1.0** (2026-07-24) — Initial version. Claim classes, claim statuses, core
  notation (`S`, `G=(V,E)`, `H(x)`, `T`, `R`, `C`, `P_θ`, `P_G`), objectives vs.
  runtime invariants, term discipline, and the CI wording rule. (Issue #123.)
