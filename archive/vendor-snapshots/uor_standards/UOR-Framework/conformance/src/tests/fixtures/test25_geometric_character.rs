/// SHACL test 25: Geometric character vocabulary — typed operation characters.
pub const TEST25_GEOMETRIC_CHARACTER: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# GeometricCharacter vocabulary individuals
op:RingReflection a op:GeometricCharacter .
op:HypercubeReflection a op:GeometricCharacter .
op:Rotation a op:GeometricCharacter .
op:RotationInverse a op:GeometricCharacter .
op:Translation a op:GeometricCharacter .
op:Scaling a op:GeometricCharacter .
op:HypercubeTranslation a op:GeometricCharacter .
op:HypercubeProjection a op:GeometricCharacter .
op:HypercubeJoin a op:GeometricCharacter .

# Operations with hasGeometricCharacter
op:neg a op:Involution ;
    op:arity 1 ;
    op:hasGeometricCharacter op:RingReflection .

op:bnot a op:Involution ;
    op:arity 1 ;
    op:hasGeometricCharacter op:HypercubeReflection .

op:succ a op:UnaryOp ;
    op:arity 1 ;
    op:hasGeometricCharacter op:Rotation .

op:add a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:Translation .

op:sub a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:Translation .

op:mul a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:Scaling .

op:xor a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:HypercubeTranslation .

op:and a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:HypercubeProjection .

op:or a op:BinaryOp ;
    op:arity 2 ;
    op:hasGeometricCharacter op:HypercubeJoin .
"#;
