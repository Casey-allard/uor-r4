//! `proof/` namespace — Verification proof structures.
//!
//! Proofs are kernel-produced attestations of algebraic properties. The
//! critical proof asserts the foundational theorem `neg(bnot(x)) = succ(x)`.
//!
//! Amendment 48 adds 11 proofs: prf_SR_8–10 (multi-session coordination axioms)
//! and prf_MC_1–8 (derivational scaling theorem proofs).
//!
//! Amendment 53 adds 17 proofs: prf_WC_1\u{2013}12 (Witt-carry bridge) and
//! prf_OA_1\u{2013}5 (Ostrowski\u{2013}Archimedean bridge).
//!
//! Amendment 58 adds 7 proofs: prf_CY_1\u{2013}7 (carry algebra).
//!
//! Amendment 59 adds 6 proofs: prf_BM_1\u{2013}6 (named base metrics).
//!
//! Amendment 60 adds 8 proofs: prf_GL_1\u{2013}4 (Galois connection) and
//! prf_NV_1\u{2013}4 (nerve operations).
//!
//! Amendment 61 adds 8 proofs: prf_SD_1\u{2013}8 (structural types).
//!
//! Amendment 62 adds 18 proofs: prf_DD_1\u{2013}2, prf_PI_1\u{2013}5,
//! prf_PA_1\u{2013}5, prf_PL_1\u{2013}3, prf_PK_1\u{2013}2, prf_PP_1
//! (composed operations).
//!
//! Amendment 63 adds 16 proofs: prf_PE_1\u{2013}7 (pipeline evaluation),
//! prf_PM_1\u{2013}7 (machine execution), prf_ER_1\u{2013}2 (execution rules).
//!
//! Amendment 64 adds 16 proofs: prf_ER_3\u{2013}4 (execution rules),
//! prf_EA_1\u{2013}4 (epoch admission), prf_OE_1\u{2013}3 (optimization
//! equivalences), prf_OE_4a\u{2013}4c (sub-lemmas), prf_CS_1\u{2013}4
//! (cost semantics).
//!
//! Amendment 65 adds 16 proofs: prf_CS_5 (cost bound), prf_FA_1\u{2013}3
//! (scheduler fairness), prf_SW_1\u{2013}4 (service window), prf_LS_1\u{2013}4
//! (lease suspension), prf_TJ_1\u{2013}3 (transaction), prf_AP_1\u{2013}3
//! (approximation).
//!
//! Amendment 66 adds 8 proofs: prf_EC_1\u{2013}5, prf_EC_4a\u{2013}4c
//! (convergence tower).
//!
//! Amendment 67 adds 7 proofs: prf_DA_1\u{2013}7 (division algebras).
//!
//! Amendment 84 adds 2 proofs: prf_CS_6\u{2013}7 (compile unit identities).
//!
//! **Space classification:** `bridge` — kernel-produced, user-consumed.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `proof/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "proof",
            iri: NS_PROOF,
            label: "UOR Proofs",
            comment: "Kernel-produced verification proofs attesting to algebraic \
                      properties of UOR objects and operations.",
            space: Space::Bridge,
            imports: &[NS_SCHEMA, NS_OP, NS_DERIVATION, NS_OBSERVABLE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/proof/Proof",
            label: "Proof",
            comment: "A kernel-produced attestation that a given algebraic property \
                      holds. The root class for all proof types.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/CoherenceProof",
            label: "CoherenceProof",
            comment: "A proof of coherence: the type system and ring structure are \
                      mutually consistent at a given quantum level.",
            subclass_of: &["https://uor.foundation/proof/Proof"],
            disjoint_with: &[],
        },
        // v3.2: Proof modality classes
        Class {
            id: "https://uor.foundation/proof/ComputationCertificate",
            label: "ComputationCertificate",
            comment: "A proof confirmed by exhaustive execution over R_n at a specific \
                      quantum level. The kernel ran the identity against all 2^n inputs \
                      and observed that it holds. The proof:atWittLevel property \
                      records the level; proof:witness links to the WitnessData. \
                      CriticalIdentityProof is a subclass of ComputationCertificate.",
            subclass_of: &["https://uor.foundation/proof/Proof"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "AxiomaticDerivation",
            comment: "A proof that follows from previously established axioms or \
                      definitions by equational, structural, or topological reasoning. \
                      The proof:derivationWitness property links to a \
                      derivation:Derivation individual recording the rewrite chain. \
                      All pipeline, constraint, observable, and topological identities \
                      are AxiomaticDerivations.",
            subclass_of: &["https://uor.foundation/proof/Proof"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/CriticalIdentityProof",
            label: "CriticalIdentityProof",
            comment: "A proof of the critical identity: neg(bnot(x)) = succ(x) \
                      for all x in R_n. This is the foundational theorem of the \
                      UOR kernel.",
            subclass_of: &["https://uor.foundation/proof/ComputationCertificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/WitnessData",
            label: "WitnessData",
            comment: "Supporting data for a proof: specific examples, counter-examples \
                      checked, or intermediate computation results.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 31/86: EmpiricalVerification class removed in Amendment 86.
        // All 4 former EmpiricalVerification proofs reclassified to
        // InductiveProof (AR_5, QM_6) or AxiomaticDerivation (OA_4, CIC_7).
        // Amendment 34: Morphospace Achievability
        Class {
            id: "https://uor.foundation/proof/ImpossibilityWitness",
            label: "ImpossibilityWitness",
            comment: "A formal witness that a topological signature (χ, β_k) is \
                      impossible to achieve for any ConstrainedType. Carries the \
                      algebraic reason and the verification domain grounding the \
                      impossibility.",
            subclass_of: &["https://uor.foundation/proof/Proof"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/MorphospaceRecord",
            label: "MorphospaceRecord",
            comment: "A formal record of a morphospace boundary point — either an \
                      achievable or forbidden topological signature. Aggregated by \
                      MorphospaceBoundary to form the queryable morphospace map.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/MorphospaceBoundary",
            label: "MorphospaceBoundary",
            comment: "An aggregate of ImpossibilityWitness instances forming the \
                      queryable morphospace map. SPARQL over this structure \
                      answers achievability queries in O(1).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 41: InductiveProof
        Class {
            id: "https://uor.foundation/proof/InductiveProof",
            label: "InductiveProof",
            comment: "A proof by structural induction on the quantum level index k. \
                      Carries a base case proof, an inductive step proof, and the \
                      minimum k for which the induction holds.",
            subclass_of: &["https://uor.foundation/proof/Proof"],
            disjoint_with: &[],
        },
        // Amendment 87: ProofStrategy controlled vocabulary
        Class {
            id: "https://uor.foundation/proof/ProofStrategy",
            label: "ProofStrategy",
            comment: "A controlled vocabulary of proof methods. Each proof individual \
                      carries exactly one strategy from this vocabulary, enabling \
                      compilation to verified theorem provers.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 92: DerivationTerm class hierarchy for proof construction AST
        Class {
            id: "https://uor.foundation/proof/DerivationTerm",
            label: "DerivationTerm",
            comment: "Root AST node for proof construction terms. Distinct from \
                      schema:TermExpression which represents mathematical terms; \
                      DerivationTerm represents proof constructions (tactic \
                      applications, lemma invocations, induction scaffolding).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/TacticApplication",
            label: "TacticApplication",
            comment: "A proof step applying a named tactic (from ProofStrategy) \
                      with arguments. Maps to a Lean4 tactic invocation.",
            subclass_of: &["https://uor.foundation/proof/DerivationTerm"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/LemmaInvocation",
            label: "LemmaInvocation",
            comment: "A proof step invoking a previously proved identity as a \
                      lemma. References the identity via proof:dependsOn.",
            subclass_of: &["https://uor.foundation/proof/DerivationTerm"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/InductionStep",
            label: "InductionStep",
            comment: "A proof step performing structural induction: base case \
                      derivation, inductive hypothesis, and step derivation.",
            subclass_of: &["https://uor.foundation/proof/DerivationTerm"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/proof/ComputationStep",
            label: "ComputationStep",
            comment: "A proof step performing exhaustive computation at a specific \
                      quantum level as verification witness.",
            subclass_of: &["https://uor.foundation/proof/DerivationTerm"],
            disjoint_with: &[],
        },
        // v0.2.1: Inhabitance Verdict Instantiation
        Class {
            id: "https://uor.foundation/proof/InhabitanceImpossibilityWitness",
            label: "InhabitanceImpossibilityWitness",
            comment: "A specialisation of proof:ImpossibilityWitness produced \
                      when the inhabitance search determines that the carrier \
                      of a ConstrainedType is empty. Aggregates into the \
                      existing proof:MorphospaceBoundary alongside other \
                      impossibility witnesses, inheriting its O(1) amortised \
                      lookup discipline for previously resolved signatures.",
            subclass_of: &["https://uor.foundation/proof/ImpossibilityWitness"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/proof/verified",
            label: "verified",
            comment: "Whether this proof has been verified by the kernel.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/proof/timestamp",
            label: "timestamp",
            comment: "The time at which this proof was produced.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: XSD_DATETIME,
        },
        Property {
            id: "https://uor.foundation/proof/witness",
            label: "witness",
            comment: "Supporting witness data for this proof.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/proof/WitnessData",
        },
        Property {
            id: "https://uor.foundation/proof/criticalIdentity",
            label: "criticalIdentity",
            comment: "Human-readable statement of the critical identity proven. \
                      E.g., 'neg(bnot(x)) = succ(x) for all x in R_n'. \
                      Annotation only — proof:provesIdentity is the typed reference.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/CriticalIdentityProof"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/proof/x",
            label: "x",
            comment: "A specific input value used as a witness for the critical \
                      identity check.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/WitnessData"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/proof/bnot_x",
            label: "bnot_x",
            comment: "The value bnot(x) for a witness x.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/WitnessData"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/proof/neg_bnot_x",
            label: "neg_bnot_x",
            comment: "The value neg(bnot(x)) for a witness x.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/WitnessData"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/proof/succ_x",
            label: "succ_x",
            comment: "The value succ(x) for a witness x.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/WitnessData"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/proof/holds",
            label: "holds",
            comment: "Whether the identity neg(bnot(x)) = succ(x) holds for \
                      this specific witness.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/WitnessData"),
            range: XSD_BOOLEAN,
        },
        // Amendment 3: provesIdentity — object property linking to op:Identity
        Property {
            id: "https://uor.foundation/proof/provesIdentity",
            label: "provesIdentity",
            comment: "The algebraic identity this proof establishes. Provides a \
                      canonical object reference alongside the existing \
                      proof:criticalIdentity string property, which remains for \
                      human readability.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/op/Identity",
        },
        // v3.2: ComputationCertificate properties
        Property {
            id: "https://uor.foundation/proof/atWittLevel",
            label: "atWittLevel",
            comment: "The quantum level at which this computation certificate was \
                      produced. A ComputationCertificate at schema:Q0 confirms the \
                      identity holds for all 256 inputs of R_8. A certificate at \
                      schema:Q1 confirms it for all 65,536 inputs of R_16.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ComputationCertificate"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/proof/wittNote",
            label: "wittNote",
            comment: "Human-readable quantum level note, e.g. 'n=8, 256 inputs'. \
                      Annotation only — proof:atWittLevel is the typed assertion.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ComputationCertificate"),
            range: XSD_STRING,
        },
        // v3.2: AxiomaticDerivation properties
        Property {
            id: "https://uor.foundation/proof/universalScope",
            label: "universalScope",
            comment: "True when this axiomatic derivation holds for all quantum levels \
                      by the definition of Z/(2^n)Z. False when the derivation depends \
                      on a property specific to a particular ring size. All current \
                      AxiomaticDerivation individuals in the spec carry universalScope \
                      true.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/AxiomaticDerivation"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/proof/derivationWitness",
            label: "derivationWitness",
            comment: "The derivation chain that witnesses this axiomatic derivation. \
                      Links a proof:AxiomaticDerivation to the derivation:Derivation \
                      individual recording the rewrite sequence. Optional at the spec \
                      level — the conformance suite requires only that the proof \
                      individual exists; full derivation chains live in generated \
                      artifacts.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/AxiomaticDerivation"),
            range: "https://uor.foundation/derivation/Derivation",
        },
        // Amendment 31/86: quantumLevelRange, verificationMethod, and verifiedAt
        // removed in Amendment 86 (EmpiricalVerification elimination).
        // Amendment 34: Morphospace Achievability
        Property {
            id: "https://uor.foundation/proof/forbidsSignature",
            label: "forbidsSignature",
            comment: "The topological signature (χ, β_k) that this \
                      ImpossibilityWitness formally forbids, expressed as a \
                      symbolic notation string (e.g., 'β₀ = 0').",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ImpossibilityWitness"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/proof/impossibilityReason",
            label: "impossibilityReason",
            comment: "Human-readable statement of the algebraic reason the \
                      signature is impossible (e.g., 'β₀ = 0 violates MS_1').",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ImpossibilityWitness"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/proof/impossibilityDomain",
            label: "impossibilityDomain",
            comment: "The verification domain grounding the impossibility \
                      (e.g., Pipeline for β₀ = 0, Algebraic for χ > n).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ImpossibilityWitness"),
            range: "https://uor.foundation/op/VerificationDomain",
        },
        // Amendment 86: domain reassigned from EmpiricalVerification to ImpossibilityWitness
        Property {
            id: "https://uor.foundation/proof/achievabilityStatus",
            label: "achievabilityStatus",
            comment: "The achievability classification of a proof-linked observable \
                      signature: Achievable or Forbidden.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/ImpossibilityWitness"),
            range: "https://uor.foundation/observable/AchievabilityStatus",
        },
        Property {
            id: "https://uor.foundation/proof/verifiedAtLevel",
            label: "verifiedAtLevel",
            comment: "The specific quantum level at which an empirical verification \
                      or impossibility witness was established.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/proof/morphospaceRecord",
            label: "morphospaceRecord",
            comment: "Links a MorphospaceBoundary to one of its constituent \
                      MorphospaceRecord individuals.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/MorphospaceBoundary"),
            range: "https://uor.foundation/proof/MorphospaceRecord",
        },
        Property {
            id: "https://uor.foundation/proof/boundaryType",
            label: "boundaryType",
            comment: "Whether this MorphospaceRecord represents an impossibility \
                      boundary (from below) or an achievability boundary (from above).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/MorphospaceRecord"),
            range: "https://uor.foundation/observable/AchievabilityStatus",
        },
        // Amendment 41: InductiveProof properties
        Property {
            id: "https://uor.foundation/proof/baseCase",
            label: "baseCase",
            comment: "The proof that the claim holds at the base level k_0.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InductiveProof"),
            range: "https://uor.foundation/proof/Proof",
        },
        Property {
            id: "https://uor.foundation/proof/inductiveStep",
            label: "inductiveStep",
            comment: "The proof that if the claim holds at Q_k, it holds at Q_{k+1}.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InductiveProof"),
            range: "https://uor.foundation/proof/Proof",
        },
        Property {
            id: "https://uor.foundation/proof/validForKAtLeast",
            label: "validForKAtLeast",
            comment: "The minimum k for which the induction is valid.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InductiveProof"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 87: Proof enrichment properties
        Property {
            id: "https://uor.foundation/proof/strategy",
            label: "strategy",
            comment: "The proof method from the ProofStrategy controlled vocabulary. \
                      Determines the compilation target (e.g., `by ring`, `by simp`, \
                      `by induction`).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/proof/ProofStrategy",
        },
        Property {
            id: "https://uor.foundation/proof/dependsOn",
            label: "dependsOn",
            comment: "An identity that this proof depends on as a lemma. Forms the \
                      proof dependency DAG. Leaf proofs (provable from definitions \
                      alone) have no dependsOn assertions.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/op/Identity",
        },
        // Amendment 92: formalDerivation retyped to ObjectProperty
        Property {
            id: "https://uor.foundation/proof/formalDerivation",
            label: "formalDerivation",
            comment: "The formal proof construction term: a DerivationTerm AST \
                      node encoding the tactic script, lemma chain, or induction \
                      scaffold that constitutes the proof.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/Proof"),
            range: "https://uor.foundation/proof/DerivationTerm",
        },
        // v0.2.1: InhabitanceImpossibilityWitness properties
        Property {
            id: "https://uor.foundation/proof/contradictionProof",
            label: "contradictionProof",
            comment: "The Lean 4 by-contradiction derivation over the predicate \
                      vocabulary attesting that no value tuple satisfies the \
                      constraint system.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InhabitanceImpossibilityWitness"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/proof/grounded",
            label: "grounded",
            comment: "The type:ConstrainedType whose carrier this witness \
                      certifies as empty.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InhabitanceImpossibilityWitness"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/proof/searchTrace",
            label: "searchTrace",
            comment: "The audit trail of the inhabitance search up to the \
                      contradiction.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/proof/InhabitanceImpossibilityWitness"),
            range: "https://uor.foundation/trace/InhabitanceSearchTrace",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        // Amendment 87: ProofStrategy controlled vocabulary individuals
        Individual {
            id: "https://uor.foundation/proof/RingAxiom",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "RingAxiom",
            comment: "Follows from ZMod ring axioms. Lean4 tactic: `by ring`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/DecideQ0",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "DecideQ0",
            comment: "Decidable at Q0 by exhaustive evaluation. Lean4: `by native_decide`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/BitwiseInduction",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "BitwiseInduction",
            comment: "Induction on bit width n. Lean4: `by induction n`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/GroupPresentation",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "GroupPresentation",
            comment: "From dihedral group presentation. Lean4: `by group`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/Simplification",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "Simplification",
            comment: "By simplification with cited lemmas. Lean4: `by simp [lemmalist]`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/ChineseRemainder",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "ChineseRemainder",
            comment: "By Chinese Remainder Theorem. Lean4: `by exact ZMod.chineseRemainder ...`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/EulerPoincare",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "EulerPoincare",
            comment: "By Euler-Poincare formula applied to the constraint nerve.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/ProductFormula",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "ProductFormula",
            comment: "By Ostrowski product formula or derived valuation arguments.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/Composition",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "Composition",
            comment: "By composing proofs of sub-identities. Lean4: `by exact ...`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/Contradiction",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "Contradiction",
            comment:
                "By deriving contradiction for impossibility witnesses. Lean4: `by contradiction`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/proof/Computation",
            type_: "https://uor.foundation/proof/ProofStrategy",
            label: "Computation",
            comment: "By computation at a specified quantum level. Lean4: `by native_decide`.",
            properties: &[],
        },
        // Critical identity: computation certificate (CriticalIdentityProof at Q0)
        Individual {
            id: "https://uor.foundation/proof/prf_criticalIdentity",
            type_: "https://uor.foundation/proof/CriticalIdentityProof",
            label: "prf_criticalIdentity",
            comment: "Computation certificate for the critical identity \
                      neg(bnot(x)) = succ(x) at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/criticalIdentity"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        // Critical identity: axiomatic derivation (dual proof)
        Individual {
            id: "https://uor.foundation/proof/prf_criticalIdentity_axiomatic",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_criticalIdentity_axiomatic",
            comment: "Axiomatic derivation of the critical identity \
                      neg(bnot(x)) = succ(x). Holds at all quantum levels.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/criticalIdentity"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // phi_1 through phi_6: computation certificates at Q0
        Individual {
            id: "https://uor.foundation/proof/prf_phi_1",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_1",
            comment: "Computation certificate for phi_1 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_1"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_phi_2",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_2",
            comment: "Computation certificate for phi_2 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_2"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_phi_3",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_3",
            comment: "Computation certificate for phi_3 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_3"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_phi_4",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_4",
            comment: "Computation certificate for phi_4 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_4"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_phi_5",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_5",
            comment: "Computation certificate for phi_5 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_5"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_phi_6",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_phi_6",
            comment: "Computation certificate for phi_6 at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/phi_6"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        // Axiomatic derivations for all remaining op identities
        Individual {
            id: "https://uor.foundation/proof/prf_AD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AD_1",
            comment: "Axiomatic derivation of AD_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AD_2",
            comment: "Axiomatic derivation of AD_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A1",
            comment: "Axiomatic derivation of R_A1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A2",
            comment: "Axiomatic derivation of R_A2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A3",
            comment: "Axiomatic derivation of R_A3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A4",
            comment: "Axiomatic derivation of R_A4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A5",
            comment: "Axiomatic derivation of R_A5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_A6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_A6",
            comment: "Axiomatic derivation of R_A6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_A6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_M1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_M1",
            comment: "Axiomatic derivation of R_M1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_M1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_M2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_M2",
            comment: "Axiomatic derivation of R_M2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_M2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_M3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_M3",
            comment: "Axiomatic derivation of R_M3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_M3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_M4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_M4",
            comment: "Axiomatic derivation of R_M4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_M4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_R_M5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_R_M5",
            comment: "Axiomatic derivation of R_M5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/R_M5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ADR-053 ring-axis completion: proofs for the 7 Euclidean-division
        // and modular-exponentiation identities. Each is an axiomatic
        // derivation grounded in the Euclidean-algorithm and square-and-
        // multiply construction of `pipeline::apply_primitive_op` /
        // `const_ring_eval_w{n}` per ADR-050.
        Individual {
            id: "https://uor.foundation/proof/prf_DV_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DV_1",
            comment: "Axiomatic derivation of DV_1 (div right-identity). Holds at \
                      every quantum level: floor(a / 1) = a.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DV_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DV_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DV_2",
            comment: "Axiomatic derivation of DV_2 (div left-absorbing). Holds by \
                      the Euclidean-algorithm rule floor(0 / b) = 0 for b ≠ 0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DV_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DV_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DV_3",
            comment: "Axiomatic derivation of DV_3 (div-of-mul recovery). Holds \
                      whenever mul(a, b) is in the unit cone (no overflow); the \
                      Euclidean-algorithm yields q = a and r = 0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DV_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DV_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DV_4",
            comment: "Axiomatic derivation of DV_4 (Euclidean compatibility). \
                      Direct consequence of the Euclidean-algorithm definition: \
                      a = q·b + r with 0 ≤ r < b.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DV_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PW_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PW_1",
            comment: "Axiomatic derivation of PW_1 (zero-exp identity). Base case \
                      of the square-and-multiply construction: empty product = 1.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PW_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PW_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PW_2",
            comment: "Axiomatic derivation of PW_2 (unit-exp identity). Follows \
                      from PW_1 + one square-and-multiply step: a · 1 = a.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PW_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PW_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PW_3",
            comment: "Axiomatic derivation of PW_3 (additive-exp decomposition). \
                      Square-and-multiply reduces pow(a, b+c) to interleaved \
                      multiplications of pow(a, b) and pow(a, c) within the ring.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PW_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_1",
            comment: "Axiomatic derivation of B_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_2",
            comment: "Axiomatic derivation of B_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_3",
            comment: "Axiomatic derivation of B_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_4",
            comment: "Axiomatic derivation of B_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_5",
            comment: "Axiomatic derivation of B_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_6",
            comment: "Axiomatic derivation of B_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_7",
            comment: "Axiomatic derivation of B_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_8",
            comment: "Axiomatic derivation of B_8. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_9",
            comment: "Axiomatic derivation of B_9. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_10",
            comment: "Axiomatic derivation of B_10. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_11",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_11",
            comment: "Axiomatic derivation of B_11. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_11"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_12",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_12",
            comment: "Axiomatic derivation of B_12. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_12"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_B_13",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_B_13",
            comment: "Axiomatic derivation of B_13. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/B_13"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_1",
            comment: "Axiomatic derivation of X_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_2",
            comment: "Axiomatic derivation of X_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_3",
            comment: "Axiomatic derivation of X_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_4",
            comment: "Axiomatic derivation of X_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_5",
            comment: "Axiomatic derivation of X_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_6",
            comment: "Axiomatic derivation of X_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_X_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_X_7",
            comment: "Axiomatic derivation of X_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/X_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_1",
            comment: "Axiomatic derivation of D_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_3",
            comment: "Axiomatic derivation of D_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_4",
            comment: "Axiomatic derivation of D_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_5",
            comment: "Axiomatic derivation of D_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_U_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_U_1",
            comment: "Axiomatic derivation of U_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/U_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ChineseRemainder"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_U_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_U_2",
            comment: "Axiomatic derivation of U_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/U_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ChineseRemainder"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_U_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_U_3",
            comment: "Axiomatic derivation of U_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/U_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ChineseRemainder"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_U_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_U_4",
            comment: "Axiomatic derivation of U_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/U_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ChineseRemainder"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_U_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_U_5",
            comment: "Axiomatic derivation of U_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/U_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ChineseRemainder"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AG_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AG_1",
            comment: "Axiomatic derivation of AG_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AG_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AG_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AG_2",
            comment: "Axiomatic derivation of AG_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AG_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AG_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AG_3",
            comment: "Axiomatic derivation of AG_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AG_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AG_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AG_4",
            comment: "Axiomatic derivation of AG_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AG_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_1",
            comment: "Axiomatic derivation of CA_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_2",
            comment: "Axiomatic derivation of CA_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_3",
            comment: "Axiomatic derivation of CA_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_4",
            comment: "Axiomatic derivation of CA_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_5",
            comment: "Axiomatic derivation of CA_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CA_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CA_6",
            comment: "Axiomatic derivation of CA_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CA_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_1",
            comment: "Axiomatic derivation of C_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_2",
            comment: "Axiomatic derivation of C_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_3",
            comment: "Axiomatic derivation of C_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_4",
            comment: "Axiomatic derivation of C_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_5",
            comment: "Axiomatic derivation of C_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_C_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_C_6",
            comment: "Axiomatic derivation of C_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/C_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CDI",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CDI",
            comment: "Axiomatic derivation of CDI. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CDI"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CL_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CL_1",
            comment: "Axiomatic derivation of CL_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CL_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CL_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CL_2",
            comment: "Axiomatic derivation of CL_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CL_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CL_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CL_3",
            comment: "Axiomatic derivation of CL_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CL_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CL_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CL_4",
            comment: "Axiomatic derivation of CL_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CL_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CL_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CL_5",
            comment: "Axiomatic derivation of CL_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CL_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CM_1",
            comment: "Axiomatic derivation of CM_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CM_2",
            comment: "Axiomatic derivation of CM_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CM_3",
            comment: "Axiomatic derivation of CM_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CR_1",
            comment: "Axiomatic derivation of CR_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CR_2",
            comment: "Axiomatic derivation of CR_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CR_3",
            comment: "Axiomatic derivation of CR_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CR_4",
            comment: "Axiomatic derivation of CR_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CR_5",
            comment: "Axiomatic derivation of CR_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_F_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_F_1",
            comment: "Axiomatic derivation of F_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/F_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_F_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_F_2",
            comment: "Axiomatic derivation of F_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/F_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_F_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_F_3",
            comment: "Axiomatic derivation of F_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/F_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_F_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_F_4",
            comment: "Axiomatic derivation of F_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/F_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FL_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FL_1",
            comment: "Axiomatic derivation of FL_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FL_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FL_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FL_2",
            comment: "Axiomatic derivation of FL_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FL_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FL_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FL_3",
            comment: "Axiomatic derivation of FL_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FL_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FL_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FL_4",
            comment: "Axiomatic derivation of FL_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FL_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_1",
            comment: "Axiomatic derivation of FPM_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_2",
            comment: "Axiomatic derivation of FPM_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_3",
            comment: "Axiomatic derivation of FPM_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_4",
            comment: "Axiomatic derivation of FPM_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_5",
            comment: "Axiomatic derivation of FPM_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_6",
            comment: "Axiomatic derivation of FPM_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_7",
            comment: "Axiomatic derivation of FPM_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_1",
            comment: "Axiomatic derivation of FS_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_2",
            comment: "Axiomatic derivation of FS_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_3",
            comment: "Axiomatic derivation of FS_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_4",
            comment: "Axiomatic derivation of FS_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_5",
            comment: "Axiomatic derivation of FS_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_6",
            comment: "Axiomatic derivation of FS_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FS_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FS_7",
            comment: "Axiomatic derivation of FS_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FS_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RE_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RE_1",
            comment: "Axiomatic derivation of RE_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RE_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IR_1",
            comment: "Axiomatic derivation of IR_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IR_2",
            comment: "Axiomatic derivation of IR_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IR_3",
            comment: "Axiomatic derivation of IR_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IR_4",
            comment: "Axiomatic derivation of IR_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SF_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SF_1",
            comment: "Axiomatic derivation of SF_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SF_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SF_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SF_2",
            comment: "Axiomatic derivation of SF_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SF_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RD_1",
            comment: "Axiomatic derivation of RD_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RD_2",
            comment: "Axiomatic derivation of RD_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SE_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SE_1",
            comment: "Axiomatic derivation of SE_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SE_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SE_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SE_2",
            comment: "Axiomatic derivation of SE_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SE_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SE_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SE_3",
            comment: "Axiomatic derivation of SE_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SE_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SE_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SE_4",
            comment: "Axiomatic derivation of SE_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SE_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OO_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OO_1",
            comment: "Axiomatic derivation of OO_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OO_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OO_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OO_2",
            comment: "Axiomatic derivation of OO_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OO_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OO_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OO_3",
            comment: "Axiomatic derivation of OO_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OO_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OO_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OO_4",
            comment: "Axiomatic derivation of OO_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OO_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OO_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OO_5",
            comment: "Axiomatic derivation of OO_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OO_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_1",
            comment: "Axiomatic derivation of CB_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_2",
            comment: "Axiomatic derivation of CB_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_3",
            comment: "Axiomatic derivation of CB_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_4",
            comment: "Axiomatic derivation of CB_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_5",
            comment: "Axiomatic derivation of CB_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CB_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CB_6",
            comment: "Axiomatic derivation of CB_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CB_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M1",
            comment: "Axiomatic derivation of OB_M1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M2",
            comment: "Axiomatic derivation of OB_M2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M3",
            comment: "Axiomatic derivation of OB_M3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M4",
            comment: "Axiomatic derivation of OB_M4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M5",
            comment: "Axiomatic derivation of OB_M5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_M6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_M6",
            comment: "Axiomatic derivation of OB_M6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_M6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_C1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_C1",
            comment: "Axiomatic derivation of OB_C1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_C1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_C2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_C2",
            comment: "Axiomatic derivation of OB_C2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_C2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_C3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_C3",
            comment: "Axiomatic derivation of OB_C3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_C3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_H1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_H1",
            comment: "Axiomatic derivation of OB_H1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_H1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_H2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_H2",
            comment: "Axiomatic derivation of OB_H2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_H2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_H3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_H3",
            comment: "Axiomatic derivation of OB_H3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_H3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_P1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_P1",
            comment: "Axiomatic derivation of OB_P1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_P1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_P2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_P2",
            comment: "Axiomatic derivation of OB_P2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_P2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OB_P3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OB_P3",
            comment: "Axiomatic derivation of OB_P3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OB_P3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CT_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CT_1",
            comment: "Axiomatic derivation of CT_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CT_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CT_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CT_2",
            comment: "Axiomatic derivation of CT_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CT_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CT_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CT_3",
            comment: "Axiomatic derivation of CT_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CT_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CT_4",
            comment: "Axiomatic derivation of CT_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CT_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CF_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CF_1",
            comment: "Axiomatic derivation of CF_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CF_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CF_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CF_2",
            comment: "Axiomatic derivation of CF_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CF_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CF_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CF_3",
            comment: "Axiomatic derivation of CF_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CF_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CF_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CF_4",
            comment: "Axiomatic derivation of CF_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CF_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HG_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HG_1",
            comment: "Axiomatic derivation of HG_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HG_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HG_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HG_2",
            comment: "Axiomatic derivation of HG_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HG_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HG_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HG_3",
            comment: "Axiomatic derivation of HG_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HG_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HG_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HG_4",
            comment: "Axiomatic derivation of HG_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HG_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HG_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HG_5",
            comment: "Axiomatic derivation of HG_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HG_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_C1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_C1",
            comment: "Axiomatic derivation of T_C1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_C1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_C2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_C2",
            comment: "Axiomatic derivation of T_C2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_C2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_C3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_C3",
            comment: "Axiomatic derivation of T_C3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_C3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_C4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_C4",
            comment: "Axiomatic derivation of T_C4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_C4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_I1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_I1",
            comment: "Axiomatic derivation of T_I1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_I1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_I2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_I2",
            comment: "Axiomatic derivation of T_I2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_I2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_I3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_I3",
            comment: "Axiomatic derivation of T_I3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_I3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_I4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_I4",
            comment: "Axiomatic derivation of T_I4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_I4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_I5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_I5",
            comment: "Axiomatic derivation of T_I5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_I5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_E1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_E1",
            comment: "Axiomatic derivation of T_E1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_E1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_E2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_E2",
            comment: "Axiomatic derivation of T_E2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_E2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_E3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_E3",
            comment: "Axiomatic derivation of T_E3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_E3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_E4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_E4",
            comment: "Axiomatic derivation of T_E4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_E4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_A1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_A1",
            comment: "Axiomatic derivation of T_A1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_A1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_A2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_A2",
            comment: "Axiomatic derivation of T_A2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_A2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_A3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_A3",
            comment: "Axiomatic derivation of T_A3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_A3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_T_A4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_T_A4",
            comment: "Axiomatic derivation of T_A4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/T_A4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AU_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AU_1",
            comment: "Axiomatic derivation of AU_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AU_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AU_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AU_2",
            comment: "Axiomatic derivation of AU_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AU_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AU_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AU_3",
            comment: "Axiomatic derivation of AU_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AU_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AU_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AU_4",
            comment: "Axiomatic derivation of AU_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AU_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AU_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AU_5",
            comment: "Axiomatic derivation of AU_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AU_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_1",
            comment: "Axiomatic derivation of EF_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_2",
            comment: "Axiomatic derivation of EF_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_3",
            comment: "Axiomatic derivation of EF_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_4",
            comment: "Axiomatic derivation of EF_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_5",
            comment: "Axiomatic derivation of EF_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_6",
            comment: "Axiomatic derivation of EF_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EF_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EF_7",
            comment: "Axiomatic derivation of EF_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EF_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_1",
            comment: "Axiomatic derivation of AA_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_2",
            comment: "Axiomatic derivation of AA_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_3",
            comment: "Axiomatic derivation of AA_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_4",
            comment: "Axiomatic derivation of AA_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_5",
            comment: "Axiomatic derivation of AA_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AA_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AA_6",
            comment: "Axiomatic derivation of AA_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AA_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AM_1",
            comment: "Axiomatic derivation of AM_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AM_2",
            comment: "Axiomatic derivation of AM_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AM_3",
            comment: "Axiomatic derivation of AM_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AM_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AM_4",
            comment: "Axiomatic derivation of AM_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AM_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_1",
            comment: "Axiomatic derivation of TH_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_2",
            comment: "Axiomatic derivation of TH_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_3",
            comment: "Axiomatic derivation of TH_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_4",
            comment: "Axiomatic derivation of TH_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_5",
            comment: "Axiomatic derivation of TH_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_6",
            comment: "Axiomatic derivation of TH_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_7",
            comment: "Axiomatic derivation of TH_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_8",
            comment: "Axiomatic derivation of TH_8. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_9",
            comment: "Axiomatic derivation of TH_9. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TH_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TH_10",
            comment: "Axiomatic derivation of TH_10. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TH_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_1",
            comment: "Axiomatic derivation of AR_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_2",
            comment: "Axiomatic derivation of AR_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_3",
            comment: "Axiomatic derivation of AR_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_4",
            comment: "Axiomatic derivation of AR_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 86: prf_AR_5 reclassified from EmpiricalVerification to InductiveProof
        Individual {
            id: "https://uor.foundation/proof/prf_AR_5",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_AR_5",
            comment: "Inductive proof of AR_5: greedy vs adiabatic cost difference \
                      is at most 5%. Base case at Q0 by exhaustive evaluation; \
                      inductive step by QLS_5 (identity preservation under lift).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_AR_5_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_AR_5_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PD_1",
            comment: "Axiomatic derivation of PD_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PD_2",
            comment: "Axiomatic derivation of PD_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PD_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PD_3",
            comment: "Axiomatic derivation of PD_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PD_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PD_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PD_4",
            comment: "Axiomatic derivation of PD_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PD_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PD_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PD_5",
            comment: "Axiomatic derivation of PD_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PD_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_1",
            comment: "Axiomatic derivation of RC_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_2",
            comment: "Axiomatic derivation of RC_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_3",
            comment: "Axiomatic derivation of RC_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_4",
            comment: "Axiomatic derivation of RC_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_5",
            comment: "Axiomatic derivation of RC_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_1",
            comment: "Axiomatic derivation of DC_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_2",
            comment: "Axiomatic derivation of DC_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_3",
            comment: "Axiomatic derivation of DC_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_4",
            comment: "Axiomatic derivation of DC_4. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_5",
            comment: "Axiomatic derivation of DC_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_6",
            comment: "Axiomatic derivation of DC_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_7",
            comment: "Axiomatic derivation of DC_7. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_8",
            comment: "Axiomatic derivation of DC_8. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_9",
            comment: "Axiomatic derivation of DC_9. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_10",
            comment: "Axiomatic derivation of DC_10. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DC_11",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DC_11",
            comment: "Axiomatic derivation of DC_11. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_11"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HA_1",
            comment: "Axiomatic derivation of HA_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HA_2",
            comment: "Axiomatic derivation of HA_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HA_3",
            comment: "Axiomatic derivation of HA_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_2",
            comment: "Axiomatic derivation of IT_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_3",
            comment: "Axiomatic derivation of IT_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_6",
            comment: "Axiomatic derivation of IT_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_7a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_7a",
            comment: "Axiomatic derivation of IT_7a. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_7a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_7b",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_7b",
            comment: "Axiomatic derivation of IT_7b. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_7b"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_7c",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_7c",
            comment: "Axiomatic derivation of IT_7c. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_7c"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IT_7d",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IT_7d",
            comment: "Axiomatic derivation of IT_7d. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_7d"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_1",
            comment: "Axiomatic derivation of psi_1. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_2",
            comment: "Axiomatic derivation of psi_2. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_3",
            comment: "Axiomatic derivation of psi_3. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_5",
            comment: "Axiomatic derivation of psi_5. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_6",
            comment: "Axiomatic derivation of psi_6. Holds at all quantum levels \
                      by definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // Axiomatic derivations for homology identities
        Individual {
            id: "https://uor.foundation/proof/prf_boundarySquaredZero",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_boundarySquaredZero",
            comment: "Axiomatic derivation of homology:boundarySquaredZero. Holds at all quantum \
                      levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/homology/boundarySquaredZero"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_4",
            comment: "Axiomatic derivation of homology:psi_4. Holds at all quantum \
                      levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/homology/psi_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_indexBridge",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_indexBridge",
            comment: "Axiomatic derivation of homology:indexBridge. Holds at all quantum \
                      levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/homology/indexBridge"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Axiomatic derivations for cohomology identities
        Individual {
            id: "https://uor.foundation/proof/prf_coboundarySquaredZero",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_coboundarySquaredZero",
            comment: "Axiomatic derivation of cohomology:coboundarySquaredZero. Holds at all \
                      quantum levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef(
                        "https://uor.foundation/cohomology/coboundarySquaredZero",
                    ),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_deRhamDuality",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_deRhamDuality",
            comment: "Axiomatic derivation of cohomology:deRhamDuality. Holds at all \
                      quantum levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/cohomology/deRhamDuality"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_sheafCohomologyBridge",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_sheafCohomologyBridge",
            comment: "Axiomatic derivation of cohomology:sheafCohomologyBridge. Holds at all \
                      quantum levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef(
                        "https://uor.foundation/cohomology/sheafCohomologyBridge",
                    ),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_localGlobalPrinciple",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_localGlobalPrinciple",
            comment: "Axiomatic derivation of cohomology:localGlobalPrinciple. Holds at all \
                      quantum levels by topological reasoning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef(
                        "https://uor.foundation/cohomology/localGlobalPrinciple",
                    ),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Gap C: Surface Symmetry proof
        Individual {
            id: "https://uor.foundation/proof/prf_surfaceSymmetry",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_surfaceSymmetry",
            comment: "Axiomatic derivation of the Surface Symmetry Theorem. Holds at all \
                      quantum levels: the composite P∘Π∘G is a well-typed morphism whenever \
                      G and P share the same state:Frame. Follows from the definition of \
                      the shared-frame condition and the type-equivalence algebra.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/surfaceSymmetry"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 25: Completeness Certification proofs (CC_1–CC_5)
        Individual {
            id: "https://uor.foundation/proof/prf_CC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_1",
            comment: "Proof that a CompleteType T satisfies: resolution(x, T) terminates \
                      in O(1) for all x ∈ R_n. Follows from IT_7d: when χ(N(C)) = n and \
                      all β_k = 0, the resolver has no topological obstructions.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_2",
            comment: "Proof that the ψ pipeline is monotone: each constraint application \
                      cannot increase the site deficit. Derived from the definition \
                      of the partition refinement order.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_3",
            comment: "Proof that a CompletenessCertificate implies CompleteType: the \
                      certificate attestation is only issued when IT_7d holds, by \
                      construction of the CompletenessResolver.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_4",
            comment: "Proof that the CompletenessAuditTrail witnessCount equals the number \
                      of CompletenessWitness records in the trail. Structural invariant \
                      of the audit accumulation protocol.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_5",
            comment: "Proof that the CechNerve nerve computation is deterministic: \
                      the same constraint set always produces the same nerve topology. \
                      Follows from the nerve functor being a functor (functoriality).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // Amendment 26: Quantum Level Scaling proofs (QL_1–QL_7)
        Individual {
            id: "https://uor.foundation/proof/prf_QL_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_1",
            comment: "Universal proof that neg(bnot(x)) = succ(x) in Z/(2^n)Z for all \
                      n ≥ 1. Derived symbolically from ring axioms: bnot is bitwise \
                      complement, neg is two's complement negation, succ is modular \
                      increment. The critical identity in universal form.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_2",
            comment: "Universal proof that the ring carrier set size is exactly 2^n for \
                      all n ≥ 1. Follows from the definition of Z/(2^n)Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_3",
            comment: "Universal proof that Landauer erasure cost scales as n × k_B T ln 2 \
                      at quantum level n. Follows from the thermodynamic interpretation: \
                      each bit erased from an n-bit ring costs k_B T ln 2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_4",
            comment: "Universal proof that the dihedral group D_{2^n} action on Z/(2^n)Z \
                      is faithful for all n ≥ 1. The stabilizer of any element is trivial \
                      under the full dihedral action.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_5",
            comment: "Universal proof that canonical form rewriting terminates at all \
                      quantum levels. The rewriting system is terminating by lexicographic \
                      descent on the term complexity measure.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_6",
            comment: "Universal proof that the completeness criterion χ(N(C)) = n \
                      generalizes to all quantum levels. Derived from the definition \
                      of the nerve construction and the Euler characteristic formula \
                      for simplicial complexes.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_7",
            comment: "Universal proof of the ring topology Euler characteristic identity: \
                      χ = 1 − n at quantum level n. Derived from the CW decomposition \
                      of the n-dimensional torus formed by the ring's cyclic group action.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // Amendment 27: Session-Scoped Resolution proofs (SR_1–SR_5)
        Individual {
            id: "https://uor.foundation/proof/prf_GR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_1",
            comment: "Proof of binding monotonicity: freeRank(B_{i+1}) ≤ freeRank(B_i) \
                      for all i in a Session. Follows from the definition of the \
                      BindingAccumulator: each appended binding either pins sites or \
                      is a no-op; it never frees them.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_2",
            comment: "Proof that the empty session is the identity element of the session \
                      algebra: freeRank(B_0) = total site space. The empty accumulator \
                      has no pinned sites by definition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_3",
            comment: "Proof of session convergence: a session terminates iff freeRank \
                      reaches its minimum (the maximum pinned by the given constraint set). \
                      Follows from the compactness of the site space and monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_4",
            comment: "Proof that disjoint bindings compose without site conflict: if two \
                      bindings address disjoint site sets, their composition is \
                      well-defined and their union is also a valid binding.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_5",
            comment: "Proof of contradiction detection correctness: ContradictionBoundary \
                      fires iff there exist bindings b, b' in the same Context with the \
                      same address, different datum, and same constraint. This is the \
                      minimal condition for type contradiction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // Amendment 28: ψ-Pipeline Inversion proof coverage
        Individual {
            id: "https://uor.foundation/proof/prf_TS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_1",
            comment: "Proof of nerve realisability: for any target profile with χ* ≤ n and \
                      β₀* = 1, there exists a ConstrainedType whose constraint nerve realises \
                      the target. Follows from the constructive synthesis algorithm.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_2",
            comment: "Proof of minimal basis bound: the MinimalConstraintBasis for the IT_7d \
                      target has size exactly n. Follows from the site-by-site construction \
                      and the minimality criterion.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_3",
            comment: "Proof of synthesis monotonicity: adding a constraint never decreases \
                      the Euler characteristic of the constraint nerve. Follows from the \
                      nerve inclusion principle.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_4",
            comment: "Proof of synthesis convergence: the TypeSynthesisResolver terminates in \
                      at most n steps. Follows from monotonicity (TS_3) and the finite site \
                      budget bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_5",
            comment: "Proof of synthesis-certification duality: a SynthesizedType achieves \
                      IT_7d iff the CompletenessResolver certifies it as CompleteType. The \
                      duality follows from the shared topological criterion.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_6",
            comment: "Proof of Jacobian-guided synthesis efficiency: the Jacobian oracle reduces \
                      expected steps from O(n²) to O(n log n). Follows from the information \
                      content of the Jacobian at each synthesis step.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_7",
            comment: "Proof of unreachable signatures: β₀ = 0 is unreachable by any non-empty \
                      ConstrainedType. Follows from the nerve connectedness of non-empty \
                      constraint sets.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 29/41: Quantum Level Spectral Sequence proof coverage
        // (reclassified from AxiomaticDerivation to InductiveProof in Amendment 41)
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_1",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_1",
            comment: "Proof of lift unobstructedness criterion: WittLift T' is CompleteType \
                      iff the spectral sequence collapses at E_2. Follows from the Leray \
                      spectral sequence of the quantum level extension.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_1_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_2",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_2",
            comment: "Proof of obstruction localisation: a non-trivial LiftObstruction is \
                      localised to a specific site at bit position n+1. Follows from the \
                      local-to-global structure of the constraint nerve.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_2_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_2_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_3",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_3",
            comment: "Proof of monotone lifting: basisSize(T') = basisSize(T) + 1 for trivially \
                      obstructed lifts. Follows from the minimal basis construction at Q_{n+1}.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_3_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_3_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_4",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_4",
            comment: "Proof of spectral sequence convergence bound: the spectral sequence \
                      converges by page E_{d+2} for depth-d configurations. Follows from \
                      the filtration length of the constraint nerve chain complex.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_4_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_4_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_5",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_5",
            comment: "Proof of universal identity preservation under quantum lifts: every \
                      universallyValid identity holds in the lifted ring. Follows from the \
                      universal validity definition and ring extension properties.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_5_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_6",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WLS_6",
            comment: "Proof of ψ-pipeline universality for quantum lifts: the ψ-pipeline \
                      produces a valid ChainComplex for any WittLift. Follows from the \
                      functorial construction of the chain complex.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_6_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_6_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        // Amendment 85: Non-self-referential base-case and step proofs for InductiveProofs
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_1_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_1_base",
            comment: "Base case for QLS_1 at Q0: lift unobstructedness holds trivially \
                      for 8-bit rings where the constraint nerve is contractible.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_2_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_2_base",
            comment: "Base case for QLS_2 at Q0: obstruction localisation holds at the \
                      8-bit level where sites are directly inspectable.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_2_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_2_step",
            comment: "Inductive step for QLS_2: if obstruction is localised at Q_k, \
                      the local-to-global structure of the constraint nerve preserves \
                      localisation at Q_{k+1}.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_3_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_3_base",
            comment: "Base case for QLS_3 at Q0: monotone lifting basis size increment \
                      holds trivially for 8-bit to 16-bit extension.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_3_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_3_step",
            comment: "Inductive step for QLS_3: the minimal basis construction at Q_{k+1} \
                      adds exactly one element from the trivially obstructed site.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_4_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_4_base",
            comment: "Base case for QLS_4 at Q0: spectral sequence convergence at \
                      E_{d+2} holds for 8-bit filtrations by direct computation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_4_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_4_step",
            comment: "Inductive step for QLS_4: filtration length at Q_{k+1} extends \
                      by at most one page from Q_k, preserving the E_{d+2} bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_5_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_5_base",
            comment: "Base case for QLS_5 at Q0: universallyValid identities hold in \
                      the 8-bit ring by definition of universal validity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_6_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_6_base",
            comment: "Base case for QLS_6 at Q0: the psi-pipeline produces a valid \
                      ChainComplex for 8-bit WittLifts by direct construction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WLS_6_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WLS_6_step",
            comment: "Inductive step for QLS_6: the functorial construction of the \
                      chain complex commutes with quantum level extension.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_3_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_3_base",
            comment: "Base case for QT_3: resolved basis size formula holds for \
                      chain length 1 by direct construction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_5_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_5_base",
            comment: "Base case for QT_5: LiftChainCertificate existence for \
                      tower height 1 follows from single-step certificate issuance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // Amendment 86: Base/step proofs for reclassified EmpiricalVerification proofs
        Individual {
            id: "https://uor.foundation/proof/prf_AR_5_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_5_base",
            comment: "Base case for AR_5 at Q0: greedy vs adiabatic cost difference \
                      verified by exhaustive enumeration over Z/256Z.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AR_5_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AR_5_step",
            comment: "Inductive step for AR_5: if greedy vs adiabatic bound holds at \
                      Q_k, it holds at Q_{k+1} by QLS_5 (universal identity \
                      preservation under quantum lift).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QM_6_base",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_6_base",
            comment: "Base case for QM_6 at Q0: amplitude index set equals monotone \
                      pinning trajectories by exhaustive trajectory enumeration over \
                      the 8-bit site lattice.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QM_6_step",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_6_step",
            comment: "Inductive step for QM_6: monotone pinning trajectories at \
                      Q_{k+1} extend those at Q_k by the site lattice ordering \
                      (monotone extension property).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 30: Monodromy Observables proof coverage
        Individual {
            id: "https://uor.foundation/proof/prf_MN_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_1",
            comment: "Proof of holonomy group containment: HolonomyGroup(T) ≤ D_{2^n}. Follows \
                      from the fact that all constraint applications are dihedral group elements.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_2",
            comment: "Proof of additive flatness: additive constraints (ResidueConstraint, \
                      DepthConstraint) generate only the identity in D_{2^n}. Follows from \
                      the additive structure of the dihedral action.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_3",
            comment: "Proof of dihedral generation: neg and bnot together generate D_{2^n}. \
                      Follows from the standard presentation of the dihedral group by \
                      involutions.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_4",
            comment: "Proof of holonomy-Betti implication: non-trivial holonomy implies β₁ ≥ 1. \
                      Follows from the fact that a non-trivial monodromy requires a topological \
                      loop in the constraint nerve.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_5",
            comment: "Proof of CompleteType holonomy: IT_7d (β₁ = 0) implies trivial holonomy \
                      (FlatType). Follows from MN_4 contrapositive: trivial holonomy ← β₁ = 0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_6",
            comment: "Proof of monodromy composition: the monodromy map is a group homomorphism \
                      from the loop space to D_{2^n}. Follows from the composition of dihedral \
                      group elements.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_7",
            comment: "Proof of TwistedType obstruction class: a TwistedType always contributes \
                      a non-zero class to H²(N(C(T')); ℤ/2ℤ) for any WittLift T'. Follows \
                      from MN_4 and the obstruction theory of dihedral torsors.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // Amendment 31: PT_ and ST_ proof individuals
        Individual {
            id: "https://uor.foundation/proof/prf_PT_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_1",
            comment: "Proof of PT_1: product type site additivity. siteBudget(A × B) = \
                      siteBudget(A) + siteBudget(B). Follows from the definition of \
                      ProductType as an independent concatenation of site spaces.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PT_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_2",
            comment: "Proof of PT_2: product type partition factorisation. \
                      partition(A × B) = partition(A) ⊗ partition(B). Follows from the \
                      tensor product structure of constraint nerves over independent \
                      site spaces.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PT_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_3",
            comment: "Proof of PT_3: product type Euler characteristic additivity. \
                      χ(N(C(A × B))) = χ(N(C(A))) + χ(N(C(B))). Follows from the \
                      Künneth formula applied to the join of constraint nerves.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PT_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_4",
            comment: "Proof of PT_4: product type entropy additivity. \
                      S(A × B) = S(A) + S(B). Follows from PT_1 (site additivity) \
                      and TH_1 (S = freeRank × ln 2).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_1",
            comment: "Proof of ST_1: sum type site budget maximum. \
                      siteBudget(A + B) = max(siteBudget(A), siteBudget(B)). \
                      Follows from SumType requiring capacity for the larger variant.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_2",
            comment: "Proof of ST_2: sum type entropy. \
                      S(A + B) = ln 2 + max(S(A), S(B)). The ln 2 term accounts \
                      for the variant discriminant bit; the max reflects that only \
                      one variant is active at a time.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 33: Grounded Context Limit proof individuals
        Individual {
            id: "https://uor.foundation/proof/prf_GS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_1",
            comment: "Proof of SC_1: context temperature. T_ctx(C) = \
                      freeRank(C) × ln 2 / n. Derived from TH_1 normalized per site.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_2",
            comment: "Proof of SC_2: saturation degree. σ(C) = (n − freeRank(C)) / n. \
                      Definitional identity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_3",
            comment: "Proof of SC_3: saturation monotonicity. Corollary of SR_1 \
                      through order-reversing SC_2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_4",
            comment: "Proof of SC_4: ground state equivalence. Four equivalent \
                      conditions for full saturation derived from SC_2, TH_1, SC_1.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_5",
            comment: "Proof of SC_5: O(1) resolution guarantee at saturation. \
                      Derived from SR_2 and FreeRank.isClosed at freeRank = 0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_6",
            comment: "Proof of SC_6: pre-reduction of effective budget. Derived from \
                      session-scoped site reduction at partial saturation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GS_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GS_7",
            comment: "Proof of SC_7: thermodynamic cooling cost. n site-closures \
                      at Landauer cost each via SR_1 + TH_4.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 34: Morphospace Achievability proof individuals
        Individual {
            id: "https://uor.foundation/proof/prf_MS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MS_1",
            comment: "Proof of MS_1: connectivity lower bound β₀ ≥ 1. \
                      Formalisation of TS_7.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MS_2",
            comment: "Proof of MS_2: Euler capacity ceiling χ ≤ n. Derived from \
                      TS_1 constraint nerve dimension bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MS_3",
            comment: "Proof of MS_3: Betti monotonicity under constraint addition. \
                      Formalisation of TS_3.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MS_4",
            comment: "Proof of MS_4: level-relative achievability. Derived from \
                      WittLift construction (Amendment 29).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MS_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MS_5",
            comment: "Proof of MS_5: empirical completeness convergence. \
                      Convergence statement for verification accumulation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 35: Computational Geodesic proof individuals
        Individual {
            id: "https://uor.foundation/proof/prf_GD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_1",
            comment: "Proof of GD_1: geodesic condition. Dual condition from \
                      AR_1 ordering + DC_10 selection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_2",
            comment: "Proof of GD_2: geodesic entropy bound. Each step on a \
                      geodesic erases exactly ln 2 nats.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GD_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_3",
            comment: "Proof of GD_3: total geodesic cost equals Landauer bound \
                      TH_4 with equality.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GD_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_4",
            comment: "Proof of GD_4: geodesic uniqueness up to step-order \
                      equivalence. Equal-J_k permutations are interchangeable.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GD_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_5",
            comment: "Proof of GD_5: subgeodesic detectability via step-by-step \
                      J_k check.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 36: Measurement Boundary proof individuals
        Individual {
            id: "https://uor.foundation/proof/prf_QM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_1",
            comment: "Proof of QM_1: von Neumann–Landauer bridge. S_vN equals \
                      Landauer erasure cost at the Landauer temperature β* = ln 2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_2",
            comment: "Proof of QM_2: measurement as site topology change. \
                      Projective collapse ≅ classical ResidueConstraint pinning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_3",
            comment: "Proof of QM_3: superposition entropy bound. 0 ≤ S_vN ≤ ln 2 \
                      for single-site superpositions.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QM_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_4",
            comment: "Proof of QM_4: collapse idempotence. Re-measurement of a \
                      collapsed state is a no-op.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // Amendment 37: Gap closure proof individuals (14)
        Individual {
            id: "https://uor.foundation/proof/prf_QM_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QM_5",
            comment: "Proof of QM_5: amplitude normalization (Born rule). \
                      Σ|αᵢ|² = 1 for well-formed SuperposedSiteState.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RC_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RC_6",
            comment: "Proof of RC_6: amplitude renormalization. Division by norm \
                      yields a normalized SuperposedSiteState.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RC_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_8",
            comment: "Proof of FPM_8: partition exhaustiveness. The four component \
                      cardinalities sum to 2ⁿ.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/DecideQ0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FPM_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FPM_9",
            comment: "Proof of FPM_9: exterior membership criterion. x ∈ Ext(T) \
                      iff x ∉ carrier(T).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FPM_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MN_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MN_8",
            comment: "Proof of MN_8: holonomy classification covering. Every \
                      ConstrainedType is flat xor twisted.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MN_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_QL_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_QL_8",
            comment: "Proof of QL_8: quantum level chain inverse. wittLevelPredecessor \
                      is the left inverse of nextLevel.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QL_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_7",
            comment: "Proof of D_7: dihedral composition rule from the semidirect \
                      product presentation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SP_1",
            comment: "Proof of SP_1: classical embedding. Superposition resolution \
                      of a classical datum reduces to classical resolution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SP_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SP_2",
            comment: "Proof of SP_2: collapse–resolve commutativity. The collapse \
                      and resolve operations commute.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SP_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SP_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SP_3",
            comment: "Proof of SP_3: amplitude preservation. The SuperpositionResolver \
                      preserves the normalized amplitude vector.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SP_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SP_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SP_4",
            comment: "Proof of SP_4: Born rule outcome probability. P(collapse to \
                      site k) = |α_k|².",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SP_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PT_2a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_2a",
            comment: "Proof of PT_2a: product type partition tensor. Π(A × B) = \
                      PartitionProduct(Π(A), Π(B)).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_2a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PT_2b",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PT_2b",
            comment: "Proof of PT_2b: sum type partition coproduct. Π(A + B) = \
                      PartitionCoproduct(Π(A), Π(B)).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PT_2b"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GD_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GD_6",
            comment: "Proof of GD_6: geodesic predicate decomposition. isGeodesic = \
                      isAR1Ordered ∧ isDC10Selected.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GD_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 34: Formal morphospace boundary individuals
        Individual {
            id: "https://uor.foundation/proof/iw_beta0_bound",
            type_: "https://uor.foundation/proof/ImpossibilityWitness",
            label: "iw_beta0_bound",
            comment: "Impossibility witness for MS_1: β₀ = 0 is forbidden for \
                      any non-empty ConstrainedType because the constraint nerve \
                      is always connected.",
            properties: &[
                (
                    "https://uor.foundation/proof/forbidsSignature",
                    IndividualValue::Str("β₀ = 0"),
                ),
                (
                    "https://uor.foundation/proof/impossibilityReason",
                    IndividualValue::Str(
                        "β₀ = 0 violates MS_1: constraint nerve of non-empty set is connected",
                    ),
                ),
                (
                    "https://uor.foundation/proof/impossibilityDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Contradiction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/iw_chi_ceiling",
            type_: "https://uor.foundation/proof/ImpossibilityWitness",
            label: "iw_chi_ceiling",
            comment: "Impossibility witness for MS_2: χ > n is forbidden at \
                      quantum level n. The Euler characteristic cannot exceed \
                      the quantum level.",
            properties: &[
                (
                    "https://uor.foundation/proof/forbidsSignature",
                    IndividualValue::Str("χ > n"),
                ),
                (
                    "https://uor.foundation/proof/impossibilityReason",
                    IndividualValue::Str(
                        "χ > n violates MS_2: Euler characteristic bounded by quantum level",
                    ),
                ),
                (
                    "https://uor.foundation/proof/impossibilityDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Contradiction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/mr_completeness_target",
            type_: "https://uor.foundation/proof/MorphospaceRecord",
            label: "mr_completeness_target",
            comment: "Morphospace record: the IT_7d completeness target χ = n is \
                      achievable (sits at the ceiling of MS_2).",
            properties: &[
                (
                    "https://uor.foundation/proof/boundaryType",
                    IndividualValue::IriRef("https://uor.foundation/observable/Achievable"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/mr_connectivity_lower",
            type_: "https://uor.foundation/proof/MorphospaceRecord",
            label: "mr_connectivity_lower",
            comment: "Morphospace record: the connectivity lower bound β₀ ≥ 1 \
                      marks the forbidden region from below.",
            properties: &[
                (
                    "https://uor.foundation/proof/boundaryType",
                    IndividualValue::IriRef("https://uor.foundation/observable/Forbidden"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // Amendment 41: QT_ tower identity proofs
        Individual {
            id: "https://uor.foundation/proof/prf_WT_1",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WT_1",
            comment: "Proof of tower chain validity by induction on chain length.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_1"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_6"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_2",
            comment: "Proof of obstruction count bound: direct from QLS_2 localization.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_3",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WT_3",
            comment: "Proof of resolved basis size formula by induction on chain \
                      length.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WT_3_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WLS_3"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_4",
            comment: "Proof of flat tower characterization: isFlat iff trivial \
                      holonomy at every step.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_5",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_WT_5",
            comment: "Proof of LiftChainCertificate existence by induction on \
                      tower height.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WT_5_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_WT_1"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_6",
            comment: "Proof of single-step reduction: QT_3 with chainLength=1 \
                      reduces to QLS_3.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_7",
            comment: "Proof of flat chain basis size formula.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // Amendment 44: Structural Gap Closure proofs
        // G7: CarryConstraint site-pinning map
        Individual {
            id: "https://uor.foundation/proof/prf_CC_PINS",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CC_PINS",
            comment: "Proof of CC_PINS: carry-constraint site-pinning map \
                      follows from ring carry propagation rule.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_PINS"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CC_COST_SITE",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_CC_COST_SITE",
            comment: "Computation certificate for CC_COST_SITE: exhaustive \
                      enumeration at Q0 confirms |pinsSites| = popcount + 1.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CC_COST_SITE"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        // G1: Nerve joint satisfiability
        Individual {
            id: "https://uor.foundation/proof/prf_jsat_RR",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_jsat_RR",
            comment: "Proof of jsat_RR: CRT joint satisfiability follows \
                      from the Chinese Remainder Theorem.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/jsat_RR"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_jsat_CR",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_jsat_CR",
            comment: "Proof of jsat_CR: carry-residue joint satisfiability \
                      follows from the carry stopping rule and residue class \
                      intersection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/jsat_CR"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_jsat_CC",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_jsat_CC",
            comment: "Computation certificate for jsat_CC: bit-pattern \
                      exhaustive enumeration at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/jsat_CC"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        // G2: DihedralElement inverse and order
        Individual {
            id: "https://uor.foundation/proof/prf_D_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_8",
            comment: "Proof of D_8: dihedral inverse formula follows from \
                      D_5 group presentation and D_7 composition rule.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_D_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_D_9",
            comment: "Proof of D_9: reflection order 2 follows from D_7 \
                      composition: (r^k s)(r^k s) = identity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/D_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G5: Constraint language expressiveness
        Individual {
            id: "https://uor.foundation/proof/prf_EXP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EXP_1",
            comment: "Proof of EXP_1: monotone carrier characterization \
                      follows from site lattice monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EXP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EXP_2",
            type_: "https://uor.foundation/proof/ComputationCertificate",
            label: "prf_EXP_2",
            comment: "Computation certificate for EXP_2: principal filter \
                      count verified by exhaustive enumeration at Q0.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EXP_2"),
                ),
                (
                    "https://uor.foundation/proof/atWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Computation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EXP_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EXP_3",
            comment: "Proof of EXP_3: SumType carrier is coproduct by \
                      definitional architectural decision.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EXP_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G4: SumType topology
        Individual {
            id: "https://uor.foundation/proof/prf_ST_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_3",
            comment: "Proof of ST_3: Euler characteristic additivity for \
                      disjoint simplicial complexes.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_4",
            comment: "Proof of ST_4: Betti number additivity via \
                      Mayer-Vietoris for disjoint union.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_5",
            comment: "Proof of ST_5: SumType completeness transfer follows \
                      from ST_3 + ST_4 + IT_7d.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // Product/Coproduct Completion Amendment — proof individuals for
        // ST_6..ST_10 and CPT_1..CPT_6 establish the identity-proof bijection
        // required by conformance/validators/ontology/inventory.rs.
        Individual {
            id: "https://uor.foundation/proof/prf_ST_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_6",
            comment: "Proof of ST_6: unique existence of the PartitionCoproduct \
                      tag site. Follows from ST_2's single `ln 2` entropy quantum — \
                      distinguishing two variants requires exactly one bit, which \
                      fixes exactly one tag site logically disjoint from every \
                      data site of either operand.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_7",
            comment: "Proof of ST_7: PartitionCoproduct variant-tagging \
                      decomposition. constraints(A + B) = constraints(A) \
                      ∪ {tag=0} ∪ constraints(B) ∪ {tag=1}. Each tag-pinning \
                      constraint is emitted as an Affine with single-coefficient \
                      1 at the tag site, distinguished by bias (0 for left, -1 \
                      for right).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_8",
            comment: "Proof of ST_8: variant nerve disjointness. The two \
                      tag-pinning constraints carry incompatible biases (0 vs -1) \
                      at the same tag site, so no single assignment satisfies \
                      both simultaneously. The variant nerves therefore share no \
                      common assignment — the bridge premise ST_3/ST_4 rely on.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_9",
            comment: "Proof of ST_9: χ additivity for PartitionCoproduct \
                      constructions. Combines ST_8's disjointness guarantee \
                      with the universal ST_3 (χ additive for any disjoint \
                      SumType).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ST_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ST_10",
            comment: "Proof of ST_10: Betti additivity for PartitionCoproduct \
                      constructions. Combines ST_8's disjointness guarantee \
                      with the universal ST_4 (Betti additive at every \
                      dimension for any disjoint SumType).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ST_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_1",
            comment: "Proof of CPT_1: site additivity for Cartesian partition \
                      products. UOR sites are bit widths; |A ⊠ B| = \
                      2^{n_A} · 2^{n_B} = 2^{n_A + n_B}, so sites add. Sister \
                      identity to PT_1 (same arithmetic; the two differ at the \
                      nerve-topology level per CPT_3 vs PT_3).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_2a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_2a",
            comment: "Proof of CPT_2a: Π(A ⊠ B) = CartesianPartitionProduct(Π(A), \
                      Π(B)). Structural axiom defining the partition map on \
                      Cartesian-product types.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_2a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_3",
            comment: "Proof of CPT_3: χ multiplicativity for Cartesian partition \
                      products. Classical topology result — the simplicial \
                      product nerve has vertex set V(N(A)) × V(N(B)) and Euler \
                      characteristic χ(N(A)) · χ(N(B)). Distinguishes \
                      CartesianPartitionProduct from the site-disjoint-union \
                      PartitionProduct (which is χ-additive per PT_3).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_4",
            comment: "Proof of CPT_4: Künneth formula for Betti composition \
                      under Cartesian partition product. Follows from the \
                      classical Künneth theorem applied to the simplicial \
                      product nerve of the two component constraint systems.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_5",
            comment: "Proof of CPT_5: entropy additivity for Cartesian partition \
                      products. S(A ⊠ B) = S(A) + S(B) — Shannon additivity for \
                      independent subsystems. Consistent pointwise with IT_7a/b \
                      on the combined system.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CPT_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CPT_6",
            comment: "Proof of CPT_6: distributivity of Cartesian partition \
                      product over PartitionCoproduct. A ⊠ (B + C) ≡ \
                      (A ⊠ B) + (A ⊠ C) at the siteBudget, SITE_COUNT, χ, and \
                      entropy levels simultaneously. Does NOT extend to \
                      PartitionProduct (distribution over × duplicates the outer \
                      factor at the site-budget level).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CPT_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G3: TypeSynthesis reachability
        Individual {
            id: "https://uor.foundation/proof/prf_TS_8",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_TS_8",
            comment: "Inductive proof of TS_8: minimum constraint count for \
                      beta_1 = k is 2k + 1. Base case at k=1 requires 3 \
                      mutually overlapping constraints.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_HA_1"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_TS_4"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_9",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_TS_9",
            comment: "Inductive proof of TS_9: TypeSynthesisResolver \
                      terminates in at most 2^n steps. Base case at n=1 has \
                      2 constraint combinations.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_TS_1"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_TS_4"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TS_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TS_10",
            comment: "Proof of TS_10: ForbiddenSignature membership follows \
                      from exhaustive enumeration bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TS_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G6: ObstructionChain termination
        Individual {
            id: "https://uor.foundation/proof/prf_WT_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_8",
            comment: "Proof of QT_8: ObstructionChain length bound follows \
                      from QLS_2 and spectral sequence convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WT_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WT_9",
            comment: "Proof of QT_9: TowerCompletenessResolver termination \
                      follows from finite chain length and QT_8 bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WT_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // G11: Coefficient ring
        Individual {
            id: "https://uor.foundation/proof/prf_COEFF_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_COEFF_1",
            comment: "Proof of COEFF_1: Z/2Z coefficient ring is \
                      definitional, consistent with MN_7.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/COEFF_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G9: GluingObstruction feedback
        Individual {
            id: "https://uor.foundation/proof/prf_GO_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GO_1",
            comment: "Proof of GO_1: cohomology killing lemma for \
                      GluingObstruction feedback.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GO_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // G8: Session saturation lifecycle bridge
        Individual {
            id: "https://uor.foundation/proof/prf_GR_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_6",
            comment: "Proof of SR_6: saturation re-entry free count follows \
                      from SR_1 monotone accumulation and SC_2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_7",
            comment: "Proof of SR_7: saturation degree degradation follows \
                      from SC_2 definition and SR_1 monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // G10: Amplitude index set (Amendment 86: reclassified to InductiveProof)
        Individual {
            id: "https://uor.foundation/proof/prf_QM_6",
            type_: "https://uor.foundation/proof/InductiveProof",
            label: "prf_QM_6",
            comment: "Inductive proof of QM_6: amplitude index set equals monotone \
                      pinning trajectories. Base case at Q0 by exhaustive trajectory \
                      enumeration; inductive step by site lattice ordering.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/QM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/baseCase",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_QM_6_base"),
                ),
                (
                    "https://uor.foundation/proof/inductiveStep",
                    IndividualValue::IriRef("https://uor.foundation/proof/prf_QM_6_step"),
                ),
                (
                    "https://uor.foundation/proof/validForKAtLeast",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        // Amendment 46: Certificate Issuance Coverage proofs
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_1",
            comment: "Proof of CIC_1: TransformCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_2",
            comment: "Proof of CIC_2: IsometryCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/GroupPresentation"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_3",
            comment: "Proof of CIC_3: InvolutionCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_4",
            comment: "Proof of CIC_4: GroundingCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_5",
            comment: "Proof of CIC_5: GeodesicCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_6",
            comment: "Proof of CIC_6: MeasurementCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        // Amendment 86: prf_CIC_7 reclassified from EmpiricalVerification to AxiomaticDerivation
        Individual {
            id: "https://uor.foundation/proof/prf_CIC_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CIC_7",
            comment: "Axiomatic derivation of CIC_7: BornRuleVerification issuance \
                      coverage. Follows by composition from corrected OA_4 (product \
                      formula chain) and QM_5 (amplitude normalization).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CIC_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GC_1",
            comment: "Proof of GC_1: GroundingCertificate issuance coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 48: Multi-Session Coordination proofs
        Individual {
            id: "https://uor.foundation/proof/prf_GR_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_8",
            comment: "Proof of SR_8: session composition tower consistency.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_9",
            comment: "Proof of SR_9: ContextLease site disjointness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GR_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GR_10",
            comment: "Proof of SR_10: ExecutionPolicy confluence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GR_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_1",
            comment: "Proof of MC_1: lease partition conserves total budget.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_2",
            comment: "Proof of MC_2: per-lease binding monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_3",
            comment: "Proof of MC_3: composition freeRank inclusion-exclusion.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_4",
            comment: "Proof of MC_4: disjoint-lease composition additivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_5",
            comment: "Proof of MC_5: policy-invariant final binding set.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_6",
            comment: "Proof of MC_6: full lease coverage implies composed saturation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_7",
            comment: "Proof of MC_7: distributed O(1) resolution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MC_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MC_8",
            comment: "Proof of MC_8: parallelism bound on per-session resolution work.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MC_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // Amendment 53: Witt\u{2013}Carry Formalization proofs \u{2014} WC_ series (12)
        Individual {
            id: "https://uor.foundation/proof/prf_WC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_1",
            comment: "Axiomatic derivation of WC_1: Witt coordinate = bit \
                      coordinate at p=2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_2",
            comment: "Axiomatic derivation of WC_2: Witt sum correction = \
                      carry.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_3",
            comment: "Axiomatic derivation of WC_3: carry recurrence = Witt \
                      polynomial recurrence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_4",
            comment: "Axiomatic derivation of WC_4: \u{03b4}-correction = \
                      single-level carry.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_5",
            comment: "Axiomatic derivation of WC_5: LiftObstruction = \
                      \u{03b4} nonvanishing.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_6",
            comment: "Axiomatic derivation of WC_6: metric discrepancy = \
                      Witt defect.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_7",
            comment: "Axiomatic derivation of WC_7: D_1 = Witt truncation \
                      order.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_8",
            comment: "Axiomatic derivation of WC_8: D_3 = Witt-Burnside \
                      conjugation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_9",
            comment: "Axiomatic derivation of WC_9: D_4 = reflection \
                      composition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_10",
            comment: "Axiomatic derivation of WC_10: Frobenius = identity \
                      on W_n(F_2).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_11",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_11",
            comment: "Axiomatic derivation of WC_11: Verschiebung = \
                      doubling.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_11"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_WC_12",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_WC_12",
            comment: "Axiomatic derivation of WC_12: \u{03b4} = squaring \
                      defect / 2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/WC_12"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/BitwiseInduction"),
                ),
            ],
        },
        // Amendment 53: Ostrowski\u{2013}Archimedean bridge proofs \u{2014} OA_ series (5)
        Individual {
            id: "https://uor.foundation/proof/prf_OA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OA_1",
            comment: "Axiomatic derivation of OA_1: Ostrowski product \
                      formula at p=2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OA_2",
            comment: "Axiomatic derivation of OA_2: crossing cost = ln 2.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OA_3",
            comment: "Axiomatic derivation of OA_3: Landauer cost grounding \
                      via product formula.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        // Amendment 86: prf_OA_4 reclassified from EmpiricalVerification to AxiomaticDerivation
        Individual {
            id: "https://uor.foundation/proof/prf_OA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OA_4",
            comment: "Axiomatic derivation of OA_4: Born rule bridge follows from \
                      the product formula chain OA_1 (Ostrowski) -> OA_2 (crossing \
                      cost) -> OA_3 (Landauer grounding) -> OA_4.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OA_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OA_5",
            comment: "Axiomatic derivation of OA_5: entropy per \u{03b4}-level \
                      = crossing cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OA_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/ProductFormula"),
                ),
            ],
        },
        // Amendment 54: Homotopy Nerve proofs \u{2014} HT_ series (8)
        Individual {
            id: "https://uor.foundation/proof/prf_HT_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_1",
            comment: "Axiomatic derivation of HT_1: the constraint nerve \
                      satisfies the Kan extension condition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_2",
            comment: "Axiomatic derivation of HT_2: path components recover \
                      Betti number \u{03b2}\u{2080}.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_3",
            comment: "Axiomatic derivation of HT_3: \u{03c0}\u{2081} factors \
                      through HolonomyGroup.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_4",
            comment: "Axiomatic derivation of HT_4: higher homotopy groups \
                      vanish above dimension.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_5",
            comment: "Axiomatic derivation of HT_5: 1-truncation determines \
                      flat/twisted classification.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_6",
            comment: "Axiomatic derivation of HT_6: trivial k-invariants \
                      imply spectral collapse.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_7",
            comment: "Axiomatic derivation of HT_7: Whitehead product detects \
                      lift obstructions.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HT_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HT_8",
            comment: "Axiomatic derivation of HT_8: Hurewicz isomorphism for \
                      first non-vanishing group.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HT_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // Amendment 55: Homotopy Pipeline proofs
        Individual {
            id: "https://uor.foundation/proof/prf_psi_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_7",
            comment: "Axiomatic derivation of \u{03c8}_7: KanComplex to \
                      PostnikovTower functor.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_8",
            comment: "Axiomatic derivation of \u{03c8}_8: PostnikovTower to \
                      HomotopyGroups extraction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_psi_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_psi_9",
            comment: "Axiomatic derivation of \u{03c8}_9: HomotopyGroups to \
                      KInvariants computation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/psi_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HP_1",
            comment: "Axiomatic derivation of HP_1: pipeline composition \
                      \u{03c8}_7 \u{2218} \u{03c8}_1.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HP_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HP_2",
            comment: "Axiomatic derivation of HP_2: homotopy extraction agrees \
                      with homology on truncation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HP_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HP_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HP_3",
            comment: "Axiomatic derivation of HP_3: \u{03c8}_9 detects QLS_4 \
                      convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HP_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HP_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HP_4",
            comment: "Axiomatic derivation of HP_4: HomotopyResolver complexity \
                      bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HP_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 56: Moduli Space proofs
        Individual {
            id: "https://uor.foundation/proof/prf_MD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_1",
            comment: "Axiomatic derivation of MD_1: moduli dimension equals \
                      basis cardinality.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_2",
            comment: "Axiomatic derivation of MD_2: automorphism group is \
                      subgroup of D_{2^n}.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_3",
            comment: "Axiomatic derivation of MD_3: first-order deformations \
                      parameterize tangent space.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_4",
            comment: "Axiomatic derivation of MD_4: obstruction space equals \
                      LiftObstruction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_5",
            comment: "Axiomatic derivation of MD_5: FlatType stratum is open \
                      (codimension 0).",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_6",
            comment: "Axiomatic derivation of MD_6: TwistedType strata have \
                      codimension at least 1.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_7",
            comment: "Axiomatic derivation of MD_7: versal deformation exists \
                      when unobstructed.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_8",
            comment: "Axiomatic derivation of MD_8: completeness-preserving iff \
                      obstruction vanishes along path.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_9",
            comment: "Axiomatic derivation of MD_9: tower map site dimension \
                      is 1 when unobstructed.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MD_10",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MD_10",
            comment: "Axiomatic derivation of MD_10: tower map site empty iff \
                      twisted at every level.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MD_10"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // Amendment 57: Moduli Resolver proofs
        Individual {
            id: "https://uor.foundation/proof/prf_MR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MR_1",
            comment: "Axiomatic derivation of MR_1: moduli resolver agrees \
                      with MorphospaceBoundary.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MR_2",
            comment: "Axiomatic derivation of MR_2: stratification record is \
                      complete.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MR_3",
            comment: "Axiomatic derivation of MR_3: moduli resolver complexity \
                      bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MR_4",
            comment: "Axiomatic derivation of MR_4: moduli-morphospace \
                      consistency.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 58: Carry Algebra proofs ──────────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_CY_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_1",
            comment: "Axiomatic derivation of CY_1: carry generation condition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_2",
            comment: "Axiomatic derivation of CY_2: carry propagation condition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_3",
            comment: "Axiomatic derivation of CY_3: carry kill condition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_4",
            comment: "Axiomatic derivation of CY_4: d_\u{0394} as \
                      carry\u{2013}Hamming discrepancy.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_5",
            comment: "Axiomatic derivation of CY_5: optimal encoding theorem.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_6",
            comment: "Axiomatic derivation of CY_6: site ordering theorem.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CY_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CY_7",
            comment: "Axiomatic derivation of CY_7: carry lookahead via \
                      prefix computation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CY_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 59: Named Base Metrics proofs ─────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_BM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_1",
            comment: "Axiomatic derivation of BM_1: saturation metric definition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_2",
            comment: "Axiomatic derivation of BM_2: Euler characteristic formula.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_3",
            comment: "Axiomatic derivation of BM_3: index theorem linking all six metrics.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BM_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_4",
            comment: "Axiomatic derivation of BM_4: Jacobian vanishes on pinned sites.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BM_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_5",
            comment: "Axiomatic derivation of BM_5: d_delta equals Witt defect.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BM_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BM_6",
            comment: "Axiomatic derivation of BM_6: metric composition tower.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // ── Amendment 60: Galois Connection proofs ────────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_GL_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GL_1",
            comment: "Axiomatic derivation of GL_1: \u{03c3} as lower adjoint.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GL_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GL_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GL_2",
            comment: "Axiomatic derivation of GL_2: r as complement of upper adjoint.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GL_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GL_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GL_3",
            comment: "Axiomatic derivation of GL_3: completeness as Galois fixpoint.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GL_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_GL_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_GL_4",
            comment: "Axiomatic derivation of GL_4: Galois order reversal.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GL_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // ── Amendment 60: Nerve Operations proofs ─────────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_NV_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_NV_1",
            comment: "Axiomatic derivation of NV_1: nerve additivity for disjoint domains.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/NV_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_NV_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_NV_2",
            comment: "Axiomatic derivation of NV_2: Mayer\u{2013}Vietoris for constraint nerves.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/NV_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_NV_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_NV_3",
            comment: "Axiomatic derivation of NV_3: Betti number bounded change.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/NV_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_NV_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_NV_4",
            comment: "Axiomatic derivation of NV_4: accumulation monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/NV_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // ── Amendment 61: Structural Type proofs ─────────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_SD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_1",
            comment: "Axiomatic derivation of SD_1: scalar grounding identity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_2",
            comment: "Axiomatic derivation of SD_2: symbol grounding identity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_3",
            comment: "Axiomatic derivation of SD_3: sequence free monoid identity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_4",
            comment: "Axiomatic derivation of SD_4: tuple site additivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_5",
            comment: "Axiomatic derivation of SD_5: graph nerve equality.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_6",
            comment: "Axiomatic derivation of SD_6: set permutation invariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_7",
            comment: "Axiomatic derivation of SD_7: tree acyclicity constraint.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SD_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SD_8",
            comment: "Axiomatic derivation of SD_8: table functorial decomposition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SD_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // Amendment 62: Composed Operations proofs (18)
        Individual {
            id: "https://uor.foundation/proof/prf_DD_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DD_1",
            comment: "Axiomatic derivation of DD_1: dispatch determinism.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DD_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DD_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DD_2",
            comment: "Axiomatic derivation of DD_2: dispatch coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DD_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PI_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PI_1",
            comment: "Axiomatic derivation of PI_1: inference idempotence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PI_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PI_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PI_2",
            comment: "Axiomatic derivation of PI_2: inference soundness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PI_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PI_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PI_3",
            comment: "Axiomatic derivation of PI_3: inference composition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PI_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PI_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PI_4",
            comment: "Axiomatic derivation of PI_4: inference complexity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PI_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PI_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PI_5",
            comment: "Axiomatic derivation of PI_5: inference coherence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PI_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PA_1",
            comment: "Axiomatic derivation of PA_1: accumulation permutation invariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PA_2",
            comment: "Axiomatic derivation of PA_2: accumulation monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PA_3",
            comment: "Axiomatic derivation of PA_3: accumulation soundness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PA_4",
            comment: "Axiomatic derivation of PA_4: accumulation base preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PA_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PA_5",
            comment: "Axiomatic derivation of PA_5: accumulation identity element.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PA_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PL_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PL_1",
            comment: "Axiomatic derivation of PL_1: lease disjointness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PL_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PL_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PL_2",
            comment: "Axiomatic derivation of PL_2: lease conservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PL_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PL_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PL_3",
            comment: "Axiomatic derivation of PL_3: lease coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PL_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PK_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PK_1",
            comment: "Axiomatic derivation of PK_1: composition validity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PK_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PK_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PK_2",
            comment: "Axiomatic derivation of PK_2: distributed resolution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PK_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PP_1",
            comment: "Axiomatic derivation of PP_1: pipeline unification.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // Amendment 63: Reduction Core proofs (16)
        Individual {
            id: "https://uor.foundation/proof/prf_PE_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_1",
            comment: "Axiomatic derivation of PE_1: state initialization.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_2",
            comment: "Axiomatic derivation of PE_2: resolver dispatch.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_3",
            comment: "Axiomatic derivation of PE_3: ring address grounding.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_4",
            comment: "Axiomatic derivation of PE_4: constraint resolution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_5",
            comment: "Axiomatic derivation of PE_5: consistent accumulation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_6",
            comment: "Axiomatic derivation of PE_6: coherent extraction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PE_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PE_7",
            comment: "Axiomatic derivation of PE_7: full pipeline composition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PE_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_1",
            comment: "Axiomatic derivation of PM_1: phase rotation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_2",
            comment: "Axiomatic derivation of PM_2: phase gate check.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_3",
            comment: "Axiomatic derivation of PM_3: conjugate rollback.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_4",
            comment: "Axiomatic derivation of PM_4: rollback involution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_5",
            comment: "Axiomatic derivation of PM_5: epoch saturation preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_6",
            comment: "Axiomatic derivation of PM_6: service window context.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PM_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PM_7",
            comment: "Axiomatic derivation of PM_7: machine determinism.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PM_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ER_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ER_1",
            comment: "Axiomatic derivation of ER_1: guard satisfaction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ER_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ER_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ER_2",
            comment: "Axiomatic derivation of ER_2: effect atomicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ER_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 64: Reduction Expansion proofs (16)
        Individual {
            id: "https://uor.foundation/proof/prf_ER_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ER_3",
            comment: "Axiomatic derivation of ER_3: guard purity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ER_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_ER_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_ER_4",
            comment: "Axiomatic derivation of ER_4: intra-stage commutativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/ER_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EA_1",
            comment: "Axiomatic derivation of EA_1: epoch boundary reset.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EA_2",
            comment: "Axiomatic derivation of EA_2: saturation monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EA_3",
            comment: "Axiomatic derivation of EA_3: service window bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EA_4",
            comment: "Axiomatic derivation of EA_4: epoch admission exclusivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_1",
            comment: "Axiomatic derivation of OE_1: stage fusion.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_2",
            comment: "Axiomatic derivation of OE_2: effect commutativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_3",
            comment: "Axiomatic derivation of OE_3: lease parallelism.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_4a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_4a",
            comment: "Axiomatic derivation of OE_4a: fusion semantics preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_4a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_4b",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_4b",
            comment: "Axiomatic derivation of OE_4b: commutation outcome preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_4b"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OE_4c",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OE_4c",
            comment: "Axiomatic derivation of OE_4c: parallelism coverage preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OE_4c"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_1",
            comment: "Axiomatic derivation of CS_1: bounded stage cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_2",
            comment: "Axiomatic derivation of CS_2: additive pipeline cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_3",
            comment: "Axiomatic derivation of CS_3: bounded rollback cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_4",
            comment: "Axiomatic derivation of CS_4: constant preflight cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 65: Reduction Completion proofs (16)
        Individual {
            id: "https://uor.foundation/proof/prf_CS_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_5",
            comment: "Axiomatic derivation of CS_5: total reduction cost bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FA_1",
            comment: "Axiomatic derivation of FA_1: query liveness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FA_2",
            comment: "Axiomatic derivation of FA_2: no starvation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FA_3",
            comment: "Axiomatic derivation of FA_3: fair lease allocation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SW_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SW_1",
            comment: "Axiomatic derivation of SW_1: memory boundedness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SW_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SW_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SW_2",
            comment: "Axiomatic derivation of SW_2: saturation invariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SW_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SW_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SW_3",
            comment: "Axiomatic derivation of SW_3: eviction releases resources.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SW_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SW_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SW_4",
            comment: "Axiomatic derivation of SW_4: non-empty window.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SW_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LS_1",
            comment: "Axiomatic derivation of LS_1: suspension preserves pinned state.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LS_2",
            comment: "Axiomatic derivation of LS_2: expiry releases resources.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LS_3",
            comment: "Axiomatic derivation of LS_3: checkpoint restore idempotence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LS_4",
            comment: "Axiomatic derivation of LS_4: suspend-resume round-trip.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TJ_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TJ_1",
            comment: "Axiomatic derivation of TJ_1: AllOrNothing rollback.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TJ_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TJ_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TJ_2",
            comment: "Axiomatic derivation of TJ_2: BestEffort partial commit.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TJ_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_TJ_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_TJ_3",
            comment: "Axiomatic derivation of TJ_3: epoch-scoped atomicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/TJ_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AP_1",
            comment: "Axiomatic derivation of AP_1: saturation monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AP_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AP_2",
            comment: "Axiomatic derivation of AP_2: quality improvement.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AP_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AP_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AP_3",
            comment: "Axiomatic derivation of AP_3: deferred query liveness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AP_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // Amendment 66: Convergence Tower (EC_)
        Individual {
            id: "https://uor.foundation/proof/prf_EC_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_1",
            comment: "Axiomatic derivation of EC_1: phase half-turn convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_2",
            comment: "Axiomatic derivation of EC_2: conjugate involution.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_3",
            comment: "Axiomatic derivation of EC_3: pairwise commutator convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_4",
            comment: "Axiomatic derivation of EC_4: triple associator convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_4a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_4a",
            comment: "Axiomatic derivation of EC_4a: associator monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_4a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_4b",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_4b",
            comment: "Axiomatic derivation of EC_4b: associator finiteness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_4b"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_4c",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_4c",
            comment: "Axiomatic derivation of EC_4c: vanishing implies associativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_4c"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_EC_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_EC_5",
            comment: "Axiomatic derivation of EC_5: Adams termination.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/EC_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        // ── Amendment 67: Division Algebras (prf_DA_) ───────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_DA_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_1",
            comment: "Axiomatic derivation of DA_1: Cayley-Dickson R\u{2192}C.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_2",
            comment: "Axiomatic derivation of DA_2: Cayley-Dickson C\u{2192}H.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_3",
            comment: "Axiomatic derivation of DA_3: Cayley-Dickson H\u{2192}O.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_4",
            comment: "Axiomatic derivation of DA_4: Adams dimension restriction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_5",
            comment: "Axiomatic derivation of DA_5: convergence-level algebra \
                      correspondence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_6",
            comment: "Axiomatic derivation of DA_6: commutator-commutativity \
                      equivalence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DA_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DA_7",
            comment: "Axiomatic derivation of DA_7: associator-associativity \
                      equivalence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DA_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 68: Interaction Algebra proofs (IN_, AS_) ──────────
        Individual {
            id: "https://uor.foundation/proof/prf_IN_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_1",
            comment: "Axiomatic derivation of IN_1: interaction cost.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_2",
            comment: "Axiomatic derivation of IN_2: disjoint leases commute.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_3",
            comment: "Axiomatic derivation of IN_3: shared sites nonzero \
                      commutator.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_4",
            comment: "Axiomatic derivation of IN_4: negotiation convergence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_5",
            comment: "Axiomatic derivation of IN_5: commutative subspace \
                      selection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_6",
            comment: "Axiomatic derivation of IN_6: pairwise outcome space.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_7",
            comment: "Axiomatic derivation of IN_7: associative subalgebra \
                      selection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_8",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_8",
            comment: "Axiomatic derivation of IN_8: nerve Betti coupling \
                      bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_8"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IN_9",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IN_9",
            comment: "Axiomatic derivation of IN_9: Betti-disagreement \
                      associator bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IN_9"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AS_1",
            comment: "Axiomatic derivation of AS_1: \u{03b4}-\u{03b9}-\u{03ba} \
                      non-associativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AS_2",
            comment: "Axiomatic derivation of AS_2: \u{03b9}-\u{03b1}-\u{03bb} \
                      non-associativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AS_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AS_3",
            comment: "Axiomatic derivation of AS_3: \u{03bb}-\u{03ba}-\u{03b4} \
                      non-associativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AS_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_AS_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_AS_4",
            comment: "Axiomatic derivation of AS_4: non-associativity root \
                      cause.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/AS_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Composition"),
                ),
            ],
        },
        // ── Amendment 69: Monoidal Composition proofs (MO_) ──────────────
        Individual {
            id: "https://uor.foundation/proof/prf_MO_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MO_1",
            comment: "Axiomatic derivation of MO_1: monoidal unit law.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MO_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MO_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MO_2",
            comment: "Axiomatic derivation of MO_2: monoidal associativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MO_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MO_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MO_3",
            comment: "Axiomatic derivation of MO_3: certificate composition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MO_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MO_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MO_4",
            comment: "Axiomatic derivation of MO_4: saturation monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MO_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_MO_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_MO_5",
            comment: "Axiomatic derivation of MO_5: residual monotonicity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/MO_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 70: Operad Composition proofs (OP_) ───────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_OP_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OP_1",
            comment: "Axiomatic derivation of OP_1: site additivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OP_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OP_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OP_2",
            comment: "Axiomatic derivation of OP_2: grounding distributivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OP_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OP_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OP_3",
            comment: "Axiomatic derivation of OP_3: d_\u{0394} decomposition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OP_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OP_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OP_4",
            comment: "Axiomatic derivation of OP_4: tabular data decomposition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OP_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_OP_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_OP_5",
            comment: "Axiomatic derivation of OP_5: hierarchical data structure.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/OP_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 71: Effect Algebra proofs (prf_FX_1–prf_FX_7) ─────
        Individual {
            id: "https://uor.foundation/proof/prf_FX_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_1",
            comment: "Axiomatic derivation of FX_1: pinning site budget decrement.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_2",
            comment: "Axiomatic derivation of FX_2: unbinding site budget increment.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_3",
            comment: "Axiomatic derivation of FX_3: phase budget invariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_4",
            comment: "Axiomatic derivation of FX_4: disjoint commutativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_5",
            comment: "Axiomatic derivation of FX_5: composite delta additivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_6",
            comment: "Axiomatic derivation of FX_6: reversible inverse.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FX_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FX_7",
            comment: "Axiomatic derivation of FX_7: external shape compliance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FX_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 72: Predicate & Dispatch proofs (prf_PR_1–prf_PR_5) ──
        Individual {
            id: "https://uor.foundation/proof/prf_PR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PR_1",
            comment: "Axiomatic derivation of PR_1: predicate totality.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PR_2",
            comment: "Axiomatic derivation of PR_2: predicate purity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PR_3",
            comment: "Axiomatic derivation of PR_3: deterministic dispatch.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PR_4",
            comment: "Axiomatic derivation of PR_4: deterministic match.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PR_5",
            comment: "Axiomatic derivation of PR_5: typed guard transition.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 73: Reduction guard + Dispatch proofs ──────────────
        Individual {
            id: "https://uor.foundation/proof/prf_CG_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CG_1",
            comment: "Axiomatic derivation of CG_1: typed entry guard.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CG_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CG_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CG_2",
            comment: "Axiomatic derivation of CG_2: typed exit guard with effect.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CG_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DIS_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DIS_1",
            comment: "Axiomatic derivation of DIS_1: exhaustive exclusive table.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DIS_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_DIS_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_DIS_2",
            comment: "Axiomatic derivation of DIS_2: deterministic resolver selection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DIS_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 74: Parallel Composition proofs (prf_PAR_1–prf_PAR_5)
        Individual {
            id: "https://uor.foundation/proof/prf_PAR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PAR_1",
            comment: "Axiomatic derivation of PAR_1: disjoint commutativity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PAR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PAR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PAR_2",
            comment: "Axiomatic derivation of PAR_2: delta additivity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PAR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PAR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PAR_3",
            comment: "Axiomatic derivation of PAR_3: exhaustive partitioning.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PAR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PAR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PAR_4",
            comment: "Axiomatic derivation of PAR_4: interleaving invariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PAR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_PAR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_PAR_5",
            comment: "Axiomatic derivation of PAR_5: certificate conjunction.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/PAR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 75: Higher-Order proofs (prf_HO_1–prf_HO_4) ──────
        Individual {
            id: "https://uor.foundation/proof/prf_HO_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HO_1",
            comment: "Axiomatic derivation of HO_1: content-addressed certification.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HO_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HO_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HO_2",
            comment: "Axiomatic derivation of HO_2: application certification.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HO_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HO_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HO_3",
            comment: "Axiomatic derivation of HO_3: composition certification.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HO_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_HO_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_HO_4",
            comment: "Axiomatic derivation of HO_4: saturation equivalence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/HO_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 75: Stream proofs (prf_STR_1–prf_STR_6) ──────────
        Individual {
            id: "https://uor.foundation/proof/prf_STR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_1",
            comment: "Axiomatic derivation of STR_1: epoch termination.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_STR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_2",
            comment: "Axiomatic derivation of STR_2: saturation preservation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_STR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_3",
            comment: "Axiomatic derivation of STR_3: finite prefix computability.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_STR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_4",
            comment: "Axiomatic derivation of STR_4: unfold seed initialization.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_STR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_5",
            comment: "Axiomatic derivation of STR_5: continuation chaining.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_STR_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_STR_6",
            comment: "Axiomatic derivation of STR_6: lease expiry budget return.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/STR_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 76: Failure Algebra proofs (prf_FLR_1–prf_FLR_6) ──
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_1",
            comment: "Axiomatic derivation of FLR_1: coproduct exhaustiveness.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_2",
            comment: "Axiomatic derivation of FLR_2: total computation guarantee.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_3",
            comment: "Axiomatic derivation of FLR_3: sequential propagation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_4",
            comment: "Axiomatic derivation of FLR_4: parallel independence.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_5",
            comment: "Axiomatic derivation of FLR_5: recovery result.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_FLR_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_FLR_6",
            comment: "Axiomatic derivation of FLR_6: conjugate rollback.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/FLR_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 77: Linear + Subtyping proofs ─────────────────────
        Individual {
            id: "https://uor.foundation/proof/prf_LN_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_1",
            comment: "Axiomatic derivation of LN_1: exact coverage.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LN_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_2",
            comment: "Axiomatic derivation of LN_2: pinning effect.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LN_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_3",
            comment: "Axiomatic derivation of LN_3: consumption linearity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LN_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_4",
            comment: "Axiomatic derivation of LN_4: lease budget decrement.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LN_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_5",
            comment: "Axiomatic derivation of LN_5: lease expiry return.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_LN_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_LN_6",
            comment: "Axiomatic derivation of LN_6: geodesic linearity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/LN_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_1",
            comment: "Axiomatic derivation of SB_1: constraint superset.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_2",
            comment: "Axiomatic derivation of SB_2: resolution subset.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_3",
            comment: "Axiomatic derivation of SB_3: nerve sub-complex.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_4",
            comment: "Axiomatic derivation of SB_4: covariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_5",
            comment: "Axiomatic derivation of SB_5: contravariance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_SB_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_SB_6",
            comment: "Axiomatic derivation of SB_6: lattice depth.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/SB_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // ── Amendment 78: Bounded Recursion proofs (prf_BR_1–prf_BR_5) ──
        Individual {
            id: "https://uor.foundation/proof/prf_BR_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BR_1",
            comment: "Axiomatic derivation of BR_1: strict descent.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BR_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BR_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BR_2",
            comment: "Axiomatic derivation of BR_2: depth bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BR_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BR_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BR_3",
            comment: "Axiomatic derivation of BR_3: termination guarantee.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BR_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BR_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BR_4",
            comment: "Axiomatic derivation of BR_4: structural measure.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BR_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_BR_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_BR_5",
            comment: "Axiomatic derivation of BR_5: base predicate zero.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/BR_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 79: Address Region proofs (prf_RG_1–prf_RG_4) ─────
        Individual {
            id: "https://uor.foundation/proof/prf_RG_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RG_1",
            comment: "Axiomatic derivation of RG_1: nerve-determined working set.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RG_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/EulerPoincare"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RG_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RG_2",
            comment: "Axiomatic derivation of RG_2: diameter bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RG_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RG_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RG_3",
            comment: "Axiomatic derivation of RG_3: addressable space bound.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RG_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_RG_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_RG_4",
            comment: "Axiomatic derivation of RG_4: working set containment.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/RG_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        // ── Amendment 81: IO Boundary proofs (prf_IO_1–prf_IO_5) ────────
        Individual {
            id: "https://uor.foundation/proof/prf_IO_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IO_1",
            comment: "Axiomatic derivation of IO_1: ingest type conformance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IO_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IO_2",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IO_2",
            comment: "Axiomatic derivation of IO_2: emit type conformance.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IO_2"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IO_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IO_3",
            comment: "Axiomatic derivation of IO_3: grounding validity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IO_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IO_4",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IO_4",
            comment: "Axiomatic derivation of IO_4: projection validity.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IO_4"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IO_5",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IO_5",
            comment: "Axiomatic derivation of IO_5: non-vacuous crossing.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IO_5"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // Amendment 84: CompileUnit proofs (2)
        Individual {
            id: "https://uor.foundation/proof/prf_CS_6",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_6",
            comment: "Axiomatic derivation of CS_6: budget solvency rejection.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_6"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/Simplification"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_CS_7",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_CS_7",
            comment: "Axiomatic derivation of CS_7: unit address computation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/CS_7"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        // v0.2.1: Inhabitance verdict identity proofs
        Individual {
            id: "https://uor.foundation/proof/prf_IH_1",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IH_1",
            comment: "Axiomatic derivation of IH_1 (inhabitance soundness): \
                      cert:InhabitanceCertificate(T).verified iff carrier(T) \u{2260} \u{2205}.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IH_1"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IH_2a",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IH_2a",
            comment: "Cost identity for the 2-SAT decider on Is2SatShape \
                      instances. Inherited from the classical 2-SAT polynomial \
                      decision procedure.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IH_2a"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IH_2b",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IH_2b",
            comment: "Cost identity for the Horn-SAT decider on IsHornShape \
                      instances. Inherited from the classical Horn-SAT \
                      polynomial decision procedure via unit propagation.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IH_2b"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/proof/prf_IH_3",
            type_: "https://uor.foundation/proof/AxiomaticDerivation",
            label: "prf_IH_3",
            comment: "Carrier preservation under basis reduction. Discharged \
                      against resolver:JacobianGuidedResolver and \
                      resolver:guidingJacobian vocabulary.",
            properties: &[
                (
                    "https://uor.foundation/proof/provesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IH_3"),
                ),
                (
                    "https://uor.foundation/proof/universalScope",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/verified",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/proof/strategy",
                    IndividualValue::IriRef("https://uor.foundation/proof/RingAxiom"),
                ),
            ],
        },
    ]
}
