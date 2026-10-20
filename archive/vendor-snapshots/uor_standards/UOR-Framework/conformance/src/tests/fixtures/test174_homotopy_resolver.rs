//! SHACL test 174: `resolver:HomotopyResolver`.

/// Instance graph for Test 174: HomotopyResolver.
pub const TEST174_HOMOTOPY_RESOLVER: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

resolver:ex_hr_174 a owl:NamedIndividual, resolver:HomotopyResolver ;
    resolver:homotopyTarget resolver:ex_nerve_174 ;
    resolver:homotopyResult observable:ex_pi_174 .

resolver:ex_nerve_174 a owl:NamedIndividual, resolver:CechNerve .

observable:ex_pi_174 a owl:NamedIndividual, observable:HomotopyGroup .
"#;
