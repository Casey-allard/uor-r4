# Architecture

## Workspace Layout

The UOR Framework is a Rust workspace with seven member crates:

| Crate | Type | Role |
|-------|------|------|
| `uor-ontology` | Library | Ontology as typed Rust data + serializers (internal, not published) |
| `uor-codegen` | Library | Code generator: ontology → Rust traits (internal) |
| `uor-foundation` | Library | **Generated** Rust trait crate (published to crates.io) |
| `uor-conformance` | Library | Workspace-wide conformance validators |
| `uor-docs` | Library | Documentation generator |
| `uor-website` | Library | Static site generator |
| `uor-clients` | Binaries | CLI tools: build, conformance, docs, website, crate |

## Dependency Order

```
uor-ontology (spec/)
  ↓ Ontology::full()
  ├── uor-build      → public/uor.foundation.{json,ttl,nt}, uor.term.ebnf
  ├── uor-docs       → public/docs/                           (documentation export)
  ├── uor-website    → public/                                 (website export)
  ├── uor-crate      → foundation/src/                         (Rust export)
  └── uor-conformance → validates all of the above
```

## spec/ Library (uor-ontology)

`uor-ontology` encodes the complete ontology as typed Rust static data:

- **`model.rs`**: Core types — `Namespace`, `Class`, `Property`, `Individual`, `Ontology`
- **`namespaces/*.rs`**: {@count:namespaces} modules, one per namespace (all {@count:amendments} amendments applied)
- **`serializer/jsonld.rs`**: Serializes to JSON-LD 1.1
- **`serializer/turtle.rs`**: Serializes to Turtle 1.1
- **`serializer/ntriples.rs`**: Serializes to N-Triples

The entry point `Ontology::full()` uses `OnceLock` for thread-safe lazy initialization.

This crate is the single source of truth. It is internal (`publish = false`).

## codegen/ Library (uor-codegen)

`uor-codegen` generates the published Rust trait crate from the ontology:

- **`mapping.rs`**: Namespace → module, XSD → Rust type, class IRI → path tables
- **`traits.rs`**: Class → trait, property → method generation
- **`enums.rs`**: Enum detection (PrimitiveOp, MetricAxis, Space, SiteState, ExecutionPolicyKind, GeometricCharacter, VerificationDomain, ComplexityClass, RewriteRule, MeasurementUnit, TriadProjection, SessionBoundaryType, PhaseBoundaryType, GroundingPhase, AchievabilityStatus, ValidityScopeKind, ProofModality) and WittLevel newtype struct generation
- **`individuals.rs`**: Named individual → const module / PrimitiveOp impl generation
- **`emit.rs`**: Rust source code writer

## foundation/ Library (uor-foundation)

`uor-foundation` is the **generated** published crate. Every file in `foundation/src/`
is produced by `uor-crate` — never hand-edited.

- **{@count:traits} traits** (one per OWL class, generic over `Primitives`)
- **{@count:methods} methods** (one per property with a domain)
- **{@count:enums} enums** (Space, PrimitiveOp, MetricAxis, SiteState, ExecutionPolicyKind, GeometricCharacter, VerificationDomain, ComplexityClass, RewriteRule, MeasurementUnit, TriadProjection, SessionBoundaryType, PhaseBoundaryType, GroundingPhase, AchievabilityStatus, ValidityScopeKind, ProofModality) **+ 1 struct** (WittLevel)
- **{@count:constant_modules} constant modules** (for named individuals)
- **Zero mandatory dependencies** — pure traits

Module structure: `kernel/` ({@count:kernel_ns} namespaces), `bridge/` ({@count:bridge_ns} namespaces), `user/` ({@count:user_ns} namespaces).

## conformance/ Library

`uor-conformance` is the workspace quality gate. It validates:

1. **Rust source**: API documentation, style conventions
2. **Ontology artifacts**: JSON-LD 1.1, OWL 2 DL constraints, RDF 1.1, Turtle 1.1, inventory counts
3. **SHACL instances**: {@count:shacl_tests} test graphs validated against {@count:shapes} NodeShapes
4. **Generated crate**: Trait completeness, method completeness, individual completeness
5. **Documentation**: Diataxis structure, completeness, accuracy, links
6. **Website**: HTML5, WCAG 2.1 AA, CSS, coverage, links

The conformance suite is the **single gate** — all components must pass before a release.

## docs/ Library

`uor-docs` generates documentation with enforced accuracy:

- Namespace reference pages are **100% auto-generated** from `uor_ontology::Ontology::full()`
- Prose pages use `{@class}`, `{@prop}`, `{@ind}` DSL, validated at build time
- Every spec term must appear in at least one page (completeness check)

