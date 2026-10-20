//! SHACL test 266: `WitnessDatum` enforcement type.

/// Instance graph for Test 266: WitnessDatum, GroundedCoordinate,
/// GroundedTuple, ValidatedWrapper, WitnessDerivation, WitnessFreeRank,
/// GroundedValueMarker, and MintingSession.
pub const TEST266_WITNESS_DATUM: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix conformance: <https://uor.foundation/conformance/> .

conformance:ex_witness_datum_266 a owl:NamedIndividual, conformance:WitnessDatum .
conformance:ex_grounded_coord_266 a owl:NamedIndividual, conformance:GroundedCoordinate .
conformance:ex_grounded_tuple_266 a owl:NamedIndividual, conformance:GroundedTuple .
conformance:ex_grounded_value_266 a owl:NamedIndividual, conformance:GroundedValueMarker .
conformance:ex_validated_266 a owl:NamedIndividual, conformance:ValidatedWrapper .
conformance:ex_derivation_266 a owl:NamedIndividual, conformance:WitnessDerivation .
conformance:ex_site_budget_266 a owl:NamedIndividual, conformance:WitnessFreeRank .
conformance:ex_minting_session_266 a owl:NamedIndividual, conformance:MintingSession .
"#;
