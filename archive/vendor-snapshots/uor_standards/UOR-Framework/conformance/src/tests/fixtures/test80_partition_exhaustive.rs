/// SHACL test 80: Partition with isExhaustive flag (Amendment 37, Gap 3).
pub const TEST80_PARTITION_EXHAUSTIVE: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix partition: <https://uor.foundation/partition/> .

partition:ex_part_80 a owl:NamedIndividual, partition:Partition ;
    partition:isExhaustive "true"^^xsd:boolean .
"#;
