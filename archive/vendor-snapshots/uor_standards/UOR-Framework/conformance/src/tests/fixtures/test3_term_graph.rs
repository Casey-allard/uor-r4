//! Test 3: Term graph with Application, Literal, and Datum.
//!
//! Validates: `schema:Application` + `schema:Literal` (denotes) + `schema:Datum`.
//! `Datum` and `Term` are `owl:disjointWith`, so a `Literal` (subclass of Term)
//! can *refer to* a `Datum` via `schema:denotes` without violating disjointness.

/// Instance graph for Test 3: Term graph.
pub const TEST3_TERM_GRAPH: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .

# A datum: raw byte content
<https://uor.foundation/instance/datum-hello>
    a               owl:NamedIndividual, schema:Datum ;
    schema:value    "5"^^xsd:nonNegativeInteger .

# A literal term that denotes the datum
<https://uor.foundation/instance/literal-hello>
    a               owl:NamedIndividual, schema:Literal ;
    schema:denotes  <https://uor.foundation/instance/datum-hello> .

# An application: applying a function term to an argument
<https://uor.foundation/instance/app-neg-x>
    a               owl:NamedIndividual, schema:Application ;
    schema:operator <https://uor.foundation/instance/term-neg> ;
    schema:argument <https://uor.foundation/instance/literal-hello> .

<https://uor.foundation/instance/term-neg>
    a               owl:NamedIndividual, schema:Term .

# A triad: (address, glyph, datum) triple
<https://uor.foundation/instance/triad-1>
    a               owl:NamedIndividual, schema:Triad .
"#;
