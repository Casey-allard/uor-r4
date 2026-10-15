# #137 — Maintain a PDF-to-implementation traceability matrix for the formal direction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:29:31Z
- **Updated:** 2026-07-24T20:12:13Z
- **Closed:** 2026-07-24T20:12:13Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/137
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: `docs/hologram_formal_analysis_direction.pdf` §§1-17.

Create a living traceability matrix that prevents sections of the formal direction from disappearing into broad tracker issues. The matrix should map each PDF recommendation to a GitHub issue, code/document location, evidence artifact, and current status.

## Required coverage

- Definitions/objectives/guarantees.
- Semantic state manifold.
- Typed graph dynamics.
- Multiple edge algebras over one graph.
- Holographic partial reconstruction.
- Predictive entropy.
- Semantic compression.
- Information bottleneck.
- Explicit graph invariants.
- Thinking/state transitions versus language emission.
- Compiler research pipeline and interventions.
- Future-state optimization/planning.
- Structural proofs.
- Overall project direction and description.
- Monograph structure.
- Immediate research sequence.

## Acceptance criteria

- The matrix is stored under `docs/` and links to all active and completed issues, including existing implementation issues #11-#34 and formal-direction issues #122 onward.
- Each row names the definition/objective/guarantee/empirical status and its owner.
- Rows cannot be marked complete without a code, document, test, certificate, or proof artifact link.
- The matrix identifies overlaps and dependencies rather than creating duplicate implementations.
- CI or a review checklist detects unmapped new sections and stale issue references.

## Comments (3)

### comment 5073103594 — 2026-07-24T18:21:20Z — auser

Note for the matrix (and its in-flight PR #156 revision): the multiplication-free inference chain #157–#161 needs rows once the matrix lands.

Suggested mapping:

- **#157** (normative multiplication-free inference contract) → PDF §1 guarantee inventory ("absence of heap allocation on the hot path" extended to the full operation set); claim class **Definition** + **Guarantee**; status **Unproven** until #160 lands machine-code evidence.
- **#158** (fixed-point scoring semantics) → PDF §§7, 12; **Guarantee** (arithmetic) / **Empirical Criterion** (fidelity).
- **#159** (allocation-free packed kernels) → PDF §§1, 9; **Guarantee**, Gate B.
- **#160** (disassembly audit) → PDF §13; **Guarantee**, status **Witnessed**; completes plan §7 T2.
- **#161** (operation/memory-cost certificates) → PDF §13; **Witnessed** + **Empirical**.

Dependency chain: #157 → #158 → #159 → #160 → #161. None of these rows can be marked complete at creation — evidence artifacts do not exist yet, per the matrix rule that rows require a code/test/certificate/proof link.


### comment 5073197598 — 2026-07-24T18:31:07Z — auser

Update to the earlier #157–#161 mapping note (2026-07-24): the five issues were expanded in place — titles and scopes changed, row mapping should follow the current titles:

- **#157** "Define the normative CPU-only, multiplication-free, zero-allocation inference contract" → PDF §1 guarantee inventory, §9 invariants; claim class **Definition** + **Guarantee**; status **Unproven** until #160 evidence lands.
- **#158** "Specify fixed-point scoring semantics: residual taxonomy, accumulation order, saturation, and tie-breaking" → PDF §§7, 12; **Guarantee** (arithmetic) / **Empirical Criterion** (quantization loss, fidelity).
- **#159** "Implement packed no-alloc CPU inference kernels over immutable graph arrays" → PDF §§1, 9; **Guarantee**, Gate B.
- **#160** "Machine-code, allocator, and dependency CI enforcement for the inference contract" → PDF §13; **Guarantee**, status **Witnessed**; completes plan §7 T2 and adds the no-`alloc` build gate and GPU/dependency audit.
- **#161** "Runtime operation, allocation, and CPU portability certificates" → PDF §13; **Witnessed** + **Empirical**.

The matrix's per-row "new requirements" column should record: CPU-only normative target (previously untracked anywhere in the repo), no-`alloc` build configuration, allocator-failure testing across all ResolutionStatus paths, and the legal address-generation exception. Dependency chain unchanged: #157 → #158 → #159 → #160 → #161. No row may be marked complete at creation — evidence artifacts do not exist yet.


### comment 5073320217 — 2026-07-24T18:45:01Z — auser

Third chain for the matrix: deterministic compiler parallelism #165–#175 (parent #122).

Suggested mapping:

- **#165** (executor abstraction) → PDF §§7, 11 (compiler pipeline); **Definition** + **Guarantee** (executor equivalence, Structural).
- **#166** (stage DAG) → PDF §§7, 11; **Definition**; finalization-spine guarantee **Unproven** until #167 lands.
- **#167** (reproducibility under parallelism) → R4G1.md §7 / D2, Gate E, plan §4.1; **Guarantee**, status **Structural** (extends #21's sequential rebuild test).
- **#168** (jobs/oversubscription), **#169** (memory budget) → plan §4.1; **Definition** + budget-safety **Guarantee** (#169); RSS numbers **Empirical**.
- **#170–#173** (parallel stages) → PDF §§7, 11; thread-count-invariant output **Guarantee** (Structural, harness-backed); throughput **Empirical**.
- **#174** (CPU-only compiler audit) → PDF §14 direction; **Guarantee**, status **Witnessed** (CI gate).
- **#175** (scaling benchmarks) → plan §8/§9.17; **Empirical Criterion** only.

All rows start without evidence artifacts — none may be marked complete at creation, per the matrix rule.



---
_Comment/review ids in this file: 5073103594, 5073197598, 5073320217_

