//! `stream/` namespace — Productive streams.
//!
//! The `stream/` namespace formalizes coinductive sequences of reduction epochs.
//! Each epoch terminates independently; the stream is the unbounded composition
//! of terminating epochs.
//!
//! - **Amendment 75**: 6 classes, 13 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `stream/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "stream",
            iri: NS_STREAM,
            label: "UOR Productive Streams",
            comment: "Coinductive sequences of reduction epochs. Each epoch \
                      terminates independently; the stream is the unbounded \
                      composition of terminating epochs.",
            space: Space::Kernel,
            imports: &[
                NS_OP,
                NS_REDUCTION,
                NS_STATE,
                NS_MONOIDAL,
                NS_CERT,
                NS_MORPHISM,
                NS_SCHEMA,
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
            id: "https://uor.foundation/stream/ProductiveStream",
            label: "ProductiveStream",
            comment: "An unbounded sequence of reduction epochs where each \
                      epoch terminates and produces a well-typed output. \
                      The coinductive dual of a finite computation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/stream/Epoch",
            label: "Epoch",
            comment: "A single bounded iteration within a productive stream. \
                      Each epoch is a complete reduction execution from \
                      Initialization through Convergence.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/stream/EpochBoundary",
            label: "EpochBoundary",
            comment: "The transition point between consecutive epochs. \
                      Carries the continuation context and the epoch output.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/stream/StreamPrefix",
            label: "StreamPrefix",
            comment: "A finite prefix of a productive stream: the first k \
                      epochs and their outputs. Every finite prefix is \
                      computable in finite time.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/stream/StreamMorphism",
            label: "StreamMorphism",
            comment: "A transform between productive streams: maps each \
                      epoch of the source to an epoch of the target while \
                      preserving the productive property.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/stream/Unfold",
            label: "Unfold",
            comment: "The coinductive constructor: given an initial context \
                      and a step function (a morphism:ComputationDatum), \
                      produces a ProductiveStream. The step function must \
                      be certified to always reach reduction convergence.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/stream/epochReduction",
            label: "epochReduction",
            comment: "The reduction execution for this epoch.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Epoch"),
            range: "https://uor.foundation/reduction/EulerReduction",
        },
        Property {
            id: "https://uor.foundation/stream/epochOutput",
            label: "epochOutput",
            comment: "The output datum produced by this epoch.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Epoch"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/stream/epochContext",
            label: "epochContext",
            comment: "The context at the start of this epoch.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Epoch"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/stream/boundaryFrom",
            label: "boundaryFrom",
            comment: "The epoch that just completed.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/EpochBoundary"),
            range: "https://uor.foundation/stream/Epoch",
        },
        Property {
            id: "https://uor.foundation/stream/boundaryTo",
            label: "boundaryTo",
            comment: "The epoch about to begin.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/EpochBoundary"),
            range: "https://uor.foundation/stream/Epoch",
        },
        Property {
            id: "https://uor.foundation/stream/continuationContext",
            label: "continuationContext",
            comment: "The context carried across the boundary.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/EpochBoundary"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/stream/prefixEpochs",
            label: "prefixEpochs",
            comment: "The epochs in this prefix (ordered by stream:epochIndex).",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/stream/StreamPrefix"),
            range: "https://uor.foundation/stream/Epoch",
        },
        Property {
            id: "https://uor.foundation/stream/unfoldSeed",
            label: "unfoldSeed",
            comment: "The initial context for the stream.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Unfold"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/stream/unfoldStep",
            label: "unfoldStep",
            comment: "The certified step function producing each epoch.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Unfold"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/stream/unfoldResult",
            label: "unfoldResult",
            comment: "The stream produced by this unfold.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Unfold"),
            range: "https://uor.foundation/stream/ProductiveStream",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/stream/epochIndex",
            label: "epochIndex",
            comment: "Zero-based index of this epoch in the stream.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/Epoch"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/stream/prefixLength",
            label: "prefixLength",
            comment: "Number of epochs in this prefix.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/StreamPrefix"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/stream/isProductive",
            label: "isProductive",
            comment: "Always true by construction: every epoch terminates. \
                      Invariant, not computed.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/ProductiveStream"),
            range: XSD_BOOLEAN,
        },
        // v0.2.1: Conformance StreamShape backing properties
        Property {
            id: "https://uor.foundation/stream/stepTerm",
            label: "stepTerm",
            comment: "A term denoting a function from the current seed value \
                      to a pair (head, next_seed).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/ProductiveStream"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/stream/productivityWitness",
            label: "productivityWitness",
            comment: "IRI of a proof of stream productivity (coinductive \
                      well-foundedness).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/stream/ProductiveStream"),
            range: XSD_STRING,
        },
    ]
}
