/// SHACL test 46: MonodromyResolver pipeline — ConstrainedType → MonodromyResolver →
/// HolonomyGroup → MonodromyClass → TwistedType classification (Amendment 30).
pub const TEST46_MONODROMY_PIPELINE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. Input — ConstrainedType whose holonomy is being computed
type:ex_ct_46 a owl:NamedIndividual, type:ConstrainedType ;
    type:holonomyGroup  observable:ex_hg_46 ;
    type:monodromyClass observable:ex_mc_46 .

# 2. MonodromyResolver — computes holonomy from closed paths
resolver:ex_mr_46 a owl:NamedIndividual, resolver:MonodromyResolver ;
    resolver:monodromyTarget type:ex_ct_46 ;
    resolver:holonomyResult  observable:ex_hg_46 .

# 3. ClosedConstraintPath — a loop in the constraint nerve
observable:ex_path_46 a owl:NamedIndividual, observable:ClosedConstraintPath ;
    observable:pathLength "4"^^xsd:nonNegativeInteger .

# 4. Monodromy — net dihedral transformation on the loop
observable:ex_mono_46 a owl:NamedIndividual, observable:Monodromy ;
    observable:monodromyLoop    observable:ex_path_46 ;
    observable:monodromyElement observable:ex_de_46 ;
    observable:isTrivialMonodromy "false"^^xsd:boolean .

# 5. DihedralElement — a generator of the holonomy group
observable:ex_de_46 a owl:NamedIndividual, observable:DihedralElement ;
    observable:isIdentityElement "false"^^xsd:boolean ;
    observable:elementOrder      "2"^^xsd:positiveInteger .

# 6. HolonomyGroup — the computed holonomy subgroup of D_{2^n}
observable:ex_hg_46 a owl:NamedIndividual, observable:HolonomyGroup ;
    observable:holonomyGroup      observable:ex_de_46 ;
    observable:holonomyGroupOrder "4"^^xsd:positiveInteger .

# 7. MonodromyClass — classification result
observable:ex_mc_46 a owl:NamedIndividual, observable:MonodromyClass .

# 8. Output — TwistedType (non-trivial holonomy)
type:ex_twisted_46 a owl:NamedIndividual, type:TwistedType ;
    type:holonomyGroup observable:ex_hg_46 .
"#;
