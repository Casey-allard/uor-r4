# Formal Vocabulary, Notation, and Claim Classes

- **Version:** 0.1.0
- **Status:** Normative for all new specification, plan, proof-model, and certificate text.
- **Source:** `docs/hologram_formal_analysis_direction.pdf` §§1, 7, 13; tracker
  [#122](https://github.com/UOR-Foundation/uor-r4/issues/122); issue
  [#123](https://github.com/UOR-Foundation/uor-r4/issues/123).
- **Related:** terminology for existing graph-compiler concepts lives in
  `docs/transformerless/GLOSSARY.md`; this document governs *claim language* and
  mathematical notation. Where the two overlap, this document wins for claim
  classification and the glossary wins for structural terms.

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
| **Structural** | Established by construction, a machine-checked test, a fuzz target, or a formal proof artifact that runs in CI. | `Verified` |
| **Witnessed** | Established per execution by a bounded, replayable witness an independent verifier replays without the teacher. | `Verified` (witness path) |
| **Empirical** | Measured on a pinned corpus with protocol, confidence intervals, and provenance identifiers, per `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`. | `DifferentialPass` |
| **Assumed** | Required by a proof or certificate but not established by the implementation. | recorded as an assumption entry |
| **Unproven** | Asserted as a goal but currently without evidence. | `Unverified` (blocks `verify_all`) |

`ExecutableSpec` in the proof matrix means the obligation has a runnable
specification but no CI-enforced check yet; documents must label such claims
**Unproven** (or **Assumed** where applicable), never **Structural**.

### 2.1 Wording rule (CI-enforced)

The prohibited phrases are **"machine-verified"**, **"machine verified"**, **"exact equivalence"**, **"exact teacher equivalence"**, and **"provably equivalent"**. They are prohibited in
normative documents (`docs/**`, crate READMEs) unless the same line links a proof
artifact or certificate, or the phrase is explicitly disavowed on that line.
`scripts/check_claim_wording.py` enforces this in CI (see §6). The project does not
claim exact teacher equivalence (disallowed per this rule), does not claim
human-level reasoning, and does not treat plausible language output as evidence of
coherent internal state transitions.

## 3. Core notation

Symbols are listed with their claim role and their concrete Rust binding. Entries
marked *compiler/reference-only* have no deployed-runtime representation yet; the
runtime path for them lands in the issues noted.

| Symbol | Role | Meaning | Rust / artifact binding |
|---|---|---|---|
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
| Termination, bounded frontier width, valid references, canonical serialization, provenance completeness | per proof matrix | R4G1 two-stage validation + proof-model entries; anything lacking a CI artifact is **Unproven** |

## 5. Term discipline (overloaded words)

| Avoid (unqualified) | Use instead | Rule |
|---|---|---|
| "intent" | **Future-state optimization**: belief = estimated current state, goal = desired future-state subset `G ⊆ S`, constraint = forbidden subset `F ⊆ S`, action = transition operator, plan = bounded trajectory `π = (a_0, …, a_n)` with `T^π(s_0) ∈ G` and `T^π_i(s_0) ∉ F` for all intermediate `i` (**Definition**, PDF §12). | Unqualified "intent" is informal prose; it must not appear in a labeled statement. |
| "semantic atom" | **Semantic region** — multiresolution, overlapping, proof-addressable; explicitly not an atom (glossary). | "Semantic atom" is prohibited in normative text. |
| "equivalence" | Qualified forms only: **byte reproducibility** (identical pinned inputs ⇒ identical artifact bytes) or **behavioral equivalence** (an **Empirical Criterion**, valid only on the declared distribution, per the equivalence protocol). | Unqualified "equivalence" is informal; "exact equivalence" hits the §2.1 wording rule. |
| "reasoning" | Precise mechanisms: **typed state transitions**, **graph navigation**, **bounded planning** (trajectory evaluation over `T`). | Bare "reasoning" is informal; plausible language output is never evidence of it (§2.1). |

Existing documents predate this convention. Their already-qualified uses
("behavioral equivalence", "not semantic atoms", "equivalence-tested") conform;
unqualified uses are hereby marked informal and are migrated opportunistically, not
by wholesale rewrite.

## 6. Enforcement

- `scripts/check_claim_wording.py` scans `docs/**/*.md` and crate `README.md` files
  and fails on §2.1 violations. Run locally: `python3 scripts/check_claim_wording.py`.
- CI runs the same script as a step of the `gates` job (`.github/workflows/ci.yml`).
- The proof-status matrix (`proof_matrix.rs`) is the machine-readable registry for
  §2 statuses; `verify_all` fails on any `Unverified` entry.

## Changelog

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
