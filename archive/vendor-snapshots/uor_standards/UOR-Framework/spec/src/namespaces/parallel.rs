//! `parallel/` namespace — Parallel composition.
//!
//! The `parallel/` namespace formalizes independent computations over provably
//! disjoint site budgets. When execution order is irrelevant — because site
//! targets have empty intersection — computations may be composed in parallel.
//!
//! - **Amendment 74**: 5 classes, 10 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `parallel/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "parallel",
            iri: NS_PARALLEL,
            label: "UOR Parallel Composition",
            comment: "Independent computations over provably disjoint site \
                      budgets. Formalizes when execution order is irrelevant.",
            space: Space::Kernel,
            imports: &[
                NS_OP,
                NS_EFFECT,
                NS_MONOIDAL,
                NS_PARTITION,
                NS_CERT,
                NS_TRACE,
            ],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/parallel/ParallelProduct",
            label: "ParallelProduct",
            comment: "A \u{2225} B: two computations with provably disjoint \
                      site targets. Execution order does not affect the \
                      result.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/parallel/DisjointnessCertificate",
            label: "DisjointnessCertificate",
            comment: "A kernel-produced certificate attesting that the site \
                      targets of two computations are disjoint.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/parallel/SynchronizationPoint",
            label: "SynchronizationPoint",
            comment: "A point where two parallel computations must agree on \
                      shared state before proceeding. Only applicable when \
                      parallelism is partial.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/parallel/ParallelTrace",
            label: "ParallelTrace",
            comment: "A computation trace recording interleaved steps from \
                      two parallel computations. Valid iff every interleaving \
                      produces the same final context.",
            subclass_of: &["https://uor.foundation/trace/ComputationTrace"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/parallel/SitePartitioning",
            label: "SitePartitioning",
            comment: "A partition of the total site budget into disjoint \
                      subsets, one per parallel component. The sum of subset \
                      cardinalities equals the total site budget.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/parallel/leftComputation",
            label: "leftComputation",
            comment: "The left parallel component (itself a sequential \
                      computation).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: "https://uor.foundation/monoidal/MonoidalProduct",
        },
        Property {
            id: "https://uor.foundation/parallel/rightComputation",
            label: "rightComputation",
            comment: "The right parallel component.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: "https://uor.foundation/monoidal/MonoidalProduct",
        },
        Property {
            id: "https://uor.foundation/parallel/disjointnessCert",
            label: "disjointnessCert",
            comment: "The certificate proving site disjointness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: "https://uor.foundation/parallel/DisjointnessCertificate",
        },
        Property {
            id: "https://uor.foundation/parallel/certLeftTarget",
            label: "certLeftTarget",
            comment: "The site target of the left computation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/DisjointnessCertificate"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        Property {
            id: "https://uor.foundation/parallel/certRightTarget",
            label: "certRightTarget",
            comment: "The site target of the right computation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/DisjointnessCertificate"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        Property {
            id: "https://uor.foundation/parallel/syncSites",
            label: "syncSites",
            comment: "The shared sites requiring synchronization.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/parallel/SynchronizationPoint"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/parallel/partitionComponents",
            label: "partitionComponents",
            comment: "The disjoint subsets composing the full site budget.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/parallel/SitePartitioning"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/parallel/componentCount",
            label: "componentCount",
            comment: "Number of parallel components.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/SitePartitioning"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/parallel/isFullyDisjoint",
            label: "isFullyDisjoint",
            comment: "True iff site targets have zero overlap (no \
                      SynchronizationPoints required).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/parallel/disjointnessCommutation",
            label: "disjointnessCommutation",
            comment: "Declares whether this parallel product commutes with \
                      disjoint effects per FX_4.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: XSD_BOOLEAN,
        },
        // v0.2.1: Conformance ParallelShape backing properties
        Property {
            id: "https://uor.foundation/parallel/sitePartition",
            label: "sitePartition",
            comment: "The partition:Partition this parallel product is over.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/parallel/disjointnessWitness",
            label: "disjointnessWitness",
            comment: "IRI of a proof of pairwise disjointness of the partition \
                      components.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/parallel/ParallelProduct"),
            range: XSD_STRING,
        },
    ]
}