## website/ Library

`uor-website` generates the static site at `https://uor.foundation/`:

- Templates use the Tera template engine
- Namespace pages are auto-generated (no hand-written HTML for spec terms)
- Search index is generated from all {@count:classes} classes, {@count:properties} properties, {@count:individuals} individuals
- No external dependencies (no CDN, no tracking, no third-party scripts)

## Build Pipeline

```
cargo run --bin uor-build       → public/uor.foundation.{json,ttl,nt}
cargo run --bin uor-crate       → foundation/src/ (generated Rust traits)
cargo run --bin uor-docs        → public/docs/ + README.md
cargo run --bin uor-website     → public/ (HTML, CSS, JS, search-index.json)
cargo run --bin uor-conformance → validates all of the above
```

## Amendment History

The spec crate implements all {@count:amendments} amendments from the UOR Foundation completion plan:

| Amendment | Namespace | Key Additions |
|-----------|-----------|---------------|
| 1 | op/ | 10 named operation individuals |
| 2 | schema/ | Ring class, 6 properties, pi1/zero individuals |
| 3 | op/, proof/ | Identity class, criticalIdentity individual, provesIdentity property |
| 4 | op/ | Group/DihedralGroup classes, D2n individual |
| 5 | partition/ (NEW) | 6 classes, 9 properties |
| 6 | morphism/ (NEW) | 4 classes, 10 properties |
| 7 | state/ (NEW) | 4 classes, 16 properties |
| 8 | all | uor:space annotation property on all namespaces |
| 9 | partition/ | SiteIndex, FreeRank, SiteBinding; 11 properties |
| 10 | type/ | Constraint hierarchy (6 classes), MetricAxis, 3 axis individuals |
| 11 | resolver/, derivation/ | ResolutionState, RefinementSuggestion, DerivationStep/RefinementStep |
| 12 | morphism/ | Composition, Identity, CompositionLaw; criticalComposition individual |
| 13 | u/, op/ | Content addressing (AD_1/AD_2), AddressHierarchy, 4 properties |
| 14 | op/ | 50 Ring/Boolean/XOR/Dihedral/Universal/Group/Carry identities |
| 15 | op/ | 42 Constraint/Site/Resolution identities |
| 16 | op/ | 52 Observable/Cert/Structure identities |
| 17 | op/ | 64 Transform/Automorphism/Embedding/Analytical identities |
| 18 | observable/, resolver/ | Jacobian, TopologicalObservable, BettiNumber, SpectralGap, CechNerve |
| 19 | op/ | 21 Thermodynamic/Analytical/Partition/Resolution identities |
| 20 | op/ | 6 Differential calculus, Homological algebra, Index theorem identities |
| 21 | op/, homology/, cohomology/ | Structural Reasoning namespaces, ψ-pipeline, identity grounding |
| 22 | morphism/, state/ | TopologicalDelta class, topologicalSnapshot property |
| 23 | op/, resolver/, derivation/, observable/, type/, query/ | Typed controlled vocabularies: 7 new classes, 35 individuals, 12 new properties (includes TriadProjection) |
| 24 | morphism/, query/, type/, cert/ | Surface Grounding Layer, GroundingMap, CompleteType, graph gap closures |
| 25 | type/, resolver/, cert/ | Completeness Certification: CompletenessCandidate, CompletenessWitness, CompletenessResolver |
| 26 | schema/, op/, resolver/ | Witt Level Scaling: W16Ring, WittLevelBinding, WittLevelResolver |
| 27 | state/, resolver/, query/ | Session-Scoped Resolution: Session, SessionBoundary, SessionBoundaryType enum |
| 28 | type/, resolver/, derivation/, observable/ | ψ-Pipeline Inversion (Type Synthesis): TypeSynthesisGoal, TypeSynthesisResult |
| 29 | type/, observable/, resolver/ | Witt Level Spectral Sequence: WittLift, LiftObstruction, SpectralSequencePage |
| 30 | observable/, type/, resolver/ | Monodromy Observables: HolonomyGroup, Monodromy, FlatType, TwistedType |
| 31 | proof/, observable/, resolver/, partition/, trace/ | Ontology Gap Closures: PhaseBoundaryType enum, JacobianGuidedResolver, PT\_/ST\_ identities (EmpiricalVerification eliminated in Amendment 86; proofs reclassified) |
| 32 | type/, resolver/, op/ | RC\_5 Superposition Vocabulary: SuperposedSiteState, SuperpositionResolver, SuperpositionDomain |
| 33 | state/, cert/, resolver/ | Grounded Context Limit: GroundedContext, GroundingWitness, GroundingPhase enum, GS\_1–GS\_7 |
| 34 | proof/, type/, observable/ | Morphospace Achievability: ImpossibilityWitness, MorphospaceRecord, AchievabilityStatus enum, MS\_1–MS\_5 |
| 35 | trace/, cert/, resolver/ | Computational Geodesic: GeodesicTrace, GeodesicViolation, GeodesicCertificate, GD\_1–GD\_5 |
| 36 | resolver/, trace/, cert/, type/, op/ | Measurement Boundary: MeasurementResolver, MeasurementEvent, CollapsedSiteState, QuantumThermodynamicDomain, QM\_1–QM\_4 |
| 37 | partition/, trace/, cert/, type/, schema/, observable/, resolver/, op/ | Ontology Gap Closures: PartitionProduct, PartitionCoproduct, MeasurementOutcome, GeodesicEvidenceBundle, BornRuleVerification |
| 38 | cert/, derivation/ | Q1 Vocabulary: SynthesisCheckpoint, isAR1Ordered, isDC10Selected, checkpointStep, checkpointState |
| 39 | (SHACL only) | Q1 Lift + Inverse Pipeline Coverage: test85–test95 |
| 40 | (SHACL only) | Q1 Normative Certification Coverage: test96–test100 |
| 41 | op/, type/, proof/, resolver/, derivation/, cert/ | Arbitrary Wₙ Scaling: ValidityScopeKind enum, LiftChain, ObstructionChain, InductiveProof, TowerCompletenessResolver, LiftChainCertificate, WT\_1–WT\_7 |
| 42 | (grammar) | UOR Term Grammar Formalization: machine-generated EBNF grammar from ontology |
| 43 | u/ | Cryptographic Primitive Pinning: digestAlgorithm, canonicalBytes, BLAKE3/SHA-256 |
| 44 | op/, type/, observable/, resolver/, proof/, derivation/, trace/, cert/, state/ | Structural Gap Closures (G1–G11): carry-depth pinning (now a `BoundConstraint` kind in v0.2.2 Phase D), joint satisfiability, dihedral algebra, constraint expressiveness, SumType topology, synthesis reachability, obstruction termination, coefficient ring, gluing feedback, session saturation bridge, amplitude index characterization |
| 45 | (conformance) | Self-Auditing Conformance Infrastructure: Three derived meta-validators — certificate issuance coverage (Rule 1), identity-proof bijection (Rule 2), SHACL fixture coverage (Rule 3). Zero new ontology terms. |
| 46 | op/, proof/, conformance/ | Gap Closure: 8 certificate issuance identities (CIC\_1–7, GC\_1) + 36 SHACL fixtures covering all kernel/bridge classes. Zero new ontology classes or properties. |
| 47 | (conformance) | Model-Derived Validation + Meta-Validator Hardening: 5 model-derived meta-validators (property IRI validity, kind/value-type conformance, functional cardinality, IriRef target resolution, datatype range conformance). Harden Rules 1–3, quantum\_scope exhaustive type guard, crate\_ count cross-checks, prf\_CIC\_7 retype. (EmpiricalVerification references removed after Amendment 86 elimination.) |
| 48 | resolver/, state/ | Multi-Session Coordination: SharedContext, ContextLease, SessionComposition, ExecutionPolicy, ExecutionPolicyKind enum (FifoPolicy/MinFreeCountFirst/MaxFreeCountFirst/DisjointFirst), SR\_8–SR\_10 + MC\_1–MC\_8 identities |
| 49 | conformance/ | Conformance Gap Closure: `op:lhs`/`op:rhs`/`proof:forbidsSignature` reclassified to AnnotationProperty; Rule 3 SHACL mandate extended to User-space (type/, morphism/, state/); `MC_` prefix added to identity guard; website class count made dynamic; `validate_forall_scope_alignment` cross-check added; test164–167 fixtures covering Embedding, Action, SessionBoundaryType, MetricAxis |
| 50 | website/ | PRISM Website Refactor: responsive design, pipeline visualization |
| 51 | docs/ | Uniform Navigation: single source of truth in uor-docs |
| 52 | spec/serializer/ | Seven-Format Artifact Suite: OWL RDF/XML, JSON Schema, SHACL Shapes serializers |
| 53 | op/, proof/ | Witt–Carry Formalization: 12 `WC_` Witt-carry bridge identities, 5 `OA_` Ostrowski–Archimedean bridge identities, `ArithmeticValuation` verification domain, CA\_1–CA\_6 reclassified Enumerative → Algebraic |
