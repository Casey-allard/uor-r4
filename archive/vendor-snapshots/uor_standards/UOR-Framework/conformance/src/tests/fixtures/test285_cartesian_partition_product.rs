/// SHACL test 285: CartesianPartitionProduct with leftCartesianFactor and
/// rightCartesianFactor (Product/Coproduct Completion Amendment, Gap 3).
///
/// Validates that an instance of `partition:CartesianPartitionProduct`
/// with both required Cartesian-factor links satisfies the
/// `CartesianPartitionProductShape` from `conformance/shapes/uor-shapes.ttl`.
pub const TEST285_CARTESIAN_PARTITION_PRODUCT: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix partition: <https://uor.foundation/partition/> .

partition:ex_cpp_285 a owl:NamedIndividual, partition:CartesianPartitionProduct ;
    partition:leftCartesianFactor partition:ex_part_285a ;
    partition:rightCartesianFactor partition:ex_part_285b .

partition:ex_part_285a a owl:NamedIndividual, partition:Partition .
partition:ex_part_285b a owl:NamedIndividual, partition:Partition .
"#;
