//! Test 8: Free rank accounting (Amendment 9).
//!
//! Validates: `partition:FreeRank` + `partition:SiteIndex` with
//! `isClosed`, `pinnedCount`, `freeRank`, `hasSite`, `hasBinding`.

/// Instance graph for Test 8: Free rank.
pub const TEST8_FREE_RANK: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix type:       <https://uor.foundation/type/> .

# A partition with free rank for R_4
<https://uor.foundation/instance/partition-R4-fb>
    a                       owl:NamedIndividual, partition:Partition ;
    partition:wittLength       "4"^^xsd:positiveInteger ;
    partition:freeRank   <https://uor.foundation/instance/budget-R4> .

# The free rank: 4 total sites, 2 pinned, 2 free
<https://uor.foundation/instance/budget-R4>
    a                       owl:NamedIndividual, partition:FreeRank ;
    partition:totalSites   "4"^^xsd:nonNegativeInteger ;
    partition:pinnedCount   "2"^^xsd:nonNegativeInteger ;
    partition:freeRank     "2"^^xsd:nonNegativeInteger ;
    partition:isClosed      false ;
    partition:hasSite      <https://uor.foundation/instance/site-0> ;
    partition:hasSite      <https://uor.foundation/instance/site-1> ;
    partition:hasSite      <https://uor.foundation/instance/site-2> ;
    partition:hasSite      <https://uor.foundation/instance/site-3> ;
    partition:hasBinding    <https://uor.foundation/instance/pinning-0> ;
    partition:hasBinding    <https://uor.foundation/instance/pinning-1> .

# Individual site coordinates
<https://uor.foundation/instance/site-0>
    a                       owl:NamedIndividual, partition:SiteIndex ;
    partition:sitePosition "0"^^xsd:nonNegativeInteger ;
    partition:siteState    "pinned" .

<https://uor.foundation/instance/site-1>
    a                       owl:NamedIndividual, partition:SiteIndex ;
    partition:sitePosition "1"^^xsd:nonNegativeInteger ;
    partition:siteState    "pinned" .

<https://uor.foundation/instance/site-2>
    a                       owl:NamedIndividual, partition:SiteIndex ;
    partition:sitePosition "2"^^xsd:nonNegativeInteger ;
    partition:siteState    "free" .

<https://uor.foundation/instance/site-3>
    a                       owl:NamedIndividual, partition:SiteIndex ;
    partition:sitePosition "3"^^xsd:nonNegativeInteger ;
    partition:siteState    "free" .

# Site bindings — record which constraint bound each site
<https://uor.foundation/instance/pinning-0>
    a                       owl:NamedIndividual, partition:SiteBinding ;
    partition:pinsCoordinate <https://uor.foundation/instance/site-0> ;
    partition:pinnedBy      <https://uor.foundation/instance/constraint-residue> .

<https://uor.foundation/instance/pinning-1>
    a                       owl:NamedIndividual, partition:SiteBinding ;
    partition:pinsCoordinate <https://uor.foundation/instance/site-1> ;
    partition:pinnedBy      <https://uor.foundation/instance/constraint-residue> .

# The constraint that does the pinning (declared as type:ResidueConstraint)
<https://uor.foundation/instance/constraint-residue>
    a                       owl:NamedIndividual, type:ResidueConstraint .
"#;
