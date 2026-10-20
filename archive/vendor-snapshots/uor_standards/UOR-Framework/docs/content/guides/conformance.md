# Conformance Guide

## Overview

The UOR conformance suite validates all workspace artifacts against professional
standards. Run it with:

```sh
cargo run --bin uor-conformance
```

## What Is Validated

### Ontology Conformance

| Artifact | Standard | Validator |
|----------|----------|-----------|
| `public/uor.foundation.jsonld` | JSON-LD 1.1 | `validators/ontology/jsonld.rs` |
| `public/uor.foundation.jsonld` | OWL 2 DL | `validators/ontology/owl.rs` |
| Inventory counts | {@count:namespaces}/{@count:classes}/{@count:properties}/{@count:individuals} | `validators/ontology/inventory.rs` |
| `public/uor.foundation.ttl` | RDF 1.1 / Turtle 1.1 | `validators/ontology/rdf.rs` |
| `public/uor.term.ebnf` | ISO/IEC 14977 EBNF | `validators/ontology/ebnf.rs` |
| {@count:shacl_tests} test instance graphs | SHACL | `validators/ontology/shacl.rs` |

### Documentation Conformance

| Check | Validator |
|-------|-----------|
| All {@count:classes} classes documented | `validators/docs/completeness.rs` |
| Namespace pages accurate | `validators/docs/accuracy.rs` |
| Diataxis structure present | `validators/docs/structure.rs` |
| No broken internal links | `validators/docs/links.rs` |

### Website Conformance

| Check | Standard | Validator |
|-------|----------|-----------|
| HTML5 structure | HTML5 | `validators/website/html.rs` |
| Accessibility | WCAG 2.1 AA | `validators/website/accessibility.rs` |
| Namespace page coverage | — | `validators/website/coverage.rs` |
| CSS validity | CSS | `validators/website/css.rs` |
| Internal links | — | `validators/website/links.rs` |

### SHACL Tests 34–53 (v3.4.0–v4.0.0)

| Test | What It Validates |
|------|-------------------|
| test34 | CompletenessCandidate, CompletenessWitness, CompletenessResolver (Amendment 25) |
| test35 | CompletenessCertificate, CompletenessAuditTrail, witnessCount (Amendment 25) |
| test36 | W16Ring, W16bitWidth, W16capacity, nextWittLevel chain (Amendment 26) |
| test37 | WittLevelBinding, universallyValid, verifiedAtLevel (Amendment 26) |
| test38 | Session, BindingAccumulator, SessionResolver, SessionQuery (Amendment 27) |
| test39 | SessionBoundary, SessionBoundaryType vocabulary individuals (Amendment 27) |
| test40 | TypeSynthesisGoal, TypeSynthesisResolver, synthesisGoal (Amendment 28) |
| test41 | Full synthesis round-trip: Goal→Resolver→Result→SynthesizedType→MinimalConstraintBasis→SynthesisSignature→SynthesisStep (Amendment 28) |
| test42 | QuantumLift, LiftObstruction (obstructionTrivial=true), IncrementalCompletenessResolver (Amendment 29) |
| test43 | SpectralSequencePage: page 1 (differentialIsZero=false) → page 2 (convergedAt=2) (Amendment 29) |
| test44 | FlatType + HolonomyGroup (order=1) + Monodromy (isTrivialMonodromy=true) + ClosedConstraintPath (Amendment 30) |
| test45 | TwistedType + non-trivial HolonomyGroup + LiftObstruction (obstructionTrivial=false) + LiftObstructionClass + DihedralElement (Amendment 30) |
| test46 | MonodromyResolver end-to-end pipeline: ConstrainedType → HolonomyGroup → MonodromyClass → TwistedType (Amendment 30) |
| test47 | ThermoObservable + hardnessEstimate + ComputationTrace + residualEntropy (Amendment 31) |
| test48 | CatastropheObservable + phaseN/phaseG + PhaseBoundaryType + onResonanceLine (Amendment 31) |
| test49 | FreeRank + SiteIndex + ancillaSite + reversibleStrategy (Amendment 31) |
| test50 | JacobianGuidedResolver + ResolutionState + guidingJacobian (Amendment 31) |
| test51 | ProductType + component assertions + FreeRank (Amendment 31) |
| test52 | SumType + variant assertions (Amendment 31) |
| test53 | SuperposedSiteState + amplitude + SuperpositionResolver (Amendment 32) |

