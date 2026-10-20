//! SHACL test 270: type:Constraint new subclasses.

/// Instance graph for Test 270: HammingConstraint, SiteConstraint,
/// AffineConstraint instances.
pub const TEST270_CONSTRAINT_SUBCLASSES: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix type: <https://uor.foundation/type/> .

type:ex_hamming_270 a owl:NamedIndividual, type:HammingConstraint .
type:ex_site_270   a owl:NamedIndividual, type:SiteConstraint .
type:ex_affine_270  a owl:NamedIndividual, type:AffineConstraint .
"#;
