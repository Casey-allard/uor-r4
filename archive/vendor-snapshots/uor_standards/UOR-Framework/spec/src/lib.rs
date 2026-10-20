//! UOR Foundation ontology encoded as typed Rust data.
//!
//! The `uor-ontology` crate provides the complete UOR Foundation ontology —
//! all namespaces, classes, properties, and named individuals —
//! as static Rust data structures, along with serializers that produce
//! JSON-LD, Turtle, and N-Triples output.
//!
//! Authoritative counts are in [`counts`].
//!
//! # Entry Point
//!
//! ```
//! let ontology = uor_ontology::Ontology::full();
//! assert_eq!(ontology.namespaces.len(), uor_ontology::counts::NAMESPACES);
//! ```
//!
//! # Serialization
//!
//! Requires the `serializers` feature (enabled by default).
//!
//! ```
//! let ontology = uor_ontology::Ontology::full();
//! let json_ld = uor_ontology::serializer::jsonld::to_json_ld(ontology);
//! let turtle  = uor_ontology::serializer::turtle::to_turtle(ontology);
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `serde` | yes | Adds `Serialize` derive to all model types |
//! | `serializers` | yes | JSON-LD, Turtle, and N-Triples serializers (pulls in `serde_json`) |
//!
//! This crate is internal (not published). The published crate `uor-foundation`
//! is generated from this data by the `uor-crate` client.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

pub mod counts;
pub mod model;
pub mod namespaces;
#[cfg(feature = "serializers")]
pub mod serializer;

pub use model::iris;
pub use model::{
    AnnotationProperty, Class, Individual, IndividualValue, Namespace, NamespaceModule, Ontology,
    Property, PropertyKind, Space,
};

