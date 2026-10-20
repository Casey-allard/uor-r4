//! `u/` namespace — Ring elements and content addressing.
//!
//! The `u/` namespace defines the content-addressing scheme used throughout
//! the UOR Framework. Elements are content-addressable identifiers derived
//! from ring-element hashes.
//!
//! **Space classification:** `kernel` — compiled into ROM.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `u/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "u",
            iri: NS_U,
            label: "UOR Content Addressing",
            comment: "Content-addressable identifiers for ring elements. \
                      Each element carries a unique content-derived identifier.",
            space: Space::Kernel,
            imports: &[],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![Class {
        id: "https://uor.foundation/u/Element",
        label: "Element",
        comment: "A content-addressable ring element. Each Element uniquely \
                      identifies a piece of content via its hash-derived identifier.",
        subclass_of: &[OWL_THING],
        disjoint_with: &[],
    }]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/u/length",
            label: "length",
            comment: "The number of Braille glyphs in an address string.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 13: Address Resolution
        Property {
            id: "https://uor.foundation/u/addresses",
            label: "addresses",
            comment: "The datum that this address references. Inverse of schema:glyph.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/u/digest",
            label: "digest",
            comment: "The content hash of this address. Format: \
                      (blake3|sha256) colon followed by 64 lowercase hex characters. \
                      The algorithm prefix must match u:digestAlgorithm.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: XSD_STRING,
        },
        // Amendment 43: Cryptographic Primitive Pinning
        Property {
            id: "https://uor.foundation/u/digestAlgorithm",
            label: "digestAlgorithm",
            comment: "The hash algorithm used to produce u:digest. \
                      Allowed values: 'blake3' (primary), 'sha256' (secondary).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/u/canonicalBytes",
            label: "canonicalBytes",
            comment: "The canonical byte serialisation of the addressed datum, \
                      per Amendment 43 section 2: header(k) || le_bytes(x, k+1). \
                      This is the exact byte string hashed to produce u:digest.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: XSD_HEX_BINARY,
        },
        Property {
            id: "https://uor.foundation/u/wittLength",
            label: "wittLength",
            comment: "The Witt level n of this element. The element encodes a \
                      datum in R_n = Z/(2^n)Z.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/u/Element"),
            range: XSD_POSITIVE_INTEGER,
        },
    ]
}
