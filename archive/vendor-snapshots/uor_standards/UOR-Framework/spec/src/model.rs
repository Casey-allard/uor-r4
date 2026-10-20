//! Core ontology model types.
//!
//! These types represent the UOR Foundation ontology vocabulary as typed Rust
//! data. All instances are built as owned `Vec`s and referenced via borrows.
//! The top-level entry point is [`Ontology::full()`](crate::Ontology::full).

use std::fmt;

/// Kernel/user/bridge classification for each namespace module.
///
/// - `Kernel`: compiled into ROM; immutable; `u/`, `schema/`, `op/`
/// - `User`: parameterizable at runtime; `type/`, `state/`, `morphism/`
/// - `Bridge`: kernel-computed, user-consumed; all remaining namespaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Space {
    /// Immutable kernel-space: compiled into ROM.
    Kernel,
    /// Parameterizable user-space: runtime declarations.
    User,
    /// Bridge: kernel-computed, user-consumed.
    Bridge,
}

impl Space {
    /// Returns the string value used in the `uor:space` annotation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Space::Kernel => "kernel",
            Space::User => "user",
            Space::Bridge => "bridge",
        }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A UOR Foundation namespace (e.g., `u/`, `schema/`, `op/`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Namespace {
    /// The prefix used in the `@context` (e.g., `"u"`).
    pub prefix: &'static str,
    /// The full IRI of the namespace (e.g., `"https://uor.foundation/u/"`).
    pub iri: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Description of the namespace.
    pub comment: &'static str,
    /// Kernel/user/bridge classification (Amendment 8).
    pub space: Space,
    /// Full IRIs of imported namespaces (`owl:imports`).
    pub imports: &'static [&'static str],
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: <{}>", self.prefix, self.iri)
    }
}

/// An OWL class definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Class {
    /// Full IRI (e.g., `"https://uor.foundation/u/Element"`).
    pub id: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Description.
    pub comment: &'static str,
    /// Full IRIs of parent classes (`rdfs:subClassOf`).
    pub subclass_of: &'static [&'static str],
    /// Full IRIs of mutually exclusive classes (`owl:disjointWith`).
    pub disjoint_with: &'static [&'static str],
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.label, self.id)
    }
}

/// Whether a property is a datatype, object, or annotation property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PropertyKind {
    /// `owl:DatatypeProperty` — relates a resource to an XSD literal.
    Datatype,
    /// `owl:ObjectProperty` — relates two resources.
    Object,
    /// `owl:AnnotationProperty` — used for documentation; not for reasoning.
    Annotation,
}

impl fmt::Display for PropertyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyKind::Datatype => f.write_str("DatatypeProperty"),
            PropertyKind::Object => f.write_str("ObjectProperty"),
            PropertyKind::Annotation => f.write_str("AnnotationProperty"),
        }
    }
}

/// An OWL property definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Property {
    /// Full IRI.
    pub id: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Description.
    pub comment: &'static str,
    /// Datatype, object, or annotation property.
    pub kind: PropertyKind,
    /// Whether this is also an `owl:FunctionalProperty`.
    pub functional: bool,
    /// Whether the ontology author commits to asserting this property
    /// on every individual whose `rdf:type` matches `domain` (or a
    /// subclass of it).
    ///
    /// - `true` → the property is required. Lean generates the
    ///   corresponding structure field as a bare `T` (functional) or
    ///   `Array T` (non-functional), and the
    ///   `meta/required_property_coverage` conformance check fails if
    ///   any domain-matching individual lacks an assertion.
    /// - `false` → the property is optional. Lean generates the
    ///   structure field as `Option T` (functional) or `Array T`
    ///   (non-functional), and a missing assertion becomes
    ///   proven-by-absence (`none` / `#[]`) instead of a gap.
    ///
    /// Default in existing `Property { ... }` literals is `false` —
    /// opt into strictness explicitly.
    pub required: bool,
    /// Full IRI of the domain class, or `None` if unspecified.
    pub domain: Option<&'static str>,
    /// Full IRI of the range class or XSD datatype.
    pub range: &'static str,
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.label, self.id)
    }
}

