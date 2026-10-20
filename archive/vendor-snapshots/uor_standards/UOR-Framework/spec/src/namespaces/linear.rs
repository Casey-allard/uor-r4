//! `linear/` namespace — Linear resources.
//!
//! The `linear/` namespace formalizes linear discipline on site consumption.
//! Each site in a complete resolution path is targeted by exactly one effect.
//!
//! - **Amendment 77**: 6 classes, 8 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `linear/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "linear",
            iri: NS_LINEAR,
            label: "UOR Linear Resources",
            comment: "Linear discipline on site consumption. Formalizes \
                      that each site in a complete resolution path is \
                      targeted by exactly one effect.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_EFFECT, NS_PARTITION, NS_TYPE, NS_STATE, NS_TRACE],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/linear/LinearSite",
            label: "LinearSite",
            comment: "A site index annotated with a linearity \
                      constraint: must be pinned exactly once in any \
                      complete resolution path.",
            subclass_of: &["https://uor.foundation/partition/SiteIndex"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/linear/LinearEffect",
            label: "LinearEffect",
            comment: "A PinningEffect that consumes its target LinearSite. \
                      After application, the site is no longer available \
                      for pinning by any subsequent effect.",
            subclass_of: &["https://uor.foundation/effect/PinningEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/linear/LinearTrace",
            label: "LinearTrace",
            comment: "A computation trace where every site in the budget is \
                      targeted by exactly one LinearEffect.",
            subclass_of: &["https://uor.foundation/trace/ComputationTrace"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/linear/LinearBudget",
            label: "LinearBudget",
            comment: "The multiset of LinearSites available at a given \
                      point in resolution. Starts as the full site budget; \
                      each LinearEffect removes exactly one element.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/linear/LeaseAllocation",
            label: "LeaseAllocation",
            comment: "A binding between a state:ContextLease and a subset \
                      of LinearSites. Formalizes what resources a lease \
                      claims.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/linear/AffineSite",
            label: "AffineSite",
            comment: "A site that may be pinned at most once (but need not \
                      be pinned). Relaxation of LinearSite for incomplete \
                      resolution paths.",
            subclass_of: &["https://uor.foundation/partition/SiteIndex"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/linear/linearTarget",
            label: "linearTarget",
            comment: "The single site consumed by this effect.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LinearEffect"),
            range: "https://uor.foundation/linear/LinearSite",
        },
        Property {
            id: "https://uor.foundation/linear/budgetSites",
            label: "budgetSites",
            comment: "The sites remaining in the budget.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/linear/LinearBudget"),
            range: "https://uor.foundation/linear/LinearSite",
        },
        Property {
            id: "https://uor.foundation/linear/budgetContext",
            label: "budgetContext",
            comment: "The context associated with this budget state.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LinearBudget"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/linear/leaseTarget",
            label: "leaseTarget",
            comment: "The sites claimed by this lease.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/linear/LeaseAllocation"),
            range: "https://uor.foundation/linear/LinearSite",
        },
        Property {
            id: "https://uor.foundation/linear/leaseSource",
            label: "leaseSource",
            comment: "The ContextLease individual that owns this allocation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LeaseAllocation"),
            range: "https://uor.foundation/state/ContextLease",
        },
        Property {
            id: "https://uor.foundation/linear/leaseAllocation",
            label: "leaseAllocation",
            comment: "Links a state:ContextLease to its LinearSite \
                      allocation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            // Cross-namespace domain: state:ContextLease
            // This property will NOT generate a trait method (cross-NS domain)
            domain: Some("https://uor.foundation/state/ContextLease"),
            range: "https://uor.foundation/linear/LeaseAllocation",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/linear/remainingCount",
            label: "remainingCount",
            comment: "Number of unconsumed sites. Equals freeRank on the \
                      associated context.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LinearBudget"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/linear/leaseCardinality",
            label: "leaseCardinality",
            comment: "Number of sites claimed by this lease.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LeaseAllocation"),
            range: XSD_POSITIVE_INTEGER,
        },
        // v0.2.2 Phase C.4 — stack budget bytes.
        Property {
            id: "https://uor.foundation/linear/stackBudgetBytes",
            label: "stackBudgetBytes",
            comment: "The stack budget available at the call site in bytes. \
                      On embedded targets this is compile-time-known; on std \
                      targets it is computed from the thread-local stack frame. \
                      Consumed by the MultiplicationResolver to bound the \
                      admissible Toom-Cook splitting factor R: deeper recursion \
                      requires more stack, so R is capped where stack would \
                      overflow.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/linear/LinearBudget"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
