/// SHACL test 48: Phase diagram — CatastropheObservable + phaseN/phaseG +
/// PhaseBoundaryType + onResonanceLine (Amendment 31, PD_1–PD_5).
pub const TEST48_PHASE_DIAGRAM: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. CatastropheObservable at point (n=8, g=3) in the phase diagram
observable:ex_cat_48 a owl:NamedIndividual, observable:CatastropheObservable ;
    observable:phaseN "8"^^xsd:positiveInteger ;
    observable:phaseG "3"^^xsd:positiveInteger ;
    observable:onResonanceLine "true"^^xsd:boolean ;
    observable:phaseBoundaryType observable:PeriodBoundary .

# 2. Power-of-two boundary example at (n=16, g=4)
observable:ex_cat_48b a owl:NamedIndividual, observable:CatastropheObservable ;
    observable:phaseN "16"^^xsd:positiveInteger ;
    observable:phaseG "4"^^xsd:positiveInteger ;
    observable:onResonanceLine "false"^^xsd:boolean ;
    observable:phaseBoundaryType observable:PowerOfTwoBoundary .
"#;
