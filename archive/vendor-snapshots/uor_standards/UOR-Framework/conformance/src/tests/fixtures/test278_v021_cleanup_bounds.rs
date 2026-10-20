//! Test 278: v0.2.1 Phase 7a cleanup bounds — SatBound, TimingBound,
//! ConstraintDefaults.
//!
//! Validates SHACL coverage for the Phase 7a ontology additions that back
//! the parametric pipeline-driver rewrites:
//! - reduction:SatBound (TwoSatBound, HornSatBound)
//! - reduction:TimingBound (PreflightTimingBound, RuntimeTimingBound)
//! - type:ConstraintDefaults (ResidueDefaultModulus)

/// Instance graph for Test 278: v0.2.1 Phase 7a bounds.
pub const TEST278_V021_CLEANUP_BOUNDS: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction:  <https://uor.foundation/reduction/> .
@prefix type:       <https://uor.foundation/type/> .

# 1. reduction:SatBound — 2-SAT decider bounds
<https://uor.foundation/instance/bounds/two_sat>
    a                   owl:NamedIndividual, reduction:SatBound ;
    reduction:maxVarCount "256"^^xsd:nonNegativeInteger ;
    reduction:maxClauseCount "512"^^xsd:nonNegativeInteger ;
    reduction:maxLiteralsPerClause "2"^^xsd:nonNegativeInteger .

# 2. reduction:SatBound — Horn-SAT decider bounds
<https://uor.foundation/instance/bounds/horn_sat>
    a                   owl:NamedIndividual, reduction:SatBound ;
    reduction:maxVarCount "256"^^xsd:nonNegativeInteger ;
    reduction:maxClauseCount "512"^^xsd:nonNegativeInteger ;
    reduction:maxLiteralsPerClause "8"^^xsd:nonNegativeInteger .

# 3. reduction:TimingBound — preflight stage budget
<https://uor.foundation/instance/bounds/preflight_timing>
    a                   owl:NamedIndividual, reduction:TimingBound ;
    reduction:preflightBudgetNs "10000000"^^xsd:nonNegativeInteger .

# 4. reduction:TimingBound — runtime stage budget
<https://uor.foundation/instance/bounds/runtime_timing>
    a                   owl:NamedIndividual, reduction:TimingBound ;
    reduction:runtimeBudgetNs "10000000"^^xsd:nonNegativeInteger .

# 5. type:ConstraintDefaults — default residue modulus
<https://uor.foundation/instance/defaults/residue_modulus>
    a                   owl:NamedIndividual, type:ConstraintDefaults ;
    type:defaultValue   "256"^^xsd:integer .
"#;
