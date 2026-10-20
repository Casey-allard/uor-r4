/// SHACL test 44: FlatType with trivial HolonomyGroup — Amendment 30 (MN_2).
/// All individuals are linked: FlatType → HolonomyGroup, Monodromy → path + element.
pub const TEST44_MONODROMY_FLAT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

type:ex_flat_44 a owl:NamedIndividual, type:FlatType ;
    type:holonomyGroup  observable:ex_holonomy_44 ;
    type:monodromyClass observable:ex_mc_44 .

observable:ex_holonomy_44 a owl:NamedIndividual, observable:HolonomyGroup ;
    observable:holonomyGroupOrder "1"^^xsd:positiveInteger .

observable:ex_monodromy_44 a owl:NamedIndividual, observable:Monodromy ;
    observable:monodromyLoop    observable:ex_path_44 ;
    observable:monodromyElement observable:ex_de_44 ;
    observable:isTrivialMonodromy "true"^^xsd:boolean .

observable:ex_de_44 a owl:NamedIndividual, observable:DihedralElement ;
    observable:isIdentityElement "true"^^xsd:boolean ;
    observable:elementOrder      "1"^^xsd:positiveInteger .

observable:ex_path_44 a owl:NamedIndividual, observable:ClosedConstraintPath ;
    observable:pathLength "3"^^xsd:nonNegativeInteger .

observable:ex_mc_44 a owl:NamedIndividual, observable:MonodromyClass .
"#;
