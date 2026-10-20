//! Extracts all ontology terms from the spec for use in documentation generation.

use uor_ontology::{Class, Individual, Namespace, NamespaceModule, Ontology, Property};

/// A flattened index of all terms in the ontology.
pub struct OntologyIndex {
    /// Ontology version string.
    pub version: &'static str,
    /// All namespace modules in assembly order.
    pub modules: Vec<&'static NamespaceModule>,
    /// All classes, flattened.
    pub classes: Vec<&'static Class>,
    /// All properties, flattened.
    pub properties: Vec<&'static Property>,
    /// All individuals, flattened.
    pub individuals: Vec<&'static Individual>,
}

impl OntologyIndex {
    /// Builds the index from the live spec ontology.
    pub fn from_spec() -> Self {
        let ontology = Ontology::full();
        let mut modules = Vec::new();
        let mut classes = Vec::new();
        let mut properties = Vec::new();
        let mut individuals = Vec::new();

        for module in &ontology.namespaces {
            modules.push(module);
            for class in &module.classes {
                classes.push(class);
            }
            for prop in &module.properties {
                properties.push(prop);
            }
            for ind in &module.individuals {
                individuals.push(ind);
            }
        }

        Self {
            version: ontology.version,
            modules,
            classes,
            properties,
            individuals,
        }
    }

    /// Returns true if the given IRI is a known class IRI.
    pub fn is_class(&self, iri: &str) -> bool {
        self.classes.iter().any(|c| c.id == iri)
    }

    /// Returns true if the given IRI is a known property IRI.
    pub fn is_property(&self, iri: &str) -> bool {
        self.properties.iter().any(|p| p.id == iri)
    }

    /// Returns true if the given IRI is a known individual IRI.
    pub fn is_individual(&self, iri: &str) -> bool {
        self.individuals.iter().any(|i| i.id == iri)
    }

    /// Finds a namespace module by prefix.
    pub fn find_module(&self, prefix: &str) -> Option<&&'static NamespaceModule> {
        self.modules.iter().find(|m| m.namespace.prefix == prefix)
    }

    /// Returns the namespace for a given IRI prefix.
    pub fn namespace_for_iri(&self, iri: &str) -> Option<&Namespace> {
        self.modules
            .iter()
            .find(|m| iri.starts_with(m.namespace.iri))
            .map(|m| &m.namespace)
    }
}