/// A value in a named individual's property assertion.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum IndividualValue {
    /// A plain string literal.
    Str(&'static str),
    /// An integer literal.
    Int(i64),
    /// A boolean literal.
    Bool(bool),
    /// A decimal literal, projecting to `xsd:decimal`. Stored as
    /// IEEE 754 double. The canonical form is a bit pattern so the
    /// enum retains `PartialEq` and `Hash` via bit-level comparison.
    Float(f64),
    /// An IRI reference to another resource.
    IriRef(&'static str),
    /// An ordered `rdf:List` of IRI references (used for `op:composedOf`).
    List(&'static [&'static str]),
}

// `IndividualValue` can no longer derive `Eq`/`Hash` automatically
// because `f64` is neither `Eq` nor `Hash`. The only public use of
// these traits was insertion into hash-based collections keyed by
// `IndividualValue`, which does not happen in the codegen pipeline —
// we always key by property IRI instead. The trait bounds are
// therefore relaxed to `PartialEq` alone.

impl fmt::Display for IndividualValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndividualValue::Str(s) => write!(f, "\"{s}\""),
            IndividualValue::Int(n) => write!(f, "{n}"),
            IndividualValue::Bool(b) => write!(f, "{b}"),
            IndividualValue::Float(x) => write!(f, "{x}"),
            IndividualValue::IriRef(iri) => write!(f, "<{iri}>"),
            IndividualValue::List(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "<{item}>")?;
                }
                f.write_str("]")
            }
        }
    }
}

/// A named individual (OWL `owl:NamedIndividual`).
///
/// `Eq`/`Hash` were dropped when `IndividualValue::Float(f64)` was
/// added, since `f64` has no such instances. No code in the workspace
/// inserted `Individual` values into hash-based collections — lookups
/// always key by IRI instead.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Individual {
    /// Full IRI.
    pub id: &'static str,
    /// Full IRI of the class this individual is an instance of.
    pub type_: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Description.
    pub comment: &'static str,
    /// Property assertions: pairs of (property IRI, value).
    pub properties: &'static [(&'static str, IndividualValue)],
}

impl fmt::Display for Individual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.label, self.id)
    }
}

/// A complete namespace module: namespace metadata + classes + properties + individuals.
///
/// `Eq` dropped because `Individual` no longer implements it (see the
/// comment on `Individual`). `PartialEq` remains sufficient for test
/// assertions that compare whole modules.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NamespaceModule {
    /// Namespace metadata.
    pub namespace: Namespace,
    /// All OWL classes defined in this namespace.
    pub classes: Vec<Class>,
    /// All OWL properties defined in this namespace.
    pub properties: Vec<Property>,
    /// All named individuals declared in this namespace.
    pub individuals: Vec<Individual>,
}

/// An annotation property defined at the ontology root level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AnnotationProperty {
    /// Full IRI.
    pub id: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Description.
    pub comment: &'static str,
    /// Full IRI of the range (typically `xsd:string`).
    pub range: &'static str,
}

/// The complete UOR Foundation ontology.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Ontology {
    /// Ontology version (e.g., `"1.1.0"`).
    pub version: &'static str,
    /// Base IRI of the ontology (e.g., `"https://uor.foundation/"`).
    pub base_iri: &'static str,
    /// All namespace modules in dependency order.
    pub namespaces: Vec<NamespaceModule>,
    /// Root-level annotation properties (Amendment 8: `uor:space`).
    pub annotation_properties: Vec<AnnotationProperty>,
}

impl Ontology {
    /// Looks up a class by its full IRI. Returns `None` if not found.
    #[must_use]
    pub fn find_class(&self, iri: &str) -> Option<&Class> {
        self.namespaces
            .iter()
            .flat_map(|m| m.classes.iter())
            .find(|c| c.id == iri)
    }

