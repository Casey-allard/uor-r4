# #859 — formal/#820-F0: specify and prove the stable R⁴ prime-router kernel in Lean 4

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T22:01:18Z
- **Updated:** 2026-08-25T23:32:21Z
- **Closed:** 2026-08-25T23:32:21Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/859
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

Cross-cutting foundation F0. This work begins during S0 but is not an S0 → S1/S2 promotion gate.

## Context

“R4 Prime Router” currently names an exploratory mathematical and software stack spread across `uor-r4-core` and `uor-r4-router`: canonical identity handling, digest-to-prime bucketing, vocabulary-relative prime assignment, pinned zeta-ordinate vectors, normalized session state, signed 512 → 4 projection, S³/C² Hopf coordinates, phase transport, sector assignment, spectral/VSA routing, normalized state evolution, and sentence prime products.

No existing proof asset establishes this router:

- `research/riemann-lean/formal/lean` is a separate conditional Prime/Riemann bridge. It is pinned to Lean 4.28, contains supplied theorem/provider boundaries, has stale post-move verification paths, is absent from root CI, and is not a router specification or implementation refinement.
- `proofs/wasm-gemm-gnaf` is pinned to Lean 4.30 and proves a separate reference WASM GEMM/GNAF problem. Its proof-process disciplines are reusable; its theorems do not transfer to R4 serving.
- `uor-r4-proof-model` contains Rust executable obligations and a small Kani surface, not a Lean formalization of the prime router.
- The Hologram/R4 monograph is a normative specification and evidence map, not a Lean theorem library.
- The dashboard's 96 rendered vertices are not currently a normative construction of the 40-point/40-line symplectic generalized quadrangle W(3,3), and W(3,3) is not an input or output of the Rust router.
- The deployed graph path is separate from the allocating f64 router, so a reference theorem must not be described as deployed-serving evidence.

## Problem

Without one machine-checked reference specification, “proofing R4” can accidentally mix:

- Euclidean S³ normalization with the separately defined Minkowski quadratic form;
- mathematical real-number functions with Rust `f64` approximations;
- a 500-bucket prime label with collision-free or injective identity addressing;
- arrival-ordered word primes with portable vocabulary coordinates;
- pinned numerical zeta data with a proof of those zeros or the Riemann hypothesis;
- a normalized chord/EMA update with a proved geodesic;
- an implemented phase formula with a derived Levi-Civita connection;
- algebraic/topological guarantees with empirical semantic usefulness;
- dashboard W(3,3) imagery with an actual finite incidence geometry;
- reference mathematics with correctness of the optimized serving implementation.

Deferring all formalization until S7 would lose its value as an early design and assumption audit. Making every intelligence stage wait for it would overstate the current router's production relevance.

## Goal

Create a pinned, importable, publication-ready Lean 4 library and proof packet for the stable pure mathematical kernel of the R⁴ prime router.

The initial top-level theorem target is:

> For every valid declared configuration and input, the pure reference kernel is total and deterministic; every selected prime bucket, window, bin, and sector lies within its declared bounds; every nonzero normalized state lies on its declared sphere; and a declared witness replays to the same reference routing decision.

Every precondition and external authority remains explicit. This theorem does **not** assert semantic quality, intelligence, collision freedom, Riemannian-geodesic status, Rust/`f64` equivalence, production reachability, or the Riemann hypothesis.

## Proof dependency shape

```text
Pinned configuration / finite dimensions / exact table data
├── Prime enumeration
│   ├── identity-prime bucket
│   └── stateful word vocabulary
├── Finite vector algebra
│   ├── zeta-word embedding
│   ├── content/session state
│   └── signed 512 → 4 projection
├── Sentence prime product
├── Sphere and cosine
│   ├── canonical Hopf map
│   ├── phase-transport algorithm
│   ├── sector encoding
│   └── normalized state evolution
└── Exact window reference
    ├── window bounds
    ├── exact QR preconditions
    └── deterministic argmax
             ↓
     RouterCoreWellFormed
             ↓
     Witness/replay model
             ↓
Rust/f64 or packed-runtime refinement — separate later proof layer

W(3,3) finite geometry — independent, conditional module after an exact bridge exists
Prime/RH research — independent
GNAF — proof-process methodology only
```

## Scope

### A. Freeze the proof boundary

