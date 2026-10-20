//! `region/` namespace — Address regions.
//!
//! The `region/` namespace formalizes spatial locality of content-addressed
//! ring elements. Defines computable working sets for resolver stages,
//! independent of physical memory layout.
//!
//! - **Amendment 79**: 5 classes, 11 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `region/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "region",
            iri: NS_REGION,
            label: "UOR Address Regions",
            comment: "Spatial locality of content-addressed ring elements. \
                      Defines computable working sets for resolver stages, \
                      independent of physical memory layout.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_U, NS_SCHEMA, NS_TYPE, NS_PARTITION, NS_OBSERVABLE],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/region/AddressRegion",
            label: "AddressRegion",
            comment: "A contiguous range of u:Element values accessible \
                      during a single reduction step. Defines the resolver\u{2019}s \
                      working set.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/region/RegionBound",
            label: "RegionBound",
            comment: "The boundary of an AddressRegion: a pair \
                      (lowerAddress, upperAddress) in the content-address \
                      ordering.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/region/LocalityMetric",
            label: "LocalityMetric",
            comment: "A metric on u:Element values determining which \
                      addresses are near each other for resolution purposes.",
            subclass_of: &["https://uor.foundation/observable/MetricObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/region/WorkingSet",
            label: "WorkingSet",
            comment: "The set of AddressRegions needed by a resolver at a \
                      specific reduction step for a specific type. Computable \
                      from the type\u{2019}s constraint nerve.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/region/RegionAllocation",
            label: "RegionAllocation",
            comment: "An assignment of AddressRegions to reduction steps for \
                      a given computation. Enables Prism to pre-compute \
                      memory layout.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/region/regionLower",
            label: "regionLower",
            comment: "The lower bound of the address range.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/RegionBound"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/region/regionUpper",
            label: "regionUpper",
            comment: "The upper bound of the address range.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/RegionBound"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/region/regionBound",
            label: "regionBound",
            comment: "The boundary of this region.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/AddressRegion"),
            range: "https://uor.foundation/region/RegionBound",
        },
        Property {
            id: "https://uor.foundation/region/localityMetric",
            label: "localityMetric",
            comment: "The metric defining contiguity for this region.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/AddressRegion"),
            range: "https://uor.foundation/region/LocalityMetric",
        },
        Property {
            id: "https://uor.foundation/region/workingSetRegions",
            label: "workingSetRegions",
            comment: "The address regions composing this working set.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/region/WorkingSet"),
            range: "https://uor.foundation/region/AddressRegion",
        },
        Property {
            id: "https://uor.foundation/region/workingSetStage",
            label: "workingSetStage",
            comment: "The reduction step this working set applies to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/WorkingSet"),
            // Full IRI: region/ doesn't import reduction/
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/region/workingSetType",
            label: "workingSetType",
            comment: "The type being resolved.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/WorkingSet"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/region/allocationStage",
            label: "allocationStage",
            comment: "Reduction steps in this allocation.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/region/RegionAllocation"),
            // Full IRI: region/ doesn't import reduction/
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/region/allocationWorkingSet",
            label: "allocationWorkingSet",
            comment: "The working sets assigned to each stage.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/region/RegionAllocation"),
            range: "https://uor.foundation/region/WorkingSet",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/region/regionCardinality",
            label: "regionCardinality",
            comment: "Number of addresses in this region.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/AddressRegion"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/region/workingSetSize",
            label: "workingSetSize",
            comment: "Total addresses across all regions in the working set.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/region/WorkingSet"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
