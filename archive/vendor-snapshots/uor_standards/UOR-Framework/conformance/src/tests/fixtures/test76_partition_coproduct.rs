/// SHACL test 76: PartitionCoproduct with leftSummand and rightSummand (Amendment 37, Gap 8).
pub const TEST76_PARTITION_COPRODUCT: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix partition: <https://uor.foundation/partition/> .

partition:ex_pc_76 a owl:NamedIndividual, partition:PartitionCoproduct ;
    partition:leftSummand partition:ex_part_c ;
    partition:rightSummand partition:ex_part_d .

partition:ex_part_c a owl:NamedIndividual, partition:Partition .
partition:ex_part_d a owl:NamedIndividual, partition:Partition .
"#;