- [ ] Inventory every equation and algorithm claimed as part of the prime-router kernel, with provenance to current Rust, prior lineage, and repository documentation.
- [ ] Resolve overloaded terms including R4/R⁴, QIMC, norm, geodesic, phase transport, W(3,3), prime coordinate, witness, and “verified.”
- [ ] Separate stable pure mathematics from hash authority, floating-point approximation, empirical mechanisms, visualization, and deployed graph semantics.
- [ ] Distinguish the Euclidean R⁴/S³ structure used by the Hopf path from `R4Vector::minkowski_norm`; prohibit theorem statements that silently interchange them.
- [ ] Classify W(3,3) separately. Do not formalize the current 96-vertex canvas as W(3,3) without an authoritative finite-geometry definition and an exact implementation mapping.
- [ ] Publish versioned definitions, theorem IDs, assumptions, open obligations, counterexamples, and excluded claims before claiming proof completion.
- [ ] Correct stale Riemann-project dependency statements and broken verification/audit paths while preserving it as separate conditional research.

### B. Establish an importable Lean package

- [ ] Create a first-party package under `proofs/r4-prime-router/` with a pinned Lean toolchain, pinned mathlib/dependencies, deterministic manifest, license, README, and reproduction commands.
- [ ] Use `set_option autoImplicit false` and a stable namespace such as `Uor.R4.PrimeRouter`.
- [ ] Expose a small public root module and stable theorem API that another clean Lean project can import.
- [ ] Add a downstream smoke fixture that imports the package and reuses at least one public theorem.
- [ ] Use no repository-local absolute paths, floating dependency branches, or ambient unpinned dependencies.
- [ ] Record a deliberate compatibility policy for the repository's currently fragmented Lean 4.28/4.30/legacy toolchains; do not imply cross-version source compatibility without testing it.

### C. Mechanize the stable exact kernel

- [ ] Formalize identity canonicalization and digest-bucket selection with the digest treated as an explicit input/authority boundary.
- [ ] Formalize the selected finite prime table congruent to 1 modulo 6 and prove count, ordering/distinctness, primality, congruence, index range, and deterministic selection.
- [ ] Prove the identity mapping's declared bucket bounds and pigeonhole non-injectivity; do not call it globally unique or collision-free.
- [ ] Formalize arrival-ordered word-to-prime assignment separately and state the vocabulary-order compatibility invariant.
- [ ] Define the zeta vector over a pinned gamma table treated as data; prove length and conditional normalization/cosine laws without asserting RH, certified zero enclosures, or semantic meaning.
- [ ] Formalize the signed 512 → 4 projection, zero fallback, Euclidean normalization, and nonzero unit-sphere result.
- [ ] Formalize normalized content/session-state updates and their zero/nonzero domain conditions. Prove only the stated span/sphere results; do not call the update a geodesic without a declared metric and derivation.
- [ ] Define the canonical Hopf map algebraically on S³/C², then connect angular chart formulas only under explicit chart preconditions.
- [ ] Prove the appropriate Hopf-coordinate identities, common-phase invariance, phase periodicity/wrapping range, bin-budget bounds, deterministic ties, and `sector_id < k` for valid `k ≥ 1`.
- [ ] Name the current phase expression `phaseTransportAlgorithm` unless a metric, connection form, and horizontal-lift derivation establish a stronger geometric name.
- [ ] Formalize sentence prime-product behavior and prove multiset recoverability only for a fixed injective vocabulary with no filtering and no saturation.
- [ ] Prove or retain counterexamples for loss of word order, stopword removal, repeated factors, unknown words, vocabulary-order drift, and saturating overflow.
- [ ] Treat exact-real zeta-window/QR well-formedness as an optional module with explicit matrix-rank and numerical-authority assumptions, not part of the mandatory core theorem.
- [ ] Make generation determinism conditional on temperature zero or an explicit entropy tape.

### D. Audit assumptions and falsifiers

- [ ] Classify SHA-256 behavior/collision resistance, pinned gamma values, transcendental evaluation, `f64` rounding, tokenizer behavior, and semantic relevance as authorities, assumptions, refinement obligations, or empirical questions.
- [ ] Run declaration-root and `#print axioms` audits; reject `sorry`, `admit`, hidden placeholder providers, project-local axioms, unreachable proof roots, and undeclared dependencies. Permit only an explicitly reviewed standard axiom whitelist.
- [ ] Run `lean4checker --fresh` or the pinned equivalent over publication roots.
- [ ] Add planted failures for wrong prime congruence, off-by-one bucket/sector indices, omitted phase wrapping, zero-vector normalization, invalid capacity, false injectivity, and false product-recovery assumptions.
- [ ] Preserve counterexamples and refuted theorem proposals as first-class results.

