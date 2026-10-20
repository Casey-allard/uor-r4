/// SHACL test 29: Coordinate kind vocabulary — typed coordinate queries.
pub const TEST29_TRIAD_PROJECTION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix query: <https://uor.foundation/query/> .
@prefix type:  <https://uor.foundation/type/> .

# TriadProjection vocabulary individuals
query:TwoAdicValuation a query:TriadProjection .
query:WalshHadamardImage a query:TriadProjection .
query:Address a query:TriadProjection .

# CoordinateQuery instances with typed triad projections
<https://uor.foundation/instance/stratum-query>
    a query:CoordinateQuery ;
    query:hasTriadProjection query:TwoAdicValuation .

<https://uor.foundation/instance/spectrum-query>
    a query:CoordinateQuery ;
    query:hasTriadProjection query:WalshHadamardImage .

<https://uor.foundation/instance/address-query>
    a query:CoordinateQuery ;
    query:hasTriadProjection query:Address .
"#;