### SHACL Tests 54–100 (v4.1.0–v5.0.0)

| Test | What It Validates |
|------|-------------------|
| test54 | GroundedContext + groundingDegree + contextTemperature + isGrounded (Amendment 33) |
| test55 | GroundingWitness + witnessBinding + witnessStep + residualFreeRank (Amendment 33) |
| test56 | DomainGroundingRecord + groundedDomain + domainFreeRank (Amendment 33) |
| test57 | GroundingPhase vocabulary: Open, PartialGrounding, FullGrounding (Amendment 33) |
| test58 | GroundingCertificate + certifiedGrounding + groundingWitness (Amendment 33) |
| test59 | GroundingAwareResolver + usedGrounding (Amendment 33) |
| test60 | ImpossibilityWitness + forbidsSignature + impossibilityReason (Amendment 34) |
| test61 | MorphospaceRecord + achievabilityStatus + verifiedAtLevel (Amendment 34) |
| test62 | MorphospaceBoundary + boundaryType (Amendment 34) |
| test63 | ForbiddenSignature + targetForbidden (Amendment 34) |
| test64 | AchievabilityStatus vocabulary: Achievable, Forbidden (Amendment 34) |
| test65 | GeodesicTrace + isGeodesic + geodesicCertificate + stepEntropyCost (Amendment 35) |
| test66 | GeodesicCertificate + certifiedGeodesic + geodesicTrace (Amendment 35) |
| test67 | GeodesicViolation + violationReason (Amendment 35) |
| test68 | GeodesicValidator + validateGeodesic (Amendment 35) |
| test69 | GeodesicTrace + adiabaticallyOrdered + jacobianAtStep (Amendment 35) |
| test70 | MeasurementResolver + collapseAmplitude + collapsedSite (Amendment 36) |
| test71 | MeasurementEvent + preCollapseEntropy + postCollapseLandauerCost (Amendment 36) |
| test72 | MeasurementCertificate + certifiedMeasurement + vonNeumannEntropy + landauerCost (Amendment 36) |
| test73 | CollapsedSiteState + collapsedFrom + survivingAmplitude (Amendment 36) |
| test74 | QuantumThermodynamicDomain + QuantumThermodynamic verification domain (Amendment 36) |

### SHACL Tests 75–84 (v4.2.0)

| Test | What It Validates |
|------|-------------------|
| test75 | PartitionProduct + leftFactor + rightFactor + exteriorCriteria (Amendment 37) |
| test76 | PartitionCoproduct + leftSummand + rightSummand + isExhaustive (Amendment 37) |
| test77 | MeasurementOutcome + outcomeValue + outcomeProbability (Amendment 37) |
| test78 | GeodesicEvidenceBundle + evidenceBundle + isAR1Ordered + isDC10Selected (Amendment 37) |
| test79 | BornRuleVerification + bornRuleVerified (Amendment 37) |
| test80 | NormativeComputationType + normalizationVerified + holonomyClassified (Amendment 37) |
| test81 | SpectralSequencePage + wittLevelPredecessor linkage (Amendment 37) |
| test82 | amplitudeVector on resolver + priorAmplitudeVector (Amendment 37) |
| test83 | amplitudeVector on trace + rotationExponent + reflectionBit (Amendment 37) |
| test84 | VerificationDomain individual with enumVariant annotation (Amendment 37) |

### SHACL Tests 85–100 (v5.0.0)