    /// Looks up a class by its local name (the last `/`-delimited segment of
    /// its IRI). Used by both Rust and Lean codegen to resolve vocabulary-enum
    /// class definitions without hardcoding IRIs.
    #[must_use]
    pub fn find_class_by_local_name(&self, local: &str) -> Option<&Class> {
        let suffix = format!("/{local}");
        self.namespaces
            .iter()
            .flat_map(|m| m.classes.iter())
            .find(|c| c.id.ends_with(&suffix))
    }

    /// Returns the canonical doc comment for a vocabulary-enum class by local
    /// name, looked up from the ontology's own class definitions.
    ///
    /// Used by both `codegen/` (Rust) and `lean-codegen/` to guarantee that
    /// enum-class doc comments in the two generators cannot drift.
    #[must_use]
    pub fn enum_class_comment(&self, class_local_name: &str) -> Option<&'static str> {
        self.find_class_by_local_name(class_local_name)
            .map(|c| c.comment)
    }

    /// Returns the namespace prefix (e.g. `"op"`, `"resolver"`) owning a
    /// vocabulary-enum class, looked up by local name.
    #[must_use]
    pub fn enum_class_namespace(&self, class_local_name: &str) -> Option<&'static str> {
        let suffix = format!("/{class_local_name}");
        for module in &self.namespaces {
            if module.classes.iter().any(|c| c.id.ends_with(&suffix)) {
                return Some(module.namespace.prefix);
            }
        }
        None
    }

    /// Looks up a property by its full IRI. Returns `None` if not found.
    #[must_use]
    pub fn find_property(&self, iri: &str) -> Option<&Property> {
        self.namespaces
            .iter()
            .flat_map(|m| m.properties.iter())
            .find(|p| p.id == iri)
    }

    /// Looks up a named individual by its full IRI. Returns `None` if not found.
    #[must_use]
    pub fn find_individual(&self, iri: &str) -> Option<&Individual> {
        self.namespaces
            .iter()
            .flat_map(|m| m.individuals.iter())
            .find(|i| i.id == iri)
    }

    /// Returns the total number of classes across all namespaces.
    #[must_use]
    pub fn class_count(&self) -> usize {
        self.namespaces.iter().map(|m| m.classes.len()).sum()
    }

    /// Returns the total number of properties across all namespaces.
    ///
    /// Includes the ontology-level `uor:space` annotation property (Amendment 8)
    /// which is defined outside any specific namespace module.
    #[must_use]
    pub fn property_count(&self) -> usize {
        // +1 for the global uor:space annotation property (Amendment 8)
        self.namespaces
            .iter()
            .map(|m| m.properties.len())
            .sum::<usize>()
            + 1
    }

    /// Returns the total number of named individuals across all namespaces.
    #[must_use]
    pub fn individual_count(&self) -> usize {
        self.namespaces.iter().map(|m| m.individuals.len()).sum()
    }

    /// Returns the set of class local names represented as Rust enums or structs
    /// (not traits) in the generated `uor-foundation` crate.
    ///
    /// Most are OWL vocabulary classes detected by `detect_vocabulary_enum()`.
    /// `WittLevel` is a struct (not enum) but also skips trait generation.
    #[must_use]
    pub fn enum_class_names() -> &'static [&'static str] {
        &[
            "AchievabilityStatus",
            "ComplexityClass",
            "ExecutionPolicyKind",
            "GeometricCharacter",
            "GroundingPhase",
            "MeasurementUnit",
            "MetricAxis",
            "PhaseBoundaryType",
            "ProofStrategy",
            "QuantifierKind",
            "RewriteRule",
            "SessionBoundaryType",
            "TriadProjection",
            "ValidityScopeKind",
            "VarianceAnnotation",
            "VerificationDomain",
            "ViolationKind",
            "WittLevel",
            "PartitionComponent",
        ]
    }
}

