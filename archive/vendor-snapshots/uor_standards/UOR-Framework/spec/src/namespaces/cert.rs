//! `cert/` namespace — Attestation certificates.
//!
//! Certificates are kernel-produced attestations of structural properties of
//! transforms and operations. They provide verifiable proofs that a specific
//! computation or operation satisfies a particular structural constraint.
//!
//! Amendment 25 adds `CompletenessAuditTrail` (an ordered collection of
//! `CompletenessWitness` records) with properties `auditTrail` and `witnessCount`
//! that provide full provenance for the completeness certification pathway.
//!
//! **Space classification:** `bridge` — kernel-produced, user-consumed.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `cert/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "cert",
            iri: NS_CERT,
            label: "UOR Certificates",
            comment: "Kernel-produced attestation certificates for transforms, \
                      isometries, and involutions. Each certificate verifies that \
                      a specific structural property holds.",
            space: Space::Bridge,
            imports: &[NS_OP, NS_PROOF, NS_SCHEMA, NS_TYPE],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/cert/Certificate",
            label: "Certificate",
            comment: "A kernel-produced attestation. The root class for all \
                      certificate types.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cert/TransformCertificate",
            label: "TransformCertificate",
            comment: "A certificate attesting to the properties of a morphism:Transform. \
                      Certifies that the transform maps source to target correctly.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cert/IsometryCertificate",
            label: "IsometryCertificate",
            comment: "A certificate attesting that a morphism:Isometry preserves \
                      metric distances. Certifies the transform is a metric isometry \
                      with respect to the specified metric.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cert/InvolutionCertificate",
            label: "InvolutionCertificate",
            comment: "A certificate attesting that an operation is an involution: \
                      f(f(x)) = x for all x in R_n.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Gap D: CompletenessCertificate
        Class {
            id: "https://uor.foundation/cert/CompletenessCertificate",
            label: "CompletenessCertificate",
            comment: "A certificate attesting that a type:CompleteType satisfies IT_7d: \
                      its constraint nerve has χ = n and all Betti numbers β_k = 0. \
                      Issued by the kernel after running the full ψ pipeline on the \
                      type's constraint set.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 25: Completeness Certification Pathway
        Class {
            id: "https://uor.foundation/cert/CompletenessAuditTrail",
            label: "CompletenessAuditTrail",
            comment: "An ordered collection of CompletenessWitness records belonging to \
                      a CompletenessCertificate. Provides full provenance of the \
                      certification process: every constraint applied, every site \
                      closed, in sequence.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 33: Grounding Certificate
        Class {
            id: "https://uor.foundation/cert/GroundingCertificate",
            label: "GroundingCertificate",
            comment: "A certificate attesting that a state:GroundedContext has \
                      reached full saturation (σ = 1, freeRank = 0, S = 0, \
                      T_ctx = 0) per SC_4. The session-layer dual of \
                      CompletenessCertificate.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 35: Geodesic Certificate
        Class {
            id: "https://uor.foundation/cert/GeodesicCertificate",
            label: "GeodesicCertificate",
            comment: "A certificate attesting that a trace:GeodesicTrace satisfies \
                      both GD_1 conditions: AR_1-ordered and DC_10-selected. \
                      Transforms ComputationTrace from descriptive to normative.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 36: Measurement Certificate
        Class {
            id: "https://uor.foundation/cert/MeasurementCertificate",
            label: "MeasurementCertificate",
            comment: "A certificate attesting that a trace:MeasurementEvent \
                      respected the von Neumann–Landauer bridge (QM_1): \
                      preCollapseEntropy = postCollapseLandauerCost at β* = ln 2.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 37: Geodesic Evidence Bundle (Gap 9)
        Class {
            id: "https://uor.foundation/cert/GeodesicEvidenceBundle",
            label: "GeodesicEvidenceBundle",
            comment: "A structured evidence bundle attesting that each sub-predicate \
                      of the geodesic condition (GD_6) holds independently: \
                      isAR1Ordered and isDC10Selected. Linked from \
                      GeodesicCertificate via evidenceBundle.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 37: Born Rule Verification (Gap 10)
        Class {
            id: "https://uor.foundation/cert/BornRuleVerification",
            label: "BornRuleVerification",
            comment: "A certificate attesting that a MeasurementEvent outcome \
                      probability matches the Born rule: P(outcome k) = |α_k|² \
                      (QM_5). Linked from MeasurementCertificate to provide \
                      probability distribution verification.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 41: LiftChainCertificate and ChainAuditTrail
        Class {
            id: "https://uor.foundation/cert/LiftChainCertificate",
            label: "LiftChainCertificate",
            comment: "A kernel-issued certificate attesting that a LiftChain from \
                      liftSourceLevel to liftTargetLevel is complete.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cert/ChainAuditTrail",
            label: "ChainAuditTrail",
            comment: "An ordered collection of per-step evidence records for a \
                      LiftChainCertificate.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1: Inhabitance Verdict Instantiation.
        // Multiple inheritance: ComputationCertificate (verdict role) and
        // Certificate (so cert:verified, cert:wittLength, cert:timestamp
        // inherit without re-declaration).
        Class {
            id: "https://uor.foundation/cert/InhabitanceCertificate",
            label: "InhabitanceCertificate",
            comment: "A ComputationCertificate verdict primitive that decides \
                      carrier non-emptiness on a type:ConstrainedType. Distinct \
                      from cert:CompletenessCertificate, which decides \
                      freeRank = 0 on the minimal basis. For ConstrainedType \
                      instances admitting multiple satisfying value tuples, \
                      InhabitanceCertificate.verified may be true while \
                      CompletenessCertificate.verified is false.",
            subclass_of: &[
                "https://uor.foundation/proof/ComputationCertificate",
                "https://uor.foundation/cert/Certificate",
            ],
            disjoint_with: &[],
        },
        // v0.2.2 Phase C.4 — MultiplicationCertificate.
        Class {
            id: "https://uor.foundation/cert/MultiplicationCertificate",
            label: "MultiplicationCertificate",
            comment: "A certificate attesting the cost-optimal Toom-Cook splitting \
                      factor R for a Datum<L> × Datum<L> multiplication at a given \
                      call-site context (stack budget, const-eval regime). Carries \
                      the chosen splitting factor, the recursive sub-multiplication \
                      count, and the accumulated Landauer cost in nats (priced per \
                      op:OA_5). Produced by resolver:MultiplicationResolver.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase E — PartitionCertificate attests the partition
        // component classification of a Datum (Irreducible / Reducible /
        // Units / Exterior).
        Class {
            id: "https://uor.foundation/cert/PartitionCertificate",
            label: "PartitionCertificate",
            comment: "A certificate attesting the partition component \
                      classification of a Datum, assigning it to one of \
                      Irreducible, Reducible, Units, or Exterior via the \
                      partition:PartitionComponent enumeration. Produced by \
                      the bridge partition walk during grounding.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Workstream C — impossibility certificates. Target §4.2 requires
        // resolver `certify` functions to return `Certified<…>` on both
        // success and failure sides; these two cert classes are the failure
        // carriers for the Phase C + Phase D resolvers respectively.
        Class {
            id: "https://uor.foundation/cert/GenericImpossibilityCertificate",
            label: "GenericImpossibilityCertificate",
            comment: "A certificate attesting that a resolver's verdict path \
                      produced a generic impossibility witness — the input \
                      failed the resolver's admissibility precondition or \
                      decision procedure in a way not covered by a more-\
                      specific witness. Returned by every Phase D resolver \
                      on failure.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cert/InhabitanceImpossibilityCertificate",
            label: "InhabitanceImpossibilityCertificate",
            comment: "A certificate attesting that the inhabitance decider \
                      concluded the input is unsatisfiable — there is no \
                      value in the carrier that satisfies the declared \
                      constraint conjunction. Returned by \
                      `resolver::inhabitance::certify` on failure.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/cert/transformType",
            label: "transformType",
            comment: "The type of transform this certificate attests to \
                      (e.g., 'isometry', 'embedding', 'action').",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/TransformCertificate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/cert/method",
            label: "method",
            comment: "The verification method used to produce this certificate \
                      (e.g., 'exhaustive_check', 'symbolic_proof', 'sampling').",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/Certificate"),
            range: "https://uor.foundation/proof/ProofStrategy",
        },
        Property {
            id: "https://uor.foundation/cert/operation",
            label: "operation",
            comment: "The operation this certificate applies to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/InvolutionCertificate"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/cert/verified",
            label: "verified",
            comment: "Whether this certificate has been verified by the kernel.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/Certificate"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/cert/wittLength",
            label: "wittLength",
            comment: "The Witt level at which this certificate was produced.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/Certificate"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cert/timestamp",
            label: "timestamp",
            comment: "The time at which this certificate was issued.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/Certificate"),
            range: XSD_DATETIME,
        },
        Property {
            id: "https://uor.foundation/cert/certifies",
            label: "certifies",
            comment: "The resource this certificate attests to. Links a certificate \
                      to the observable, transform, or other entity it covers.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/Certificate"),
            range: OWL_THING,
        },
        // Gap D: CompletenessCertificate property
        Property {
            id: "https://uor.foundation/cert/certifiedType",
            label: "certifiedType",
            comment: "The TypeDefinition whose completeness this certificate attests. \
                      The kernel issues this certificate after running the ψ pipeline \
                      on the type's constraint set and confirming IT_7d.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/CompletenessCertificate"),
            range: "https://uor.foundation/type/CompleteType",
        },
        // Amendment 25: Completeness Certification Pathway properties
        Property {
            id: "https://uor.foundation/cert/auditTrail",
            label: "auditTrail",
            comment: "The audit trail attesting the certification provenance. \
                      Links a CompletenessCertificate to its ordered sequence of \
                      CompletenessWitness records.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/CompletenessCertificate"),
            range: "https://uor.foundation/cert/CompletenessAuditTrail",
        },
        Property {
            id: "https://uor.foundation/cert/witnessCount",
            label: "witnessCount",
            comment: "Total number of witness steps in this audit trail.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/CompletenessAuditTrail"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 33: Grounding Certificate properties
        Property {
            id: "https://uor.foundation/cert/certifiedGrounding",
            label: "certifiedGrounding",
            comment: "The GroundedContext whose full saturation this certificate \
                      attests. Uses IRI string (cert cannot import state).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GroundingCertificate"),
            range: "https://uor.foundation/state/GroundedContext",
        },
        Property {
            id: "https://uor.foundation/cert/groundingWitness",
            label: "groundingWitness",
            comment: "The GroundingWitness providing step-by-step evidence \
                      of the saturation process.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GroundingCertificate"),
            range: "https://uor.foundation/state/GroundingWitness",
        },
        // Amendment 35: Geodesic Certificate properties
        Property {
            id: "https://uor.foundation/cert/certifiedGeodesic",
            label: "certifiedGeodesic",
            comment: "The GeodesicTrace whose geodesic status this certificate \
                      attests. Uses IRI string (cert cannot import trace).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GeodesicCertificate"),
            range: "https://uor.foundation/trace/GeodesicTrace",
        },
        Property {
            id: "https://uor.foundation/cert/geodesicTrace",
            label: "geodesicTrace",
            comment: "The computation trace that this GeodesicCertificate covers. \
                      Redundant with certifiedGeodesic but expresses the inverse \
                      direction for queryability.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GeodesicCertificate"),
            range: "https://uor.foundation/trace/GeodesicTrace",
        },
        // Amendment 36: Measurement Certificate properties
        Property {
            id: "https://uor.foundation/cert/certifiedMeasurement",
            label: "certifiedMeasurement",
            comment: "The MeasurementEvent whose QM_1 compliance this certificate \
                      attests. Uses IRI string (cert cannot import trace).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MeasurementCertificate"),
            range: "https://uor.foundation/trace/MeasurementEvent",
        },
        Property {
            id: "https://uor.foundation/cert/vonNeumannEntropy",
            label: "vonNeumannEntropy",
            comment: "The von Neumann entropy S_vN of the pre-measurement \
                      SuperposedSiteState, recorded by this certificate.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MeasurementCertificate"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/cert/landauerCost",
            label: "landauerCost",
            comment: "The Landauer cost incurred by the projective collapse, \
                      recorded by this certificate. Equals vonNeumannEntropy \
                      at β* = ln 2 per QM_1.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MeasurementCertificate"),
            range: XSD_DECIMAL,
        },
        // Amendment 37: Geodesic evidence bundle link (Gap 9)
        Property {
            id: "https://uor.foundation/cert/evidenceBundle",
            label: "evidenceBundle",
            comment: "The GeodesicEvidenceBundle attesting to the decomposed \
                      sub-predicates (isAR1Ordered, isDC10Selected) of this \
                      GeodesicCertificate's geodesic claim.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GeodesicCertificate"),
            range: "https://uor.foundation/cert/GeodesicEvidenceBundle",
        },
        // Amendment 37: Born rule verification flag (Gap 10)
        Property {
            id: "https://uor.foundation/cert/bornRuleVerified",
            label: "bornRuleVerified",
            comment: "Whether this BornRuleVerification certificate confirms \
                      that all outcome probabilities match the Born rule \
                      (QM_5): P(k) = |α_k|² for every site k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/BornRuleVerification"),
            range: XSD_BOOLEAN,
        },
        // Amendment 41: LiftChainCertificate properties
        Property {
            id: "https://uor.foundation/cert/certifiedChain",
            label: "certifiedChain",
            comment: "The LiftChain this certificate attests to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/LiftChainCertificate"),
            range: "https://uor.foundation/type/LiftChain",
        },
        Property {
            id: "https://uor.foundation/cert/chainAuditTrail",
            label: "chainAuditTrail",
            comment: "The ordered per-step evidence for this certificate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/LiftChainCertificate"),
            range: "https://uor.foundation/cert/ChainAuditTrail",
        },
        Property {
            id: "https://uor.foundation/cert/targetLevel",
            label: "targetLevel",
            comment: "The quantum level Q_k at which the certificate was issued.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/LiftChainCertificate"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/cert/sourceLevel",
            label: "sourceLevel",
            comment: "The quantum level Q_j from which the tower was started.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/LiftChainCertificate"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        // Amendment 41: ChainAuditTrail property
        Property {
            id: "https://uor.foundation/cert/chainStepCount",
            label: "chainStepCount",
            comment: "Number of lift steps in this ChainAuditTrail. Must equal \
                      chainLength of the certified LiftChain. Distinct from \
                      witnessCount (domain-locked to CompletenessAuditTrail).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/ChainAuditTrail"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 38: GeodesicEvidenceBundle sub-predicate properties
        Property {
            id: "https://uor.foundation/cert/isAR1Ordered",
            label: "isAR1Ordered",
            comment: "True iff the linked GeodesicTrace is ordered by the AR_1 \
                      canonical rewriting rule (smallest lexicographic \
                      representative first).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GeodesicEvidenceBundle"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/cert/isDC10Selected",
            label: "isDC10Selected",
            comment: "True iff each constraint in the trace was selected by \
                      the DC_10 Jacobian oracle at the step where it was \
                      applied.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/GeodesicEvidenceBundle"),
            range: XSD_BOOLEAN,
        },
        // v0.2.1: InhabitanceCertificate properties
        Property {
            id: "https://uor.foundation/cert/witness",
            label: "witness",
            comment: "A specific value tuple in the carrier when verified is \
                      true; absent otherwise. The witness form for \
                      cert:InhabitanceCertificate.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cert/InhabitanceCertificate"),
            range: "https://uor.foundation/schema/ValueTuple",
        },
        Property {
            id: "https://uor.foundation/cert/searchTrace",
            label: "searchTrace",
            comment: "The audit trail of the inhabitance search that produced \
                      this certificate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/InhabitanceCertificate"),
            range: "https://uor.foundation/trace/InhabitanceSearchTrace",
        },
        Property {
            id: "https://uor.foundation/cert/grounded",
            label: "grounded",
            comment: "The type:ConstrainedType this InhabitanceCertificate is \
                      issued for.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/InhabitanceCertificate"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        // v0.2.2 Phase C.4 — MultiplicationCertificate evidence properties.
        Property {
            id: "https://uor.foundation/cert/splittingFactor",
            label: "splittingFactor",
            comment: "The Toom-Cook splitting factor R chosen by the multiplication \
                      resolver. R = 1 is schoolbook (the const-eval bottom-out); \
                      R = 2 is Karatsuba; R >= 3 is Toom-k. The resolver picks the \
                      cost-optimal R subject to the call-site's stack budget and \
                      const-eval depth constraints.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MultiplicationCertificate"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cert/subMultiplicationCount",
            label: "subMultiplicationCount",
            comment: "The number of recursive sub-multiplications the chosen \
                      splitting factor induces for one Datum<L> × Datum<L> \
                      multiplication at this call site. For splitting factor R, \
                      the count is (2R - 1) for R > 1, and 1 for R = 1.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MultiplicationCertificate"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cert/landauerCostNats",
            label: "landauerCostNats",
            comment: "The accumulated Landauer cost of the certified multiplication \
                      in nats, priced per op:OA_5 (each irreversible bit erasure \
                      costs ln 2 nats in the Archimedean completion). Unit: \
                      observable:Nats. The value is an xsd:decimal.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cert/MultiplicationCertificate"),
            range: XSD_DECIMAL,
        },
    ]
}
