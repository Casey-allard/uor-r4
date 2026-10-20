/// SHACL test 45: TwistedType with non-trivial LiftObstruction — Amendment 30 (MN_7).
/// All individuals are linked: TwistedType → HolonomyGroup, Monodromy → path + element.
pub const TEST45_MONODROMY_TWISTED: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

type:ex_twisted_45 a owl:NamedIndividual, type:TwistedType ;
    type:holonomyGroup  observable:ex_holonomy_45 ;
    type:monodromyClass observable:ex_mc_45 .

observable:ex_holonomy_45 a owl:NamedIndividual, observable:HolonomyGroup ;
    observable:holonomyGroup      observable:ex_dihedral_45 ;
    observable:holonomyGroupOrder "4"^^xsd:positiveInteger .

observable:ex_monodromy_45 a owl:NamedIndividual, observable:Monodromy ;
    observable:monodromyLoop    observable:ex_path_45 ;
    observable:monodromyElement observable:ex_dihedral_45 ;
    observable:isTrivialMonodromy "false"^^xsd:boolean .

observable:ex_dihedral_45 a owl:NamedIndividual, observable:DihedralElement ;
    observable:isIdentityElement "false"^^xsd:boolean ;
    observable:elementOrder      "2"^^xsd:positiveInteger .

observable:ex_path_45 a owl:NamedIndividual, observable:ClosedConstraintPath ;
    observable:pathLength "4"^^xsd:nonNegativeInteger .

type:ex_obstruction_45 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionTrivial "false"^^xsd:boolean .

observable:ex_obstruction_class_45 a owl:NamedIndividual, observable:LiftObstructionClass .

observable:ex_mc_45 a owl:NamedIndividual, observable:MonodromyClass .
"#;
