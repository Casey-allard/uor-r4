//! Test 12: Factorization oracle case study.
//!
//! Validates a full PRISM pipeline: CoordinateQuery → DihedralFactorizationResolver →
//! Partition + FreeRank → Observable → TransformCertificate → Trace.

/// Instance graph for Test 12: Factorization oracle.
pub const TEST12_FACTORIZATION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix query:      <https://uor.foundation/query/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix op:         <https://uor.foundation/op/> .

# 1. Query — coordinate query for factorization
<https://uor.foundation/instance/fact/query>
    a               owl:NamedIndividual, query:CoordinateQuery ;
    query:hasTriadProjection query:TwoAdicValuation .

# 2. Resolver — dihedral factorization
<https://uor.foundation/instance/fact/resolver>
    a                   owl:NamedIndividual, resolver:DihedralFactorizationResolver ;
    resolver:strategy   "dihedral-factorization" .

# 3. Partition with site budget
<https://uor.foundation/instance/fact/partition>
    a                       owl:NamedIndividual, partition:Partition ;
    partition:wittLength       "8"^^xsd:positiveInteger ;
    partition:siteBudget   <https://uor.foundation/instance/fact/budget> .

<https://uor.foundation/instance/fact/budget>
    a                       owl:NamedIndividual, partition:FreeRank ;
    partition:totalSites   "8"^^xsd:nonNegativeInteger ;
    partition:pinnedCount   "8"^^xsd:nonNegativeInteger ;
    partition:freeRank      "0"^^xsd:nonNegativeInteger ;
    partition:isClosed      true .

# 4. Observable — dihedral metric observation
<https://uor.foundation/instance/fact/obs>
    a                   owl:NamedIndividual, observable:Observable ;
    observable:value    "42"^^xsd:integer ;
    observable:source   <https://uor.foundation/instance/fact/partition> .

# 5. Certificate
<https://uor.foundation/instance/fact/cert>
    a                   owl:NamedIndividual, cert:TransformCertificate ;
    cert:certifies      <https://uor.foundation/instance/fact/obs> ;
    cert:verified       true .

# 6. Trace
<https://uor.foundation/instance/fact/trace>
    a                   owl:NamedIndividual, trace:ComputationTrace ;
    trace:certifiedBy   <https://uor.foundation/instance/fact/cert> .
"#;