impl Ontology {
    /// Returns the complete UOR Foundation ontology with all namespaces
    /// and all amendments applied.
    ///
    /// Assembly order follows the dependency graph specified in the UOR Foundation
    /// completion plan:
    /// `u → schema → op → query → resolver → type → partition → foundation →
    ///  observable → carry → homology → cohomology → proof → derivation → trace → cert → morphism → state → reduction → convergence → division → interaction → monoidal → operad → effect → predicate → parallel → stream → failure → linear → recursion → region → boundary → conformance`
    #[must_use]
    pub fn full() -> &'static Ontology {
        static ONTOLOGY: std::sync::OnceLock<Ontology> = std::sync::OnceLock::new();
        ONTOLOGY.get_or_init(|| Ontology {
            version: env!("CARGO_PKG_VERSION"),
            base_iri: "https://uor.foundation/",
            namespaces: vec![
                namespaces::u::module(),
                namespaces::schema::module(),
                namespaces::op::module(),
                namespaces::query::module(),
                namespaces::resolver::module(),
                namespaces::type_::module(),
                namespaces::partition::module(),
                namespaces::foundation::module(),
                namespaces::observable::module(),
                namespaces::carry::module(),
                namespaces::homology::module(),
                namespaces::cohomology::module(),
                namespaces::proof::module(),
                namespaces::derivation::module(),
                namespaces::trace::module(),
                namespaces::cert::module(),
                namespaces::morphism::module(),
                namespaces::state::module(),
                namespaces::reduction::module(),
                namespaces::convergence::module(),
                namespaces::division::module(),
                namespaces::interaction::module(),
                namespaces::monoidal::module(),
                namespaces::operad::module(),
                namespaces::effect::module(),
                namespaces::predicate::module(),
                namespaces::parallel::module(),
                namespaces::stream::module(),
                namespaces::failure::module(),
                namespaces::linear::module(),
                namespaces::recursion::module(),
                namespaces::region::module(),
                namespaces::boundary::module(),
                namespaces::conformance_::module(),
            ],
            annotation_properties: vec![model::annotation_space_property()],
        })
    }

    /// Looks up a namespace module by its short prefix (e.g., `"u"`, `"schema"`).
    #[must_use]
    pub fn find_namespace(&self, prefix: &str) -> Option<&NamespaceModule> {
        self.namespaces
            .iter()
            .find(|m| m.namespace.prefix == prefix)
    }

    /// Looks up a namespace module by its full IRI (e.g., `"https://uor.foundation/u/"`).
    #[must_use]
    pub fn find_namespace_by_iri(&self, iri: &str) -> Option<&NamespaceModule> {
        self.namespaces.iter().find(|m| m.namespace.iri == iri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_count() {
        assert_eq!(Ontology::full().namespaces.len(), counts::NAMESPACES);
    }

    #[test]
    fn class_count() {
        let total: usize = Ontology::full()
            .namespaces
            .iter()
            .map(|m| m.classes.len())
            .sum();
        assert_eq!(total, counts::CLASSES);
    }

    #[test]
    fn property_count() {
        assert_eq!(Ontology::full().property_count(), counts::PROPERTIES);
    }

    #[test]
    fn individual_count() {
        let total: usize = Ontology::full()
            .namespaces
            .iter()
            .map(|m| m.individuals.len())
            .sum();
        assert_eq!(total, counts::INDIVIDUALS);
    }

    /// Drift guard: every property marked `required: true` must be
    /// asserted on every individual whose `rdf:type` is the
    /// property's domain or a transitive subclass of it. Catches
    /// amendment-driven drift (e.g., "Amendment N adds new required
    /// property P to class C but forgets to backfill existing C
    /// individuals") within ~50ms of `cargo test` — faster than any
    /// downstream codegen + Lean-build roundtrip.
    #[test]
    fn all_required_properties_asserted() {
        let ontology = Ontology::full();

        // Index classes by IRI so we can walk the extends chain for
        // subclass closure.
        let class_by_iri: std::collections::HashMap<&str, &model::Class> = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.classes.iter())
            .map(|c| (c.id, c))
            .collect();

        fn subclass_closure<'a>(
            root: &'a str,
            class_by_iri: &std::collections::HashMap<&'a str, &'a model::Class>,
        ) -> std::collections::HashSet<&'a str> {
            let mut result: std::collections::HashSet<&'a str> = std::collections::HashSet::new();
            result.insert(root);
            let mut changed = true;
            while changed {
                changed = false;
                for (child_iri, child) in class_by_iri {
                    if result.contains(child_iri) {
                        continue;
                    }
                    if child.subclass_of.iter().any(|p| result.contains(p)) {
                        result.insert(child_iri);
                        changed = true;
                    }
                }
            }
            result
        }

        let mut missing: Vec<String> = Vec::new();

        for module in &ontology.namespaces {
            for prop in &module.properties {
                if !prop.required {
                    continue;
                }
                let Some(domain_iri) = prop.domain else {
                    missing.push(format!("{} (required: true but no domain)", prop.id));
                    continue;
                };
                let domain_closure = subclass_closure(domain_iri, &class_by_iri);
                for m2 in &ontology.namespaces {
                    for ind in &m2.individuals {
                        if !domain_closure.contains(ind.type_) {
                            continue;
                        }
                        let has_assertion = ind.properties.iter().any(|(p, _)| *p == prop.id);
                        if !has_assertion {
                            missing.push(format!(
                                "{} :: {} (required by domain <{}>)",
                                ind.id, prop.label, domain_iri
                            ));
                        }
                    }
                }
            }
        }

        assert!(
            missing.is_empty(),
            "required-property drift detected ({} missing assertions):\n  {}",
            missing.len(),
            missing.join("\n  ")
        );
    }

    #[test]
    fn all_class_iris_unique() {
        let mut iris = std::collections::HashSet::new();
        for module in &Ontology::full().namespaces {
            for class in &module.classes {
                assert!(iris.insert(class.id), "Duplicate class IRI: {}", class.id);
            }
        }
    }

    #[test]
    fn all_property_iris_unique() {
        let mut iris = std::collections::HashSet::new();
        for module in &Ontology::full().namespaces {
            for prop in &module.properties {
                assert!(iris.insert(prop.id), "Duplicate property IRI: {}", prop.id);
            }
        }
    }

    #[test]
    fn all_individual_iris_unique() {
        let mut iris = std::collections::HashSet::new();
        for module in &Ontology::full().namespaces {
            for ind in &module.individuals {
                assert!(iris.insert(ind.id), "Duplicate individual IRI: {}", ind.id);
            }
        }
    }

    #[test]
    fn space_annotations_on_all_namespaces() {
        for module in &Ontology::full().namespaces {
            // Every namespace must have a space classification.
            let _ = &module.namespace.space; // Space is non-optional; this compiles only if present.
        }
    }

    #[test]
    fn find_namespace_by_prefix() {
        let ontology = Ontology::full();
        let u = ontology.find_namespace("u");
        assert!(u.is_some());
        assert_eq!(
            u.map(|m| m.namespace.iri),
            Some("https://uor.foundation/u/")
        );
    }

    #[test]
    fn find_namespace_by_iri_works() {
        let ontology = Ontology::full();
        let schema = ontology.find_namespace_by_iri("https://uor.foundation/schema/");
        assert!(schema.is_some());
        assert_eq!(schema.map(|m| m.namespace.prefix), Some("schema"));
    }

    // ── v0.2.1 parametric metadata tests ───────────────────────────

    /// v0.2.1: the five Inhabitance verdict classes resolve in the ontology.
    #[test]
    fn v021_inhabitance_classes_present() {
        let ontology = Ontology::full();
        let expected = [
            "https://uor.foundation/cert/InhabitanceCertificate",
            "https://uor.foundation/proof/InhabitanceImpossibilityWitness",
            "https://uor.foundation/trace/InhabitanceSearchTrace",
            "https://uor.foundation/derivation/InhabitanceStep",
            "https://uor.foundation/derivation/InhabitanceCheckpoint",
        ];
        for iri in expected {
            let found = ontology
                .namespaces
                .iter()
                .flat_map(|m| m.classes.iter())
                .any(|c| c.id == iri);
            assert!(found, "missing v0.2.1 class: {iri}");
        }
    }

    /// v0.2.1: every `resolver:Resolver` subclass with a CertifyMapping
    /// individual must have non-empty `forResolver` / `producesCertificate`
    /// / `producesWitness` values. Catches drift when a new resolver is
    /// added to the ontology without the matching mapping.
    #[test]
    fn v021_resolver_certify_mappings_well_formed() {
        let ontology = Ontology::full();
        let mut mapping_count = 0usize;
        for ns in &ontology.namespaces {
            for ind in &ns.individuals {
                if ind.type_ != "https://uor.foundation/resolver/CertifyMapping" {
                    continue;
                }
                mapping_count += 1;
                let for_resolver = ind
                    .properties
                    .iter()
                    .find(|(k, _)| *k == "https://uor.foundation/resolver/forResolver");
                let produces_cert = ind
                    .properties
                    .iter()
                    .find(|(k, _)| *k == "https://uor.foundation/resolver/producesCertificate");
                let produces_witness = ind
                    .properties
                    .iter()
                    .find(|(k, _)| *k == "https://uor.foundation/resolver/producesWitness");
                assert!(
                    for_resolver.is_some(),
                    "CertifyMapping {} missing forResolver",
                    ind.id
                );
                assert!(
                    produces_cert.is_some(),
                    "CertifyMapping {} missing producesCertificate",
                    ind.id
                );
                assert!(
                    produces_witness.is_some(),
                    "CertifyMapping {} missing producesWitness",
                    ind.id
                );
            }
        }
        // v0.2.1 ships 4 mappings: Tower, Incremental, GroundingAware, Inhabitance.
        assert!(
            mapping_count >= 4,
            "expected ≥4 CertifyMapping individuals, got {mapping_count}"
        );
    }

    /// v0.2.1: every `conformance:Shape` individual must carry a
    /// `surfaceForm` so the parametric EBNF emitter can produce a
    /// production for it.
    #[test]
    fn v021_conformance_shapes_have_surface_form() {
        let ontology = Ontology::full();
        let surface_form_iri = "https://uor.foundation/conformance/surfaceForm";
        for ns in &ontology.namespaces {
            for ind in &ns.individuals {
                if ind.type_ != "https://uor.foundation/conformance/Shape" {
                    continue;
                }
                let has_surface = ind.properties.iter().any(|(k, _)| *k == surface_form_iri);
                assert!(
                    has_surface,
                    "conformance:Shape {} missing surfaceForm annotation",
                    ind.id
                );
            }
        }
    }

    /// v0.2.1: every `reduction:PipelineFailureReason` individual must have
    /// at least one `reduction:FailureField` individual referencing it.
    #[test]
    fn v021_pipeline_failure_fields_cover_all_reasons() {
        let ontology = Ontology::full();
        let reason_type = "https://uor.foundation/reduction/PipelineFailureReason";
        let field_type = "https://uor.foundation/reduction/FailureField";
        let of_failure_iri = "https://uor.foundation/reduction/ofFailure";
        let all_inds: Vec<_> = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.individuals.iter())
            .collect();
        for ind in &all_inds {
            if ind.type_ != reason_type {
                continue;
            }
            // Check at least one FailureField points at this reason.
            let covered = all_inds.iter().any(|f| {
                f.type_ == field_type
                    && f.properties.iter().any(|(k, v)| {
                        *k == of_failure_iri
                            && matches!(v, model::IndividualValue::IriRef(iri) if iri == &ind.id)
                    })
            });
            assert!(
                covered,
                "reduction:PipelineFailureReason {} has no FailureField",
                ind.id
            );
        }
    }

    /// v0.2.1: the `predicate:InhabitanceDispatchTable` has exactly 3
    /// dispatch rules with distinct priorities {0, 1, 2}.
    #[test]
    fn v021_inhabitance_dispatch_table_well_formed() {
        let ontology = Ontology::full();
        let rule_type = "https://uor.foundation/predicate/DispatchRule";
        let priority_iri = "https://uor.foundation/predicate/dispatchPriority";
        let rules: Vec<_> = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.individuals.iter())
            .filter(|i| {
                i.type_ == rule_type
                    && i.id
                        .starts_with("https://uor.foundation/predicate/inhabitance_rule_")
            })
            .collect();
        assert_eq!(rules.len(), 3, "expected 3 inhabitance dispatch rules");
        let mut priorities: Vec<i64> = rules
            .iter()
            .filter_map(|r| {
                r.properties.iter().find_map(|(k, v)| {
                    if *k == priority_iri {
                        if let model::IndividualValue::Int(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();
        priorities.sort_unstable();
        assert_eq!(priorities, vec![0, 1, 2]);
    }

    /// v0.2.1: every op:IH_* identity has a proof individual.
    #[test]
    fn v021_inhabitance_identities_have_proofs() {
        let ontology = Ontology::full();
        let identities = ["IH_1", "IH_2a", "IH_2b", "IH_3"];
        for id_name in identities {
            let id_iri = format!("https://uor.foundation/op/{id_name}");
            let proof_found = ontology
                .namespaces
                .iter()
                .flat_map(|m| m.individuals.iter())
                .any(|ind| {
                    ind.properties.iter().any(|(k, v)| {
                        *k == "https://uor.foundation/proof/provesIdentity"
                            && matches!(v, model::IndividualValue::IriRef(iri) if *iri == id_iri)
                    })
                });
            assert!(proof_found, "op:{id_name} has no proof individual");
        }
    }
}