### E. Define the Rust/refinement boundary

- [ ] Map every Lean definition/theorem to the corresponding Rust function, or mark it reference-only/unmapped.
- [ ] Differentially test exact finite domains and golden vectors only where Lean and Rust semantics are genuinely comparable.
- [ ] For hash, trigonometric, square-root, QR, random, and `f64` paths, provide a proved error/refinement contract or mark implementation equivalence open.
- [ ] Do not report golden-vector agreement, bounded testing, a source scan, or Kani model checking as a proof of the complete Rust implementation.
- [ ] Leave packed/deployed scorer, parser, witness, patch, composition, and production-call-graph refinement to #856 after those semantics stabilize.

### F. Publication, CI, and conformance

- [ ] Publish a human-readable proof packet containing definitions, theorem statements, dependency graph, Lean declaration links, assumptions, counterexamples, bibliography/provenance, toolchain identities, source hashes, and exact reproduction commands.
- [ ] Generate a deterministic machine-readable theorem/proof-status manifest suitable for #855/#856.
- [ ] Add a path-scoped Lean verification gate consistent with the repository's CI policy: fast PR-side validation/stub and binding build/audits in the merge queue under the same required check name.
- [ ] Add an external/downstream import test to substantiate interoperability at the Lean package/API level.
- [ ] Allocate a new RF capability only for the built, importable Lean reference package; classify its execution scope as formal reference mathematics, not deployed serving.
- [ ] Add any built capability through `model/ids.toml` → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- [ ] Register retained unrefined or disputed mechanisms in `model/ledger.toml` with explicit closure/activation conditions.
- [ ] Reconcile README, ROADMAP, RESEARCH, glossary, formal monograph, proof docs, and current explainers when a proof or counterexample revises a live claim.

## Non-goals

- Prove the Riemann hypothesis, the correctness of the pinned zeta table, prime-gap manuscripts, or unrelated Prime/Riemann bridge results.
- Prove semantic relevance, retrieval quality, prompt conditioning, instruction following, reasoning, intelligence, or geometric advantage.
- Prove SHA-256 collision resistance or unique identity routing.
- Claim the existing f64 Rust implementation is formally refined without a separately checked refinement/error result.
- Formalize thermodynamic, Ricci, quantum, cosmological, or information-theoretic language without precise definitions and actual code ownership.
- Treat the dashboard's 96 rendered vertices as a proof object for W(3,3).
- Promote `uor-r4-router` into the graph-runtime path.
- Duplicate the GNAF GEMM proof or #856's stable production-refinement obligations.

## Dependencies and entry gate

No blocker to begin.

Coordinate with:

- #829 for the canonical completion-plan and documentation hierarchy;
- #830 for execution-scope and formal-evidence representation;
- #845 for any later geometry candidate that consumes these definitions;
- #855/#856 for the final TCB map and production refinement.

This issue blocks #845 and #856. It does **not** block #821, S1, S2, or S3 empirical work.

## Acceptance criteria

- [ ] A clean clone builds the pinned Lean package and downstream import fixture.
- [ ] All mandatory declarations compile without `sorry`, `admit`, hidden placeholder providers, unexpected axioms, or missing proof roots.
- [ ] Every theorem has a stable ID, exact statement, assumptions, provenance, Lean declaration, and implementation/refinement status.
- [ ] The top-level totality/determinism/bounds/normalization/replay theorem is kernel-checked for the declared pure reference kernel.
- [ ] Prime-bucket non-injectivity, vocabulary-order dependence, sentence-product information loss, and sphere/chart edge conditions are proved properties or checked counterexamples.
- [ ] Exact finite differential fixtures agree with Rust, and planted mutations prove the relevant gates can fail.
- [ ] Transcendental, hash, numerical-data, randomness, and `f64` boundaries cannot be mistaken for proved implementation equivalence.
- [ ] W(3,3) is backed by an authoritative separate specification and exact bridge or remains explicitly excluded/open.
- [ ] The proof packet and theorem manifest reproduce from pinned inputs.
- [ ] A clean external Lean project imports the public root and reuses a named theorem.
- [ ] Public documentation uses “machine-checked reference theorem” only with a direct proof-artifact link and never says “R4 is proven” without its exact boundary.