/// Returns the `uor:space` annotation property (Amendment 8).
#[must_use]
pub fn annotation_space_property() -> AnnotationProperty {
    AnnotationProperty {
        id: "https://uor.foundation/space",
        label: "space",
        comment: "Whether this module belongs to kernel-space (immutable, compiled), \
                  user-space (parameterizable, runtime), or bridge (kernel-computed, \
                  user-consumed). Values: 'kernel', 'user', 'bridge'.",
        range: "http://www.w3.org/2001/XMLSchema#string",
    }
}

/// Rewrites identity individuals' `lhs`, `rhs`, and `forAll` property values
/// from `IndividualValue::Str` to `IndividualValue::IriRef` pointing to the
/// corresponding `schema/term_{localName}_{suffix}` AST individual.
///
/// Uses `Box::leak` to produce `&'static` references from dynamic strings.
/// Call this on the `Vec<Individual>` returned from each namespace's identity
/// data before returning from `individuals()`.
#[must_use]
pub fn rewrite_identity_ast_refs(mut individuals: Vec<Individual>) -> Vec<Individual> {
    let identity_type = "https://uor.foundation/op/Identity";
    let lhs_prop = "https://uor.foundation/op/lhs";
    let rhs_prop = "https://uor.foundation/op/rhs";
    let forall_prop = "https://uor.foundation/op/forAll";

    for ind in &mut individuals {
        if ind.type_ != identity_type {
            continue;
        }

        let name = match ind.id.rfind('/') {
            Some(pos) => &ind.id[pos + 1..],
            None => ind.id,
        };

        let mut needs_rewrite = false;
        for &(prop, ref val) in ind.properties.iter() {
            if (prop == lhs_prop || prop == rhs_prop || prop == forall_prop)
                && matches!(val, IndividualValue::Str(_))
            {
                needs_rewrite = true;
                break;
            }
        }

        if !needs_rewrite {
            continue;
        }

        let mut new_props: Vec<(&'static str, IndividualValue)> = Vec::new();
        for &(prop, ref val) in ind.properties.iter() {
            if matches!(val, IndividualValue::Str(_))
                && (prop == lhs_prop || prop == rhs_prop || prop == forall_prop)
            {
                let suffix = if prop == lhs_prop {
                    "lhs"
                } else if prop == rhs_prop {
                    "rhs"
                } else {
                    "forAll"
                };
                let iri = format!("https://uor.foundation/schema/term_{name}_{suffix}");
                let iri: &'static str = Box::leak(iri.into_boxed_str());
                new_props.push((prop, IndividualValue::IriRef(iri)));
            } else {
                new_props.push((prop, *val));
            }
        }
        let leaked: &'static [(&'static str, IndividualValue)] =
            Box::leak(new_props.into_boxed_slice());
        ind.properties = leaked;
    }

    individuals
}

/// Standard IRI constants used across all namespace modules.
pub mod iris {
    /// OWL namespace.
    pub const OWL: &str = "http://www.w3.org/2002/07/owl#";
    /// RDF namespace.
    pub const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    /// RDFS namespace.
    pub const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
    /// XSD namespace.
    pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
    /// SHACL namespace.
    pub const SH: &str = "http://www.w3.org/ns/shacl#";

    /// UOR Foundation base IRI.
    pub const UOR: &str = "https://uor.foundation/";