| Test | What It Validates |
|------|-------------------|
| test85 | W16Ring individual grounding at Witt level W16 (Amendment 39) |
| test86 | QuantumLift with trivial LiftObstruction — Q1 lift (Amendment 39) |
| test87 | SpectralSequencePage convergence at E2 — Q1 scale (Amendment 39) |
| test88 | Non-trivial LiftObstruction with TwistedType — Q1 scale (Amendment 39) |
| test89 | LiftRefinementSuggestion with obstructionClass — Q1 failure path (Amendment 39) |
| test90 | Resolved lift — SynthesizedType at Q1 with basisSize increment (Amendment 39) |
| test91 | TypeSynthesisGoal with Q1 target signature (Amendment 39) |
| test92 | SynthesisCheckpoint with checkpointStep and checkpointState (Amendment 38/39) |
| test93 | SynthesisSignature with achievabilityStatus — Q1 scale (Amendment 39) |
| test94 | SynthesizedType with TypeSynthesisResult + MinimalConstraintBasis (Amendment 39) |
| test95 | Unreachable signature rejection — Forbidden status (Amendment 39) |
| test96 | GeodesicTrace at Q1 ring scale with GeodesicCertificate (Amendment 40) |
| test97 | GeodesicEvidenceBundle with isAR1Ordered (Amendment 40) |
| test98 | GeodesicEvidenceBundle with isDC10Selected (Amendment 40) |
| test99 | MeasurementCertificate with BornRuleVerification at Q1 (Amendment 40) |
| test100 | Full normative chain — Trace → Certificate → EvidenceBundle (Amendment 40) |

### SHACL Tests 101–110 (v5.1.0)

| Test | What It Validates |
|------|-------------------|
| test101 | Flat LiftChain Q0→Q3 with trivial ObstructionChain (Amendment 41) |
| test102 | Twisted LiftChain Q0→Q2 with non-trivial LiftObstruction (Amendment 41) |
| test103 | ObstructionChain with obstructionCount=0 and isFlat=true (Amendment 41) |
| test104 | ObstructionChain with 2 obstructionAt assertions, isFlat=false (Amendment 41) |
| test105 | LiftChainCertificate with certifiedChain, chainAuditTrail, levels (Amendment 41) |
| test106 | ChainAuditTrail with chainStepCount matching chain (Amendment 41) |
| test107 | TowerCompletenessResolver with Q0 source, Q47 target (Amendment 41) |
| test108 | InductiveProof with baseCase, inductiveStep, validForKAtLeast (Amendment 41) |
| test109 | Identity with validityKind=ParametricLower, validKMin=3 (Amendment 41) |
| test110 | Full tower round-trip: Q0 → LiftChain → LiftChainCertificate → Q\_k (Amendment 41) |

### SHACL Tests 111–112 (v5.2.0)

| Test | What It Validates |
|------|-------------------|
| test111 | Address with digestAlgorithm, canonicalBytes, blake3-prefixed digest (Amendment 43) |
| test112 | Address at Q1 with 6-byte canonical form and sha256 algorithm (Amendment 43) |

### SHACL Tests 113–123 (v5.3.0)

| Test | What It Validates |
|------|-------------------|
| test113 | CC\_PINS and CC\_COST\_SITE identity grounding (Amendment 44) |
| test114 | jsat\_RR, jsat\_CR, jsat\_CC identity grounding (Amendment 44) |
| test115 | D\_8 and D\_9 identity grounding (Amendment 44) |
| test116 | EXP\_1, EXP\_2, EXP\_3 identity grounding (Amendment 44) |
| test117 | ST\_3, ST\_4, ST\_5 identity grounding (Amendment 44) |
| test118 | TS\_8, TS\_9, TS\_10 identity grounding (Amendment 44) |
| test119 | QT\_8 and QT\_9 identity grounding (Amendment 44) |
| test120 | COEFF\_1 identity grounding (Amendment 44) |
| test121 | GO\_1 identity grounding (Amendment 44) |
| test122 | SR\_6 and SR\_7 identity grounding (Amendment 44) |
| test123 | QM\_6 identity grounding (Amendment 44) |

### SHACL Tests 124–157 (v5.4.0)

