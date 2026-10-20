//! `carry/` namespace — Carry chain algebra and encoding optimization.
//!
//! The `carry/` namespace formalizes the carry chain
//! c\_{k+1} = or(and(x\_k, y\_k), and(xor(x\_k, y\_k), c\_k)) as a first-class
//! algebraic object. It extends the existing WC\_ (Witt\u{2013}carry) and CA\_
//! (carry arithmetic) identity families with refined carry event
//! classification, encoding configuration, and encoding quality metrics.
//!
//! - **Amendment 58**: 5 classes, 15 properties, carry algebra formalization
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `carry/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "carry",
            iri: NS_CARRY,
            label: "UOR Carry Algebra",
            comment: "Carry chain algebra: generate/propagate/kill event \
                      classification, carry profiles, encoding configurations, \
                      and encoding quality metrics for d_\u{0394} optimization.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_OBSERVABLE, NS_PARTITION],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/carry/CarryChain",
            label: "CarryChain",
            comment: "The Boolean function chain \
                      c_\u{007b}k+1\u{007d} = or(and(x_k, y_k), and(xor(x_k, y_k), c_k)). \
                      The carry chain is the algebraic mechanism behind the \
                      incompatibility metric d_\u{0394}.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/carry/CarryEvent",
            label: "CarryEvent",
            comment: "A single carry event at site k. Three kinds: Generate \
                      (and(x_k, y_k) = 1), Propagate (xor(x_k, y_k) = 1 and \
                      c_k = 1), Kill (neither generate nor propagate).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/carry/CarryProfile",
            label: "CarryProfile",
            comment: "The complete carry pattern for an addition x + y. \
                      Aggregates carry events across all sites into counts \
                      and position masks.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/carry/EncodingConfiguration",
            label: "EncodingConfiguration",
            comment: "A mapping from a finite symbol set S to Z/(2^k)Z where \
                      2^k \u{2265} |S|. Determines how domain values are \
                      represented as ring elements.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/carry/EncodingQuality",
            label: "EncodingQuality",
            comment: "The d_\u{0394} quality metric for an encoding over observed \
                      data. Measures how well an encoding minimizes carry-induced \
                      metric incompatibility.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.2 Phase D (Q4) — observable backing the carryConstraintKind
        // BoundConstraint individual.
        Class {
            id: "https://uor.foundation/carry/CarryDepthObservable",
            label: "CarryDepthObservable",
            comment: "Observes the carry depth of a Datum in the W\u{2082} \
                      tower, computed as the maximum carry-chain length in \
                      any operation producing it. Used as the bound \
                      observable for the carryConstraintKind BoundConstraint.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // CarryChain properties
        Property {
            id: "https://uor.foundation/carry/chainLength",
            label: "chainLength",
            comment: "The number of sites in this carry chain.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/generateMask",
            label: "generateMask",
            comment: "Bit mask of site positions where carry is generated: \
                      and(x_k, y_k) = 1.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/propagateMask",
            label: "propagateMask",
            comment: "Bit mask of site positions where carry propagates: \
                      xor(x_k, y_k) = 1.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/killMask",
            label: "killMask",
            comment: "Bit mask of site positions where carry is killed: \
                      neither generated nor propagated.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // CarryEvent properties
        Property {
            id: "https://uor.foundation/carry/eventKind",
            label: "eventKind",
            comment: "The kind of carry event: Generate, Propagate, or Kill.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryEvent"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/sitePosition",
            label: "sitePosition",
            comment: "The site index k at which this carry event occurs.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryEvent"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // CarryProfile properties
        Property {
            id: "https://uor.foundation/carry/carryCount",
            label: "carryCount",
            comment: "The total number of carry events in this profile.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryProfile"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/maxPropagationLength",
            label: "maxPropagationLength",
            comment: "The longest consecutive propagation run in this profile.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryProfile"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/profileChain",
            label: "profileChain",
            comment: "The carry chain that this profile summarizes.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/CarryProfile"),
            range: "https://uor.foundation/carry/CarryChain",
        },
        // EncodingConfiguration properties
        Property {
            id: "https://uor.foundation/carry/symbolSetSize",
            label: "symbolSetSize",
            comment: "The cardinality of the symbol set S being encoded.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingConfiguration"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/quantizationBits",
            label: "quantizationBits",
            comment: "The number of bits k used for encoding (2^k \u{2265} |S|).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingConfiguration"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/carry/encodingMap",
            label: "encodingMap",
            comment: "String representation of the mapping from symbols to ring elements.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingConfiguration"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // EncodingQuality properties
        Property {
            id: "https://uor.foundation/carry/meanDelta",
            label: "meanDelta",
            comment: "The mean d_\u{0394} over observed pairs for this encoding.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingQuality"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/carry/discriminationRatio",
            label: "discriminationRatio",
            comment: "The ratio of distinguishable pairs to total pairs under \
                      this encoding.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingQuality"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/carry/isOptimalEncoding",
            label: "isOptimalEncoding",
            comment: "Whether this encoding minimizes \u{03a3} d_\u{0394} over \
                      observed pairs.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/carry/EncodingQuality"),
            range: XSD_BOOLEAN,
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![]
}
