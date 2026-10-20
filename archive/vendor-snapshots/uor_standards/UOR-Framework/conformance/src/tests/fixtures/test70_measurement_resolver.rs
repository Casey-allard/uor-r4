/// SHACL test 70: Measurement resolver — MeasurementResolver with
/// collapseAmplitude, collapsedSite, measurementOutcome (Amendment 36).
pub const TEST70_MEASUREMENT_RESOLVER: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix type:       <https://uor.foundation/type/> .

# 1. MeasurementResolver
resolver:ex_mr_70 a owl:NamedIndividual, resolver:MeasurementResolver ;
    resolver:collapseAmplitude "0.707"^^xsd:decimal ;
    resolver:collapsedSite partition:ex_fc_70 ;
    resolver:measurementOutcome type:ex_ct_70 .

# 2. Referenced site and type
partition:ex_fc_70 a owl:NamedIndividual, partition:SiteIndex .
type:ex_ct_70 a owl:NamedIndividual, type:ConstrainedType .
"#;