| Test | What It Validates |
|------|-------------------|
| test124 | (removed — Glyph deleted from foundation) |
| test125 | op:DihedralGroup class (Amendment 45) |
| test126 | op:ValidityScopeKind enum class (Amendment 45) |
| test127 | resolver:WittLevelResolver class (Amendment 45) |
| test128 | observable:StratumObservable class (Amendment 45) |
| test129 | observable:MetricObservable class (Amendment 45) |
| test130 | observable:PathObservable class (Amendment 45) |
| test131 | observable:ReductionObservable class (Amendment 45) |
| test132 | observable:HolonomyObservable class (Amendment 45) |
| test133 | observable:IncompatibilityMetric class (Amendment 45) |
| test134 | observable:StratumValue class (Amendment 45) |
| test135 | observable:StratumDelta class (Amendment 45) |
| test136 | observable:StratumTrajectory class (Amendment 45) |
| test137 | observable:PathLength class (Amendment 45) |
| test138 | observable:TotalVariation class (Amendment 45) |
| test139 | observable:WindingNumber class (Amendment 45) |
| test140 | observable:ReductionLength class (Amendment 45) |
| test141 | observable:ReductionCount class (Amendment 45) |
| test142 | observable:CatastropheThreshold class (Amendment 45) |
| test143 | observable:CatastropheCount class (Amendment 45) |
| test144 | observable:Commutator class (Amendment 45) |
| test145 | observable:CurvatureFlux class (Amendment 45) |
| test146 | observable:ParallelTransport class (Amendment 45) |
| test147 | observable:ReductionEntropy class (Amendment 45) |
| test148 | observable:PhaseBoundaryType enum class (Amendment 45) |
| test149 | homology:FaceMap class (Amendment 45) |
| test150 | homology:NerveFunctor class (Amendment 45) |
| test151 | homology:ChainFunctor class (Amendment 45) |
| test152 | cohomology:RestrictionMap class (Amendment 45) |
| test153 | proof:CoherenceProof class (Amendment 45) |
| test154 | derivation:DerivationStep class (Amendment 45) |
| test155 | trace:ComputationStep class (Amendment 45) |
| test156 | trace:TraceMetrics class (Amendment 45) |
| test157 | cert:IsometryCertificate class (Amendment 45) |

### SHACL Tests 158–159 (v5.4.0)

| Test | What It Validates |
|------|-------------------|
| test158 | cert:LiftChainCertificate with verified + chainStepCount (Amendment 46) |
| test159 | cert:ChainAuditTrail class (Amendment 46) |

### SHACL Tests 160–163 (v5.6.0)

| Test | What It Validates |
|------|-------------------|
| test160 | state:SharedContext + state:ContextLease multi-session leasing with two site-disjoint leases (Amendment 48) |
| test161 | resolver:ExecutionPolicy + resolver:ExecutionPolicyKind scheduling vocabulary (MinFreeCountFirst individual) (Amendment 48) |
| test162 | state:SessionComposition with composedFrom, compositionCompatible, compositionResult, towerConsistencyVerified (Amendment 48) |
| test163 | Distributed grounding: SharedContext → two ContextLeases → SessionComposition → GroundedContext (Amendment 48) |

### SHACL Tests 164–167 (v6.0.0)

| Test | What It Validates |
|------|-------------------|
| test164 | `morphism:Embedding` with `sourceQuantum` and `targetQuantum` (Amendment 49) |
| test165 | `morphism:Action` with `morphism:group=op:DihedralGroup` and `actionIsometry` (Amendment 49) |
| test166 | `state:SessionBoundaryType` vocabulary — `ExplicitReset` named individual (Amendment 49) |
| test167 | `type:MetricAxis` vocabulary — `verticalAxis` named individual (Amendment 49) |

## Adding a New SHACL Test

1. Create `conformance/src/tests/fixtures/test<n>_<name>.rs`
2. Define a `pub const TEST<N>_<NAME>: &str = r#"..."#;` with Turtle source
3. Export it from `conformance/src/tests/fixtures/mod.rs`
4. Register it in `conformance/src/validators/ontology/shacl.rs`
5. Add a check function `validate_<name>(src: &str) -> Result<(), String>`

## Running Individual Validators

The conformance library is structured so each validator can be called independently:

```rust
use uor_conformance::validators::ontology::owl;

let report = owl::validate();
assert!(report.all_passed());
```

## CI Integration

The CI workflow runs full conformance as the last step:

```yaml
- run: cargo run --bin uor-conformance  # exits non-zero on failure
```
