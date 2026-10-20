/// SHACL test 75: PartitionProduct with leftFactor and rightFactor (Amendment 37, Gap 8).
pub const TEST75_PARTITION_PRODUCT: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix partition: <https://uor.foundation/partition/> .

partition:ex_pp_75 a owl:NamedIndividual, partition:PartitionProduct ;
    partition:leftFactor partition:ex_part_a ;
    partition:rightFactor partition:ex_part_b .

partition:ex_part_a a owl:NamedIndividual, partition:Partition .
partition:ex_part_b a owl:NamedIndividual, partition:Partition .
"#;