## Verification

- [ ] Run the pinned `lake build` and proof-package verification command from a clean environment.
- [ ] Run sorry/admit, axiom/root, dependency-firewall, required-declaration, manifest, deterministic-generation, and `lean4checker` audits.
- [ ] Run the downstream import/reuse smoke project.
- [ ] Run all mutation/planted-counterexample checks.
- [ ] Run Lean/reference/Rust golden-vector and finite-domain differential suites.
- [ ] Run `cargo test --workspace --no-fail-fast --offline`, clippy, fmt, and the required WASM check for any touched Rust core/router code.
- [ ] Run claim-wording, model/conformance, deferral, and generated-file checks.

## Compatibility and migration

The Lean package is additive and absent from deployed runtime dependencies. Definitions and theorem statements receive a versioned proof era; incompatible changes preserve historical proof artifacts and explicitly invalidate affected refinement links.

Downstream Lean compatibility means an importable, pinned theorem API. It does not imply compatibility with every Lean toolchain or automatic proof transfer to unrelated projects.

## Repository conformance

- **Execution scope:** Exact/reference Lean mathematics plus explicitly named finite differential bridges. No deployed-serving, semantic-quality, or target-binary claim.
- Lean establishes only the encoded statements under their listed assumptions.
- Empirical questions remain empirical and use CID-bound protocols if later evaluated.
- Proof artifacts and public claims distinguish formal proof, executable check, model check, differential evidence, source check, target-binary evidence, empirical evidence, and assumption.
- Generated `CONFORMANCE.md` is never hand-edited.

## Claim status and next action

Current status: the R⁴ prime-router mathematical kernel is **not** formalized in Lean.

Successful completion establishes a machine-checked reference theorem library and explicit proof boundary. It does not establish that the router is intelligent, semantically effective, production-reachable, or correctly implemented in floating point.

## Comments (2)

### comment 5368815836 — 2026-08-21T10:42:29Z — Casey-allard

### Maintainer scoping (2026-08-21): Milestone-0 sanctioned — a statements-only scaffold, for review before any proof work

Per the maintainer's decision this morning, F0 begins with a bounded, review-gated first slice:

**M0 scope (no theorems asserted):**

- A Lean **4.30** project (matching the `proofs/wasm-gemm-gnaf` pin, whose proof-process disciplines this issue names reusable), self-contained under `proofs/` — no coupling to `research/riemann-lean` (4.28, the separate conditional Prime/Riemann bridge, which stays out of scope).
- The kernel **STATEMENT spec as Definitions only**: the ℝ-model semantics of the stable router-kernel surface (canonical identity handling, digest-to-prime bucketing as a bucketing — not injective-addressing — claim, vocabulary-relative prime assignment, the normalized state objects, the S³ Euclidean form named explicitly as such). Each definition carries its intended Rust binding as a comment so review = checking statements against intent, per `docs/formal_vocabulary.md` (everything labeled **Definition**; nothing labeled Guarantee until a proof milestone earns it).
- **Explicitly OUT of M0:** the f64 refinement layer, W(3,3), the Minkowski form's relationship to the S³ normalization (flagged as a modeling question for review, not silently chosen), pinned zeta data (numerical inputs, never proof subjects), and anything Riemann-related.
- CI: a non-required build-only job that elaborates the Lean project (reported in both contexts per the required-check naming rule if it is ever promoted to required).
- Deliverable: a PR marked **"References #859"** (M0 does not close this issue) for maintainer review of the statements BEFORE any proof milestone is scoped. Confusion hazards this issue's Problem section lists are treated as review checklist items on that PR.


### comment 5418399811 — 2026-08-25T23:32:21Z — Casey-allard

Closed as not planned under the current architecture reset #948 / #949. This issue would formalize the exploratory prime-router kernel before the learned causal geometry is known. Its specification remains useful design evidence, but proof work is deferred until #951/#952 identify the operator that actually carries language behavior. A narrow successor may be opened then; no theorem or implementation result is claimed here.


---
_Comment/review ids in this file: 5368815836, 5418399811_

