//! `boundary/` namespace — IO boundary.
//!
//! The `boundary/` namespace formalizes the typed interface between kernel
//! computation and the external world. Every data flow into or out of the
//! ring crosses exactly one IOBoundary.
//!
//! - **Amendment 81**: 8 classes, 12 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `bridge` — kernel-computed, user-consumed.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `boundary/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "boundary",
            iri: NS_BOUNDARY,
            label: "UOR IO Boundary",
            comment: "Typed interface between kernel computation and the \
                      external world. Formalizes how data enters and exits \
                      the ring substrate.",
            space: Space::Bridge,
            imports: &[
                NS_OP,
                NS_SCHEMA,
                NS_EFFECT,
                NS_MORPHISM,
                NS_TYPE,
                NS_STATE,
                NS_CERT,
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
            id: "https://uor.foundation/boundary/IOBoundary",
            label: "IOBoundary",
            comment: "A typed interface point between the kernel and the \
                      external world. Every data flow into or out of the \
                      ring crosses exactly one IOBoundary.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/Source",
            label: "Source",
            comment: "A typed source of external data entering the ring. \
                      Carries an expected TypeDefinition describing the \
                      shape of incoming data.",
            subclass_of: &["https://uor.foundation/boundary/IOBoundary"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/Sink",
            label: "Sink",
            comment: "A typed destination for data leaving the ring. \
                      Carries an expected TypeDefinition describing the \
                      shape of outgoing data.",
            subclass_of: &["https://uor.foundation/boundary/IOBoundary"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/BoundaryEffect",
            label: "BoundaryEffect",
            comment: "An effect that crosses the kernel/external boundary. \
                      Specializes effect:ExternalEffect with explicit source \
                      or sink binding.",
            subclass_of: &["https://uor.foundation/effect/ExternalEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/IngestEffect",
            label: "IngestEffect",
            comment: "A BoundaryEffect that reads from a Source and \
                      produces a datum in the ring.",
            subclass_of: &["https://uor.foundation/boundary/BoundaryEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/EmitEffect",
            label: "EmitEffect",
            comment: "A BoundaryEffect that writes a ring datum to a Sink.",
            subclass_of: &["https://uor.foundation/boundary/BoundaryEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/BoundaryProtocol",
            label: "BoundaryProtocol",
            comment: "A specification of the data shape, ordering, and \
                      framing constraints for data crossing a boundary.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/boundary/BoundarySession",
            label: "BoundarySession",
            comment: "A Session that includes BoundaryEffects. Extends the \
                      session model to track which boundaries were crossed.",
            subclass_of: &["https://uor.foundation/state/Session"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/boundary/sourceType",
            label: "sourceType",
            comment: "The expected type of data arriving from this source.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/Source"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/boundary/sinkType",
            label: "sinkType",
            comment: "The expected type of data departing through this sink.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/Sink"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        // Target §3: `boundary:sourceGrounding` removed. The Rust-side
        // discipline lives in `Grounding::Map` (enforcement.rs), which
        // carries the kind discriminator at the type level. The
        // `source id : T via GroundingMap` grammar form declares the
        // binding; the materialized `boundary:Source` individual inherits
        // the binding through its Rust `Grounding` impl.
        // Target §3: `boundary:sinkProjection` removed. The Rust-side
        // discipline lives in `Sinking::ProjectionMap` (enforcement.rs),
        // which carries the kind discriminator at the type level. The
        // `sink id : T via ProjectionMap` grammar form declares the
        // binding; the materialized `boundary:Sink` individual inherits
        // the binding through its Rust `Sinking` impl.
        Property {
            id: "https://uor.foundation/boundary/effectBoundary",
            label: "effectBoundary",
            comment: "The boundary this effect crosses.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundaryEffect"),
            range: "https://uor.foundation/boundary/IOBoundary",
        },
        Property {
            id: "https://uor.foundation/boundary/ingestSource",
            label: "ingestSource",
            comment: "The source being read.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/IngestEffect"),
            range: "https://uor.foundation/boundary/Source",
        },
        Property {
            id: "https://uor.foundation/boundary/emitSink",
            label: "emitSink",
            comment: "The sink being written to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/EmitEffect"),
            range: "https://uor.foundation/boundary/Sink",
        },
        Property {
            id: "https://uor.foundation/boundary/protocolType",
            label: "protocolType",
            comment: "The type specification for boundary data.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundaryProtocol"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/boundary/protocolOrdering",
            label: "protocolOrdering",
            comment: "Sequencing constraints on boundary data.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundaryProtocol"),
            range: "https://uor.foundation/type/Conjunction",
        },
        Property {
            id: "https://uor.foundation/boundary/sessionBoundaries",
            label: "sessionBoundaries",
            comment: "The boundaries crossed during this session.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundarySession"),
            range: "https://uor.foundation/boundary/IOBoundary",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/boundary/isIdempotent",
            label: "isIdempotent",
            comment: "True iff applying the boundary effect twice produces \
                      the same result as applying it once.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundaryEffect"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/boundary/crossingCount",
            label: "crossingCount",
            comment: "Total number of boundary crossings in this session.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/boundary/BoundarySession"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
