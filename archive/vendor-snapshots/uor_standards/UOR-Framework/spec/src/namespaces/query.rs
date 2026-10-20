//! `query/` namespace — Information extraction queries (Amendment 24).
//!
//! Queries are the user-initiated requests for information from the UOR kernel.
//! They are kernel-executed: the user initiates a query, the kernel resolves it.
//!
//! Amendment 24 adds `RelationQuery`: a query with a grounded source address, a
//! typed relation constraint (read from the observable coordinate triple
//! `(d_R, d_H, d_I)`), and an open target `partition:FreeRank`. This is the
//! canonical form for all relational inference across NLP, ARC-AGI grids, music,
//! images, sensor streams, and logical formulae.
//!
//! Amendment 27 adds `SessionQuery`: a RelationQuery that explicitly declares its
//! session membership, enabling the conformance suite to validate session-scoped
//! site reduction across multi-turn Prism deployments.
//!
//! **Space classification:** `bridge` — user-initiated, kernel-executed.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `query/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "query",
            iri: NS_QUERY,
            label: "UOR Queries",
            comment: "Information extraction queries. Users initiate queries; \
                      the kernel resolves them against the ring substrate.",
            space: Space::Bridge,
            imports: &[
                NS_SCHEMA,
                NS_U,
                NS_TYPE,
                NS_PARTITION,
                NS_MORPHISM,
                NS_STATE,
            ],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/query/Query",
            label: "Query",
            comment: "A request for information from the UOR kernel. The root \
                      abstraction for all query types.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/query/CoordinateQuery",
            label: "CoordinateQuery",
            comment: "A query for the ring-coordinate position of a datum: its \
                      stratum, spectrum, and address within the ring geometry.",
            subclass_of: &["https://uor.foundation/query/Query"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/query/MetricQuery",
            label: "MetricQuery",
            comment: "A query for a metric value between two datums: ring distance, \
                      Hamming distance, or their divergence (curvature).",
            subclass_of: &["https://uor.foundation/query/Query"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/query/RepresentationQuery",
            label: "RepresentationQuery",
            comment: "A query for the canonical representation of a datum or term: \
                      its normal form under the active resolver strategy.",
            subclass_of: &["https://uor.foundation/query/Query"],
            disjoint_with: &[],
        },
        // Amendment 24: Surface Grounding Layer
        Class {
            id: "https://uor.foundation/query/RelationQuery",
            label: "RelationQuery",
            comment: "A Query with a known source address, a typed relation constraint, and an open \
                      target partition:FreeRank. Represents any question of the form: given this \
                      source symbol and relation type, what is the target? The relation type is read \
                      from the observable coordinate triple (d_R, d_H, d_I) — not externally supplied. \
                      Applies across NLP, ARC-AGI grids, music, images, sensor streams, and \
                      logical formulae.",
            subclass_of: &["https://uor.foundation/query/Query"],
            disjoint_with: &[],
        },
        // Amendment 23: Typed controlled vocabulary class
        Class {
            id: "https://uor.foundation/query/TriadProjection",
            label: "TriadProjection",
            comment: "A classification of coordinate types that a CoordinateQuery \
                      can extract. Each TriadProjection individual names a specific \
                      coordinate system (stratum, spectrum, address) replacing the \
                      string-valued query:coordinate property.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 27: Session-Scoped Resolution
        Class {
            id: "https://uor.foundation/query/SessionQuery",
            label: "SessionQuery",
            comment: "A RelationQuery that explicitly declares its session \
                      membership, allowing the conformance suite to validate \
                      session-scoped site reduction across multi-turn Prism \
                      deployments.",
            subclass_of: &["https://uor.foundation/query/RelationQuery"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/query/inputType",
            label: "inputType",
            comment: "The type of input the query accepts.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/Query"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/query/outputType",
            label: "outputType",
            comment: "The type of output the query produces.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/Query"),
            range: OWL_CLASS,
        },
        // query:coordinate property removed by Amendment 23 (replaced by hasTriadProjection)
        // Amendment 23: Typed controlled vocabulary property
        Property {
            id: "https://uor.foundation/query/hasTriadProjection",
            label: "hasTriadProjection",
            comment: "The typed coordinate kind this query extracts. Replaces \
                      the string-valued query:coordinate property with a typed \
                      reference to a TriadProjection individual.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/CoordinateQuery"),
            range: "https://uor.foundation/query/TriadProjection",
        },
        // Amendment 24: RelationQuery properties
        Property {
            id: "https://uor.foundation/query/sourceAddress",
            label: "sourceAddress",
            comment: "The ring address of the grounded source symbol.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/query/relationType",
            label: "relationType",
            comment: "The transformation type, expressed as a type:Conjunction composed \
                      from the primitive BoundConstraint basis. At inference time this is the \
                      output of an observable coordinate read on example pairs — computed from \
                      (d_R, d_H, d_I). Not an externally supplied input; read from the \
                      representation space.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/query/targetSite",
            label: "targetSite",
            comment: "The open site budget for the unknown target. Begins fully free; \
                      closes to isClosed = true upon resolution.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/partition/FreeRank",
        },
        Property {
            id: "https://uor.foundation/query/groundingMap",
            label: "groundingMap",
            comment: "The GroundingMap that resolved the source symbol to its ring address.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/morphism/GroundingMap",
        },
        Property {
            id: "https://uor.foundation/query/projectionMap",
            label: "projectionMap",
            comment: "The ProjectionMap that will render the resolved address neighbourhood \
                      back to surface symbols.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/morphism/ProjectionMap",
        },
        Property {
            id: "https://uor.foundation/query/sessionContext",
            label: "sessionContext",
            comment: "Accumulated session state carrying all state:Binding instances produced \
                      by prior queries. Each binding records a resolved address, its datum, and \
                      the type under which it was resolved. Prior bindings monotonically reduce \
                      the free site space for subsequent queries.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/RelationQuery"),
            range: "https://uor.foundation/state/Context",
        },
        // Amendment 27: Session-Scoped Resolution property
        Property {
            id: "https://uor.foundation/query/sessionMembership",
            label: "sessionMembership",
            comment: "The session this query belongs to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/query/SessionQuery"),
            range: "https://uor.foundation/state/Session",
        },
    ]
}

// Amendment 23: Typed controlled vocabulary individuals
fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/query/TwoAdicValuation",
            type_: "https://uor.foundation/query/TriadProjection",
            label: "TwoAdicValuation",
            comment: "The stratum coordinate: the layer position of a datum \
                      within the ring's stratification.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/query/WalshHadamardImage",
            type_: "https://uor.foundation/query/TriadProjection",
            label: "WalshHadamardImage",
            comment: "The spectrum coordinate: the spectral decomposition of a \
                      datum under the ring's Fourier analysis.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/query/Address",
            type_: "https://uor.foundation/query/TriadProjection",
            label: "Address",
            comment: "The address coordinate: the content-addressable position \
                      of a datum in the Braille glyph encoding. Renamed from \
                      RingElement in v0.2.2 W8 to unify vocabulary with the \
                      schema:Triad bundling properties.",
            properties: &[],
        },
    ]
}