    // Namespace IRIs
    /// Content addressing namespace.
    pub const NS_U: &str = "https://uor.foundation/u/";
    /// Core schema namespace.
    pub const NS_SCHEMA: &str = "https://uor.foundation/schema/";
    /// Operations namespace.
    pub const NS_OP: &str = "https://uor.foundation/op/";
    /// Query namespace.
    pub const NS_QUERY: &str = "https://uor.foundation/query/";
    /// Resolver namespace.
    pub const NS_RESOLVER: &str = "https://uor.foundation/resolver/";
    /// Type system namespace.
    pub const NS_TYPE: &str = "https://uor.foundation/type/";
    /// Partition namespace.
    pub const NS_PARTITION: &str = "https://uor.foundation/partition/";
    /// Observable namespace.
    pub const NS_OBSERVABLE: &str = "https://uor.foundation/observable/";
    /// Proof namespace.
    pub const NS_PROOF: &str = "https://uor.foundation/proof/";
    /// Derivation namespace.
    pub const NS_DERIVATION: &str = "https://uor.foundation/derivation/";
    /// Trace namespace.
    pub const NS_TRACE: &str = "https://uor.foundation/trace/";
    /// Certificate namespace.
    pub const NS_CERT: &str = "https://uor.foundation/cert/";
    /// Morphism namespace.
    pub const NS_MORPHISM: &str = "https://uor.foundation/morphism/";
    /// State namespace.
    pub const NS_STATE: &str = "https://uor.foundation/state/";
    /// Homology namespace.
    pub const NS_HOMOLOGY: &str = "https://uor.foundation/homology/";
    /// Cohomology namespace.
    pub const NS_COHOMOLOGY: &str = "https://uor.foundation/cohomology/";
    /// Carry algebra namespace.
    pub const NS_CARRY: &str = "https://uor.foundation/carry/";
    /// Reduction namespace.
    pub const NS_REDUCTION: &str = "https://uor.foundation/reduction/";
    /// Convergence namespace.
    pub const NS_CONVERGENCE: &str = "https://uor.foundation/convergence/";
    /// Division algebras namespace.
    pub const NS_DIVISION: &str = "https://uor.foundation/division/";
    /// Interaction algebra namespace.
    pub const NS_INTERACTION: &str = "https://uor.foundation/interaction/";
    /// Monoidal composition namespace.
    pub const NS_MONOIDAL: &str = "https://uor.foundation/monoidal/";
    /// Operad composition namespace.
    pub const NS_OPERAD: &str = "https://uor.foundation/operad/";
    /// Predicate and dispatch namespace.
    pub const NS_PREDICATE: &str = "https://uor.foundation/predicate/";
    /// Parallel composition namespace.
    pub const NS_PARALLEL: &str = "https://uor.foundation/parallel/";
    /// Productive streams namespace.
    pub const NS_STREAM: &str = "https://uor.foundation/stream/";
    /// Failure algebra namespace.
    pub const NS_FAILURE: &str = "https://uor.foundation/failure/";
    /// Linear resources namespace.
    pub const NS_LINEAR: &str = "https://uor.foundation/linear/";
    /// Bounded recursion namespace.
    pub const NS_RECURSION: &str = "https://uor.foundation/recursion/";
    /// Address regions namespace.
    pub const NS_REGION: &str = "https://uor.foundation/region/";
    /// IO boundary namespace.
    pub const NS_BOUNDARY: &str = "https://uor.foundation/boundary/";
    /// Conformance shapes namespace.
    pub const NS_CONFORMANCE: &str = "https://uor.foundation/conformance/";

    // ── New namespaces (Amendments 71+) ──────────────────────────────

    /// Effect algebra namespace.
    pub const NS_EFFECT: &str = "https://uor.foundation/effect/";
    /// Foundation-level layout invariants namespace (Product/Coproduct
    /// Completion Amendment). Complements op-namespace theorems by
    /// quantifying over foundation-defined `SITE_COUNT` arithmetic and
    /// `ConstraintRef` encoding disciplines.
    pub const NS_FOUNDATION: &str = "https://uor.foundation/foundation/";

    // XSD datatypes
    /// `xsd:string`.
    pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
    /// `xsd:integer`.
    pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
    /// `xsd:positiveInteger`.
    pub const XSD_POSITIVE_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#positiveInteger";
    /// `xsd:nonNegativeInteger`.
    pub const XSD_NON_NEGATIVE_INTEGER: &str =
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger";
    /// `xsd:boolean`.
    pub const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
    /// `xsd:decimal`.
    pub const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
    /// `xsd:dateTime`.
    pub const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
    /// `xsd:hexBinary`.
    pub const XSD_HEX_BINARY: &str = "http://www.w3.org/2001/XMLSchema#hexBinary";
    /// `owl:Thing`.
    pub const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
    /// `owl:Class`.
    pub const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
    /// `rdf:List`.
    pub const RDF_LIST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#List";
}
