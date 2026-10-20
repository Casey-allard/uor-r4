//! SHACL test 167: `type:MetricAxis` vocabulary — the three metric axes.
//!
//! `MetricAxis` is a typed controlled vocabulary identifying the vertical
//! (ring/additive), horizontal (Hamming/bitwise), and diagonal (incompatibility)
//! metric axes of UOR geometry.

/// Instance graph for Test 167: type:MetricAxis.
pub const TEST167_METRIC_AXIS: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix type: <https://uor.foundation/type/> .

type:verticalAxis
    a owl:NamedIndividual, type:MetricAxis .
"#;
