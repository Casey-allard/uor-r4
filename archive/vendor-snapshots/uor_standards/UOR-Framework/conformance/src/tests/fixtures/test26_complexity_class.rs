/// SHACL test 26: Complexity class vocabulary — typed resolver complexity.
pub const TEST26_COMPLEXITY_CLASS: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver: <https://uor.foundation/resolver/> .

# ComplexityClass vocabulary individuals
resolver:ConstantTime a resolver:ComplexityClass .
resolver:LogarithmicTime a resolver:ComplexityClass .
resolver:LinearTime a resolver:ComplexityClass .
resolver:ExponentialTime a resolver:ComplexityClass .

# Resolver instances with typed complexity
<https://uor.foundation/instance/eval-resolver>
    a resolver:EvaluationResolver ;
    resolver:hasComplexityClass resolver:ExponentialTime ;
    resolver:strategy "Direct enumeration" .

<https://uor.foundation/instance/canon-resolver>
    a resolver:CanonicalFormResolver ;
    resolver:hasComplexityClass resolver:LinearTime ;
    resolver:strategy "Term rewriting" .

<https://uor.foundation/instance/dihedral-resolver>
    a resolver:DihedralFactorizationResolver ;
    resolver:hasComplexityClass resolver:LogarithmicTime ;
    resolver:strategy "Dihedral factorization" .
"#;
